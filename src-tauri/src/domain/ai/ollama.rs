use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use async_trait::async_trait;
use reqwest::{header, Client, Method};
use serde_json::{json, Value};
use tokio::{net::lookup_host, time::timeout};
use url::Url;

use super::{
    provider::{
        normalize_models, AiFinishReason, AiModel, AiUsage, MAXIMUM_MODELS, MAXIMUM_OUTPUT_TOKENS,
        MAXIMUM_RESPONSE_CHARS,
    },
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiProviderId,
    AiRequest, AiResponse,
};

const DEFAULT_HTTP_PORT: u16 = 80;
const MAXIMUM_RESOLVED_ADDRESSES: usize = 16;
const MAXIMUM_CHAT_REQUEST_BYTES: usize = 64 * 1_024;
const MAXIMUM_CHAT_RESPONSE_BYTES: usize = 4 * 1_024 * 1_024;
const MAXIMUM_MODELS_RESPONSE_BYTES: usize = 1_024 * 1_024;
const CHAT_TIMEOUT: Duration = Duration::from_secs(120);
const MODELS_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, Eq, PartialEq)]
struct OllamaEndpoint {
    hostname: String,
    origin: String,
    port: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransportFailure {
    InvalidEndpoint,
    Resolution,
    NonLoopbackResolution,
    RequestTooLarge,
    Network,
    Timeout,
    UnexpectedAddress,
    Redirect,
    UnsupportedEncoding,
    ResponseTooLarge,
    Http,
    InvalidResponse,
}

impl TransportFailure {
    fn provider_error(self) -> AiProviderError {
        let code = if self == Self::InvalidEndpoint {
            AiProviderErrorCode::Configuration
        } else {
            AiProviderErrorCode::Connection
        };
        let message = match self {
            Self::InvalidEndpoint => {
                "Ollama only supports local endpoints using localhost or 127.0.0.1."
            }
            Self::Resolution | Self::NonLoopbackResolution | Self::UnexpectedAddress => {
                "The local Ollama endpoint could not be validated."
            }
            Self::RequestTooLarge => "The Ollama request exceeds the safe size limit.",
            Self::Timeout => "Ollama did not respond in time.",
            Self::Redirect => "The Ollama server returned an unsupported redirect.",
            Self::UnsupportedEncoding => "The Ollama response encoding is not allowed.",
            Self::ResponseTooLarge => "Ollama returned more data than Ducky can safely process.",
            Self::Network | Self::Http | Self::InvalidResponse => {
                "The local Ollama connection was rejected."
            }
        };
        AiProviderError::new(AiProviderId::Ollama, code, message)
    }
}

#[derive(Debug, Default)]
pub(crate) struct OllamaProvider;

impl OllamaProvider {
    pub(crate) fn new() -> Self {
        Self
    }

    async fn request_json(
        &self,
        configuration: AiProviderConfiguration<'_>,
        method: Method,
        path: &'static str,
        payload: Option<Value>,
        maximum_response_bytes: usize,
        request_timeout: Duration,
    ) -> Result<Value, AiProviderError> {
        let endpoint =
            parse_endpoint(configuration.endpoint).map_err(TransportFailure::provider_error)?;
        let body = payload
            .map(|value| serde_json::to_vec(&value))
            .transpose()
            .map_err(|_| TransportFailure::InvalidResponse.provider_error())?
            .unwrap_or_default();
        if body.len() > MAXIMUM_CHAT_REQUEST_BYTES {
            return Err(TransportFailure::RequestTooLarge.provider_error());
        }
        let addresses = resolve_endpoint(&endpoint, request_timeout)
            .await
            .map_err(TransportFailure::provider_error)?;
        let mut last_failure = TransportFailure::Network;

        for address in addresses {
            match request_address(
                &endpoint,
                address,
                method.clone(),
                path,
                &body,
                maximum_response_bytes,
                request_timeout,
            )
            .await
            {
                Ok(value) => return Ok(value),
                Err(TransportFailure::Network) => {
                    last_failure = TransportFailure::Network;
                }
                Err(failure) => return Err(failure.provider_error()),
            }
        }

        Err(last_failure.provider_error())
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Ollama
    }

    fn display_name(&self) -> &'static str {
        "Ollama"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let model = configuration.model.trim();
        if model.is_empty() {
            return Err(AiProviderError::new(
                AiProviderId::Ollama,
                AiProviderErrorCode::Configuration,
                "Ollama requires an installed model.",
            ));
        }
        let value = timeout(
            CHAT_TIMEOUT,
            self.request_json(
                configuration,
                Method::POST,
                "/api/chat",
                Some(json!({
                    "model": model,
                    "messages": [{ "role": "user", "content": request.prompt }],
                    "options": { "num_predict": MAXIMUM_OUTPUT_TOKENS },
                    "stream": false
                })),
                MAXIMUM_CHAT_RESPONSE_BYTES,
                CHAT_TIMEOUT,
            ),
        )
        .await
        .map_err(|_| TransportFailure::Timeout.provider_error())??;
        parse_chat_response(value)
    }

    async fn list_models(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        let value = timeout(
            MODELS_TIMEOUT,
            self.request_json(
                configuration,
                Method::GET,
                "/api/tags",
                None,
                MAXIMUM_MODELS_RESPONSE_BYTES,
                MODELS_TIMEOUT,
            ),
        )
        .await
        .map_err(|_| TransportFailure::Timeout.provider_error())??;
        parse_models(value)
    }

    async fn test_connection(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<String, AiProviderError> {
        let models = self.list_models(configuration).await?;
        Ok(if models.is_empty() {
            "Ollama connected. No local models are installed."
        } else {
            "Ollama connected successfully."
        }
        .to_owned())
    }
}

fn parse_endpoint(value: &str) -> Result<OllamaEndpoint, TransportFailure> {
    if value.is_empty() || value != value.trim() || !has_exact_loopback_origin_shape(value) {
        return Err(TransportFailure::InvalidEndpoint);
    }
    let parsed = Url::parse(value).map_err(|_| TransportFailure::InvalidEndpoint)?;
    let hostname = parsed
        .host_str()
        .map(str::to_ascii_lowercase)
        .ok_or(TransportFailure::InvalidEndpoint)?;
    if parsed.scheme() != "http"
        || (hostname != "localhost" && hostname != "127.0.0.1")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || (parsed.path() != "/" && !parsed.path().is_empty())
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(TransportFailure::InvalidEndpoint);
    }
    let port = parsed.port().unwrap_or(DEFAULT_HTTP_PORT);
    let origin = format!(
        "http://{hostname}{}",
        if port == DEFAULT_HTTP_PORT {
            String::new()
        } else {
            format!(":{port}")
        }
    );
    Ok(OllamaEndpoint {
        hostname,
        origin,
        port,
    })
}

fn has_exact_loopback_origin_shape(value: &str) -> bool {
    let Some(scheme) = value.get(..7) else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http://") {
        return false;
    }
    let authority = value[7..].strip_suffix('/').unwrap_or(&value[7..]);
    if authority.contains('/') {
        return false;
    }
    let (hostname, port) = authority
        .rsplit_once(':')
        .map_or((authority, None), |(hostname, port)| (hostname, Some(port)));
    if !hostname.eq_ignore_ascii_case("localhost") && hostname != "127.0.0.1" {
        return false;
    }
    match port {
        None => true,
        Some(port) => {
            !port.is_empty()
                && port.len() <= 5
                && port.bytes().all(|character| character.is_ascii_digit())
        }
    }
}

async fn resolve_endpoint(
    endpoint: &OllamaEndpoint,
    request_timeout: Duration,
) -> Result<Vec<SocketAddr>, TransportFailure> {
    if endpoint.hostname == "127.0.0.1" {
        return Ok(vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            endpoint.port,
        )]);
    }
    let resolved = timeout(
        request_timeout,
        lookup_host((endpoint.hostname.as_str(), endpoint.port)),
    )
    .await
    .map_err(|_| TransportFailure::Timeout)?
    .map_err(|_| TransportFailure::Resolution)?
    .collect::<Vec<_>>();
    if resolved.is_empty() || resolved.len() > MAXIMUM_RESOLVED_ADDRESSES {
        return Err(TransportFailure::Resolution);
    }
    if resolved.iter().any(|address| !address.ip().is_loopback()) {
        return Err(TransportFailure::NonLoopbackResolution);
    }
    Ok(resolved
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

async fn request_address(
    endpoint: &OllamaEndpoint,
    address: SocketAddr,
    method: Method,
    path: &'static str,
    body: &[u8],
    maximum_response_bytes: usize,
    request_timeout: Duration,
) -> Result<Value, TransportFailure> {
    let client = Client::builder()
        .timeout(request_timeout)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .resolve(&endpoint.hostname, address)
        .build()
        .map_err(|_| TransportFailure::Network)?;
    let mut request = client
        .request(method, format!("{}{path}", endpoint.origin))
        .header(header::ACCEPT, "application/json")
        .header(header::ACCEPT_ENCODING, "identity")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, "Ducky/Ollama");
    if !body.is_empty() {
        request = request.body(body.to_vec());
    }
    let mut response = request.send().await.map_err(|error| {
        if error.is_timeout() {
            TransportFailure::Timeout
        } else {
            TransportFailure::Network
        }
    })?;
    let connected = response
        .remote_addr()
        .ok_or(TransportFailure::UnexpectedAddress)?;
    if !addresses_match(address.ip(), connected.ip()) {
        return Err(TransportFailure::UnexpectedAddress);
    }
    if response.status().is_redirection() {
        return Err(TransportFailure::Redirect);
    }
    if response
        .headers()
        .get(header::CONTENT_ENCODING)
        .is_some_and(|encoding| {
            encoding
                .to_str()
                .map_or(true, |value| !value.eq_ignore_ascii_case("identity"))
        })
    {
        return Err(TransportFailure::UnsupportedEncoding);
    }
    if response
        .content_length()
        .is_some_and(|length| length > maximum_response_bytes as u64)
    {
        return Err(TransportFailure::ResponseTooLarge);
    }
    let status = response.status();
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| TransportFailure::Network)?
    {
        if bytes.len() + chunk.len() > maximum_response_bytes {
            return Err(TransportFailure::ResponseTooLarge);
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        return Err(TransportFailure::Http);
    }
    serde_json::from_slice(&bytes).map_err(|_| TransportFailure::InvalidResponse)
}

fn addresses_match(expected: IpAddr, connected: IpAddr) -> bool {
    expected == connected
        || matches!(
            (expected, connected),
            (IpAddr::V4(expected), IpAddr::V6(connected))
                if connected.to_ipv4_mapped() == Some(expected)
        )
}

fn parse_chat_response(value: Value) -> Result<AiResponse, AiProviderError> {
    let content = value["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    if content.is_empty() {
        return Err(AiProviderError::new(
            AiProviderId::Ollama,
            AiProviderErrorCode::EmptyResponse,
            "Ollama returned an empty response.",
        ));
    }
    let usage = match (
        value["prompt_eval_count"].as_u64(),
        value["eval_count"].as_u64(),
    ) {
        (Some(input_tokens), Some(output_tokens)) => Some(AiUsage {
            input_tokens,
            output_tokens,
        }),
        _ => None,
    };
    Ok(AiResponse {
        provider_id: AiProviderId::Ollama,
        content,
        finish_reason: if value["done_reason"].as_str() == Some("length") {
            AiFinishReason::Length
        } else {
            AiFinishReason::Stop
        },
        usage,
    })
}

fn parse_models(value: Value) -> Result<Vec<AiModel>, AiProviderError> {
    let raw_models = value["models"].as_array().ok_or_else(|| {
        AiProviderError::new(
            AiProviderId::Ollama,
            AiProviderErrorCode::Connection,
            "Ollama returned an invalid model list.",
        )
    })?;
    if raw_models.len() > MAXIMUM_MODELS {
        return Err(TransportFailure::ResponseTooLarge.provider_error());
    }
    let mut models = Vec::with_capacity(raw_models.len());
    for model in raw_models {
        let id = model["model"].as_str().ok_or_else(|| {
            AiProviderError::new(
                AiProviderId::Ollama,
                AiProviderErrorCode::Connection,
                "Ollama returned an invalid model list.",
            )
        })?;
        let display_name = match model.get("name") {
            Some(Value::String(name)) => Some(name.clone()),
            Some(Value::Null) | None => None,
            Some(_) => {
                return Err(AiProviderError::new(
                    AiProviderId::Ollama,
                    AiProviderErrorCode::Connection,
                    "Ollama returned an invalid model list.",
                ));
            }
        };
        models.push(AiModel {
            id: id.to_owned(),
            display_name,
        });
    }
    Ok(normalize_models(models))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_policy_accepts_only_exact_loopback_http_origins() {
        assert!(parse_endpoint("http://localhost:11434").is_ok());
        assert!(parse_endpoint("http://127.0.0.1").is_ok());
        assert!(parse_endpoint("HTTP://LOCALHOST:80/").is_ok());
        for endpoint in [
            "https://localhost:11434",
            "http://[::1]:11434",
            "http://localhost:11434/api",
            "http://localhost:11434?query=1",
            "http://user@localhost:11434",
            "http://127.0.0.2:11434",
            " http://localhost:11434",
        ] {
            assert_eq!(
                parse_endpoint(endpoint),
                Err(TransportFailure::InvalidEndpoint)
            );
        }
    }

    #[test]
    fn address_verification_accepts_only_the_pinned_peer() {
        assert!(addresses_match(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        ));
        assert!(addresses_match(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            "::ffff:127.0.0.1".parse().unwrap()
        ));
        assert!(!addresses_match(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            "127.0.0.2".parse().unwrap()
        ));
    }

    #[test]
    fn response_parsers_preserve_electron_contracts() {
        let response = parse_chat_response(json!({
            "message": { "content": " hello " },
            "done_reason": "length",
            "prompt_eval_count": 4,
            "eval_count": 2
        }))
        .unwrap();
        assert_eq!(response.content, "hello");
        assert_eq!(response.finish_reason, AiFinishReason::Length);
        assert_eq!(
            response.usage,
            Some(AiUsage {
                input_tokens: 4,
                output_tokens: 2
            })
        );
        assert_eq!(
            parse_models(json!({
                "models": [{ "model": "llama3", "name": "Llama 3" }]
            }))
            .unwrap(),
            [AiModel {
                id: "llama3".to_owned(),
                display_name: Some("Llama 3".to_owned())
            }]
        );
    }
}
