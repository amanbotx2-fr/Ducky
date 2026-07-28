use std::{
    net::IpAddr,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use async_trait::async_trait;
use reqwest::{header, Client, Method};
use serde_json::{json, Value};
use url::Url;

use super::{
    provider::{
        normalize_models, AiFinishReason, AiModel, AiUsage, MAXIMUM_OUTPUT_TOKENS,
        MAXIMUM_RESPONSE_CHARS,
    },
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiProviderId,
    AiRequest, AiResponse,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAXIMUM_RESPONSE_BYTES: usize = 4 * 1_024 * 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestProtocol {
    ChatCompletions,
    Responses,
}

#[derive(Debug, Default)]
struct ProtocolState {
    base_url: String,
    resolved: Option<RequestProtocol>,
}

#[derive(Debug)]
enum RequestFailure {
    Status(u16),
    Provider(AiProviderError),
}

impl RequestFailure {
    fn into_provider_error(self) -> AiProviderError {
        match self {
            Self::Provider(error) => error,
            Self::Status(status) => connection_error(match status {
                401 | 403 => "Custom provider authentication failed.",
                429 => "Custom provider rate limit reached.",
                500..=599 => "Custom provider is temporarily unavailable.",
                _ => "Custom provider request failed.",
            }),
        }
    }

    fn is_unsupported_endpoint(&self) -> bool {
        matches!(self, Self::Status(404 | 405 | 501))
    }
}

#[derive(Debug)]
pub(crate) struct CustomProvider {
    client: Client,
    protocol: Mutex<ProtocolState>,
}

impl CustomProvider {
    pub(crate) fn new() -> Result<Self, AiProviderError> {
        Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map(|client| Self {
                client,
                protocol: Mutex::new(ProtocolState::default()),
            })
            .map_err(|_| connection_error("Custom provider client initialization failed."))
    }

    async fn request_json(
        &self,
        configuration: AiProviderConfiguration<'_>,
        method: Method,
        path: &'static str,
        payload: Option<Value>,
    ) -> Result<Value, RequestFailure> {
        let base_url =
            normalize_base_url(configuration.base_url).map_err(RequestFailure::Provider)?;
        let mut request = self
            .client
            .request(method, format!("{base_url}{path}"))
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(api_key) = configuration
            .api_key
            .map(str::trim)
            .filter(|api_key| !api_key.is_empty())
        {
            request = request.bearer_auth(api_key);
        }
        if let Some(payload) = payload {
            request = request.json(&payload);
        }
        let mut response = request.send().await.map_err(|_| {
            RequestFailure::Provider(connection_error(
                "Could not connect to the custom provider.",
            ))
        })?;
        if !response.status().is_success() {
            return Err(RequestFailure::Status(response.status().as_u16()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| {
            RequestFailure::Provider(connection_error(
                "Custom provider returned an unreadable response.",
            ))
        })? {
            if bytes.len() + chunk.len() > MAXIMUM_RESPONSE_BYTES {
                return Err(RequestFailure::Provider(connection_error(
                    "Custom provider response exceeded the safe limit.",
                )));
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| {
            RequestFailure::Provider(connection_error(
                "Custom provider returned an invalid response.",
            ))
        })
    }

    fn configured_protocol(
        &self,
        base_url: &str,
    ) -> Result<Option<RequestProtocol>, AiProviderError> {
        let mut state = self.protocol.lock().map_err(unavailable_error)?;
        if state.base_url != base_url {
            state.base_url = base_url.to_owned();
            state.resolved = None;
        }
        Ok(state.resolved)
    }

    fn remember_protocol(
        &self,
        base_url: &str,
        protocol: RequestProtocol,
    ) -> Result<(), AiProviderError> {
        let mut state = self.protocol.lock().map_err(unavailable_error)?;
        state.base_url = base_url.to_owned();
        state.resolved = Some(protocol);
        Ok(())
    }

    async fn send_chat_completions(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: &AiRequest,
    ) -> Result<AiResponse, RequestFailure> {
        let model = required_model(configuration.model).map_err(RequestFailure::Provider)?;
        let value = self
            .request_json(
                configuration,
                Method::POST,
                "/chat/completions",
                Some(json!({
                    "model": model,
                    "messages": [{ "role": "user", "content": request.prompt }],
                    "max_tokens": MAXIMUM_OUTPUT_TOKENS
                })),
            )
            .await?;
        parse_chat_response(value).map_err(RequestFailure::Provider)
    }

    async fn send_responses(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: &AiRequest,
    ) -> Result<AiResponse, RequestFailure> {
        let model = required_model(configuration.model).map_err(RequestFailure::Provider)?;
        let value = self
            .request_json(
                configuration,
                Method::POST,
                "/responses",
                Some(json!({
                    "model": model,
                    "input": request.prompt,
                    "max_output_tokens": MAXIMUM_OUTPUT_TOKENS
                })),
            )
            .await?;
        parse_responses_response(value).map_err(RequestFailure::Provider)
    }

    async fn list_models_with_support(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<(Vec<AiModel>, bool), AiProviderError> {
        match self
            .request_json(configuration, Method::GET, "/models", None)
            .await
        {
            Ok(value) => parse_models(value).map(|models| (models, true)),
            Err(error) if error.is_unsupported_endpoint() => Ok((Vec::new(), false)),
            Err(error) => Err(error.into_provider_error()),
        }
    }
}

#[async_trait]
impl AiProvider for CustomProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Custom
    }

    fn display_name(&self) -> &'static str {
        "Custom provider"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let base_url = normalize_base_url(configuration.base_url)?;
        match self.configured_protocol(&base_url)? {
            Some(RequestProtocol::ChatCompletions) => self
                .send_chat_completions(configuration, &request)
                .await
                .map_err(RequestFailure::into_provider_error),
            Some(RequestProtocol::Responses) => self
                .send_responses(configuration, &request)
                .await
                .map_err(RequestFailure::into_provider_error),
            None => match self.send_chat_completions(configuration, &request).await {
                Ok(response) => {
                    self.remember_protocol(&base_url, RequestProtocol::ChatCompletions)?;
                    Ok(response)
                }
                Err(error) if error.is_unsupported_endpoint() => {
                    let response = self
                        .send_responses(configuration, &request)
                        .await
                        .map_err(RequestFailure::into_provider_error)?;
                    self.remember_protocol(&base_url, RequestProtocol::Responses)?;
                    Ok(response)
                }
                Err(error) => Err(error.into_provider_error()),
            },
        }
    }

    async fn list_models(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        self.list_models_with_support(configuration)
            .await
            .map(|(models, _supported)| models)
    }

    async fn test_connection(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<String, AiProviderError> {
        let (_models, supported) = self.list_models_with_support(configuration).await?;
        Ok(if supported {
            "Connection successful."
        } else {
            "Connection successful. Models endpoint unavailable."
        }
        .to_owned())
    }
}

fn normalize_base_url(value: &str) -> Result<String, AiProviderError> {
    let normalized = value.trim();
    let url = Url::parse(normalized).map_err(|_| configuration_error())?;
    let host = url.host_str().ok_or_else(configuration_error)?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || (url.scheme() == "http" && !is_local_or_private_host(host))
    {
        return Err(configuration_error());
    }
    let mut result = format!("{}://{}", url.scheme(), url.host().unwrap());
    if let Some(port) = url.port() {
        result.push(':');
        result.push_str(&port.to_string());
    }
    let path = url.path().trim_end_matches('/');
    if !path.is_empty() {
        result.push_str(path);
    }
    Ok(result)
}

fn is_local_or_private_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => address.is_loopback() || address.is_private(),
        Ok(IpAddr::V6(address)) => address.is_loopback() || address.is_unique_local(),
        Err(_) => false,
    }
}

fn required_model(model: &str) -> Result<&str, AiProviderError> {
    let model = model.trim();
    if model.is_empty() {
        Err(AiProviderError::new(
            AiProviderId::Custom,
            AiProviderErrorCode::Configuration,
            "Custom provider requires a model.",
        ))
    } else {
        Ok(model)
    }
}

fn parse_chat_response(value: Value) -> Result<AiResponse, AiProviderError> {
    let choice = value["choices"]
        .as_array()
        .and_then(|choices| choices.first());
    let content = choice
        .and_then(|choice| choice["message"]["content"].as_str())
        .unwrap_or_default()
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    response_from_content(
        content,
        choice.and_then(|choice| choice["finish_reason"].as_str()) == Some("length"),
        value.get("usage").map(|usage| AiUsage {
            input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0),
            output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0),
        }),
    )
}

fn parse_responses_response(value: Value) -> Result<AiResponse, AiProviderError> {
    let content = value["output_text"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| {
            value["output"].as_array().and_then(|output| {
                output.iter().find_map(|item| {
                    item["content"].as_array().and_then(|content| {
                        content.iter().find_map(|part| {
                            (part["type"].as_str() == Some("output_text"))
                                .then(|| part["text"].as_str())
                                .flatten()
                                .map(str::to_owned)
                        })
                    })
                })
            })
        })
        .unwrap_or_default()
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    let status = value["status"].as_str();
    let usage = value.get("usage").map(|usage| AiUsage {
        input_tokens: usage["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["output_tokens"].as_u64().unwrap_or(0),
    });
    let mut response = response_from_content(content, status == Some("incomplete"), usage)?;
    if status == Some("cancelled") {
        response.finish_reason = AiFinishReason::Cancelled;
    }
    Ok(response)
}

fn response_from_content(
    content: String,
    truncated: bool,
    usage: Option<AiUsage>,
) -> Result<AiResponse, AiProviderError> {
    if content.is_empty() {
        return Err(AiProviderError::new(
            AiProviderId::Custom,
            AiProviderErrorCode::EmptyResponse,
            "Custom provider returned an empty response.",
        ));
    }
    Ok(AiResponse {
        provider_id: AiProviderId::Custom,
        content,
        finish_reason: if truncated {
            AiFinishReason::Length
        } else {
            AiFinishReason::Stop
        },
        usage,
    })
}

fn parse_models(value: Value) -> Result<Vec<AiModel>, AiProviderError> {
    let models = value["data"]
        .as_array()
        .ok_or_else(|| connection_error("Custom provider returned an invalid model list."))?;
    Ok(normalize_models(
        models
            .iter()
            .filter_map(|model| model["id"].as_str())
            .map(|id| AiModel {
                id: id.to_owned(),
                display_name: None,
            })
            .collect(),
    ))
}

fn configuration_error() -> AiProviderError {
    AiProviderError::new(
        AiProviderId::Custom,
        AiProviderErrorCode::Configuration,
        "Custom provider requires a valid base URL.",
    )
}

fn connection_error(message: &'static str) -> AiProviderError {
    AiProviderError::new(
        AiProviderId::Custom,
        AiProviderErrorCode::Connection,
        message,
    )
}

fn unavailable_error<T>(_error: PoisonError<T>) -> AiProviderError {
    connection_error("Custom provider state is unavailable.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_base_url_policy_preserves_security_contract() {
        assert_eq!(
            normalize_base_url(" https://openrouter.ai/api/v1/ ").unwrap(),
            "https://openrouter.ai/api/v1"
        );
        assert_eq!(
            normalize_base_url("http://192.168.1.25:8000/v1/").unwrap(),
            "http://192.168.1.25:8000/v1"
        );
        for value in [
            "",
            "ftp://localhost/v1",
            "http://example.com/v1",
            "http://169.254.169.254/latest",
            "http://user:password@localhost/v1",
            "http://localhost/v1?token=secret",
        ] {
            assert!(normalize_base_url(value).is_err(), "{value}");
        }
    }

    #[test]
    fn both_final_response_protocols_preserve_usage() {
        let chat = parse_chat_response(json!({
            "choices": [{
                "message": { "content": "chat response" },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 4, "completion_tokens": 2 }
        }))
        .unwrap();
        let responses = parse_responses_response(json!({
            "status": "incomplete",
            "output_text": "responses response",
            "usage": { "input_tokens": 3, "output_tokens": 2 }
        }))
        .unwrap();
        assert_eq!(chat.content, "chat response");
        assert_eq!(
            chat.usage,
            Some(AiUsage {
                input_tokens: 4,
                output_tokens: 2
            })
        );
        assert_eq!(responses.content, "responses response");
        assert_eq!(responses.finish_reason, AiFinishReason::Length);
    }
}
