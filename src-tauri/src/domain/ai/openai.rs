use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

use super::{
    provider::{
        create_http_diagnostics, normalize_models, AiFinishReason, AiModel, AiUsage,
        MAXIMUM_OUTPUT_TOKENS, MAXIMUM_RESPONSE_CHARS,
    },
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiProviderId,
    AiRequest, AiResponse,
};

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(crate) struct OpenAiProvider {
    client: Client,
}

impl OpenAiProvider {
    pub(crate) fn new() -> Result<Self, AiProviderError> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| connection_error("OpenAI client initialization failed."))?;
        Ok(Self { client })
    }

    async fn checked_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<Value, AiProviderError> {
        let request_url = request
            .try_clone()
            .and_then(|request| request.build().ok())
            .map(|request| request.url().to_string())
            .unwrap_or_else(|| OPENAI_BASE_URL.to_owned());
        let response = request
            .send()
            .await
            .map_err(|_| connection_error("OpenAI connection failed."))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| connection_error("OpenAI returned an unreadable response."))?;
        if bytes.len() > 4 * 1_024 * 1_024 {
            return Err(connection_error("OpenAI response exceeded the safe limit."));
        }
        if !status.is_success() {
            return Err(status_error(status, &request_url, &bytes));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| connection_error("OpenAI returned an invalid response."))
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Openai
    }

    fn display_name(&self) -> &'static str {
        "OpenAI"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let api_key = required(configuration.api_key, "OpenAI requires an API key.")?;
        let model = required_text(configuration.model, "OpenAI requires a model.")?;
        let value = self
            .checked_json(
                self.client
                    .post(format!("{OPENAI_BASE_URL}/responses"))
                    .bearer_auth(api_key)
                    .json(&json!({
                        "model": model,
                        "input": request.prompt,
                        "max_output_tokens": MAXIMUM_OUTPUT_TOKENS
                    })),
            )
            .await?;
        parse_response(value)
    }

    async fn list_models(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        let api_key = required(configuration.api_key, "OpenAI requires an API key.")?;
        let value = self
            .checked_json(
                self.client
                    .get(format!("{OPENAI_BASE_URL}/models"))
                    .bearer_auth(api_key),
            )
            .await?;
        let models = value["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|model| model["id"].as_str())
            .filter(|id| is_text_model(id))
            .map(|id| AiModel {
                id: id.to_owned(),
                display_name: None,
            })
            .collect();
        Ok(normalize_models(models))
    }
}

fn required<'a>(value: Option<&'a str>, message: &'static str) -> Result<&'a str, AiProviderError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AiProviderError::new(
                AiProviderId::Openai,
                AiProviderErrorCode::Configuration,
                message,
            )
        })
}

fn required_text<'a>(value: &'a str, message: &'static str) -> Result<&'a str, AiProviderError> {
    required(Some(value), message)
}

fn parse_response(value: Value) -> Result<AiResponse, AiProviderError> {
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
        .unwrap_or_default();
    let content = content
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    if content.is_empty() {
        return Err(AiProviderError::new(
            AiProviderId::Openai,
            AiProviderErrorCode::EmptyResponse,
            "OpenAI returned an empty response.",
        ));
    }

    let status = value["status"].as_str();
    let finish_reason = match status {
        Some("cancelled") => AiFinishReason::Cancelled,
        Some("incomplete") => AiFinishReason::Length,
        _ => AiFinishReason::Stop,
    };
    let usage = value.get("usage").map(|usage| AiUsage {
        input_tokens: usage["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["output_tokens"].as_u64().unwrap_or(0),
    });
    Ok(AiResponse {
        provider_id: AiProviderId::Openai,
        content,
        finish_reason,
        usage,
    })
}

fn is_text_model(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    let base = id.strip_prefix("ft:").unwrap_or(&id);
    let text_family = base.starts_with("gpt-")
        || ["o1", "o3", "o4"]
            .iter()
            .any(|prefix| base == *prefix || base.starts_with(&format!("{prefix}-")));
    text_family
        && ![
            "audio",
            "image",
            "moderation",
            "realtime",
            "speech",
            "transcribe",
            "tts",
            "whisper",
        ]
        .iter()
        .any(|marker| base.contains(marker))
}

fn connection_error(message: &'static str) -> AiProviderError {
    AiProviderError::new(
        AiProviderId::Openai,
        AiProviderErrorCode::Connection,
        message,
    )
}

fn status_error(status: StatusCode, request_url: &str, bytes: &[u8]) -> AiProviderError {
    let message = match status.as_u16() {
        400 => "OpenAI rejected the configured request.",
        401 => "OpenAI did not accept the configured API key.",
        403 => "OpenAI denied access for this API key.",
        404 => "OpenAI could not find the configured model or endpoint.",
        429 => "OpenAI rate limit or quota reached.",
        500..=599 => "OpenAI is temporarily unavailable.",
        _ => "OpenAI request failed.",
    };

    connection_error(message).with_diagnostics(create_http_diagnostics(
        request_url,
        status,
        bytes,
        message,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_payload_maps_content_finish_reason_and_usage() {
        let response = parse_response(json!({
            "status": "incomplete",
            "output": [{
                "content": [{ "type": "output_text", "text": " hello " }]
            }],
            "usage": { "input_tokens": 4, "output_tokens": 2 }
        }))
        .unwrap();

        assert_eq!(response.provider_id, AiProviderId::Openai);
        assert_eq!(response.content, "hello");
        assert_eq!(response.finish_reason, AiFinishReason::Length);
        assert_eq!(
            response.usage,
            Some(AiUsage {
                input_tokens: 4,
                output_tokens: 2
            })
        );
    }

    #[test]
    fn openai_model_filter_excludes_non_text_families() {
        assert!(is_text_model("gpt-4.1"));
        assert!(is_text_model("o3-mini"));
        assert!(!is_text_model("gpt-4o-realtime-preview"));
        assert!(!is_text_model("text-embedding-3-small"));
    }

    #[test]
    fn openai_quota_errors_retain_bounded_safe_diagnostics() {
        let error = status_error(
            StatusCode::TOO_MANY_REQUESTS,
            "https://api.openai.com/v1/responses",
            br#"{"error":{"message":"You exceeded your current quota.","type":"insufficient_quota","code":"insufficient_quota","api_key":"secret"}}"#,
        );
        let diagnostics = error.diagnostics().unwrap();

        assert_eq!(error.message(), "OpenAI rate limit or quota reached.");
        assert_eq!(diagnostics.http_status_code, Some(429));
        assert_eq!(
            diagnostics.error_code.as_deref(),
            Some("insufficient_quota")
        );
        assert_eq!(
            diagnostics.error_message,
            "You exceeded your current quota."
        );
        assert!(!diagnostics.response_body.contains("secret"));
    }
}
