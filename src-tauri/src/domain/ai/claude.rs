use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::{
    provider::{
        normalize_models, AiFinishReason, AiModel, AiUsage, MAXIMUM_OUTPUT_TOKENS,
        MAXIMUM_RESPONSE_CHARS,
    },
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiProviderId,
    AiRequest, AiResponse,
};

const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug)]
pub(crate) struct ClaudeProvider {
    client: Client,
}

impl ClaudeProvider {
    pub(crate) fn new() -> Result<Self, AiProviderError> {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|client| Self { client })
            .map_err(|_| connection_error("Claude client initialization failed."))
    }

    async fn checked_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<Value, AiProviderError> {
        let response = request
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send()
            .await
            .map_err(|_| connection_error("Claude connection failed."))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| connection_error("Claude returned an unreadable response."))?;
        if bytes.len() > 4 * 1_024 * 1_024 {
            return Err(connection_error("Claude response exceeded the safe limit."));
        }
        if !status.is_success() {
            return Err(connection_error(match status.as_u16() {
                401 | 403 => "Claude authentication failed.",
                429 => "Claude rate limit reached.",
                _ => "Claude request failed.",
            }));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| connection_error("Claude returned an invalid response."))
    }

    fn authenticated(
        &self,
        request: reqwest::RequestBuilder,
        api_key: &str,
    ) -> reqwest::RequestBuilder {
        request.header("x-api-key", api_key)
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Claude
    }

    fn display_name(&self) -> &'static str {
        "Claude"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let api_key = required(configuration.api_key, "Claude requires an API key.")?;
        let model = required(Some(configuration.model), "Claude requires a model.")?;
        let value = self
            .checked_json(
                self.authenticated(
                    self.client
                        .post(format!("{ANTHROPIC_BASE_URL}/messages"))
                        .json(&json!({
                            "model": model,
                            "max_tokens": MAXIMUM_OUTPUT_TOKENS,
                            "messages": [{ "role": "user", "content": request.prompt }]
                        })),
                    api_key,
                ),
            )
            .await?;
        parse_response(value)
    }

    async fn list_models(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        let api_key = required(configuration.api_key, "Claude requires an API key.")?;
        let value = self
            .checked_json(
                self.authenticated(
                    self.client
                        .get(format!("{ANTHROPIC_BASE_URL}/models"))
                        .query(&[("limit", "1000")]),
                    api_key,
                ),
            )
            .await?;
        let models = value["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|model| {
                Some(AiModel {
                    id: model["id"].as_str()?.to_owned(),
                    display_name: model["display_name"].as_str().map(str::to_owned),
                })
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
                AiProviderId::Claude,
                AiProviderErrorCode::Configuration,
                message,
            )
        })
}

fn parse_response(value: Value) -> Result<AiResponse, AiProviderError> {
    let content = value["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|block| block["type"].as_str() == Some("text"))
        .filter_map(|block| block["text"].as_str())
        .collect::<String>();
    let content = content
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    if content.is_empty() {
        return Err(AiProviderError::new(
            AiProviderId::Claude,
            AiProviderErrorCode::EmptyResponse,
            "Claude returned an empty response.",
        ));
    }
    let finish_reason = if value["stop_reason"].as_str() == Some("max_tokens") {
        AiFinishReason::Length
    } else {
        AiFinishReason::Stop
    };
    let usage = value.get("usage").map(|usage| AiUsage {
        input_tokens: usage["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["output_tokens"].as_u64().unwrap_or(0),
    });
    Ok(AiResponse {
        provider_id: AiProviderId::Claude,
        content,
        finish_reason,
        usage,
    })
}

fn connection_error(message: &'static str) -> AiProviderError {
    AiProviderError::new(
        AiProviderId::Claude,
        AiProviderErrorCode::Connection,
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_response_is_aggregated_before_leaving_rust() {
        let response = parse_response(json!({
            "content": [
                { "type": "text", "text": "Hello" },
                { "type": "text", "text": " from Claude" }
            ],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 7, "output_tokens": 4 }
        }))
        .unwrap();
        assert_eq!(response.content, "Hello from Claude");
        assert_eq!(response.finish_reason, AiFinishReason::Stop);
        assert_eq!(
            response.usage,
            Some(AiUsage {
                input_tokens: 7,
                output_tokens: 4
            })
        );
    }
}
