use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

use super::{
    provider::{
        normalize_models, AiFinishReason, AiModel, AiUsage, MAXIMUM_MODELS, MAXIMUM_OUTPUT_TOKENS,
        MAXIMUM_RESPONSE_CHARS,
    },
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiProviderId,
    AiRequest, AiResponse,
};

const XAI_BASE_URL: &str = "https://api.x.ai/v1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(crate) struct GrokProvider {
    client: Client,
}

impl GrokProvider {
    pub(crate) fn new() -> Result<Self, AiProviderError> {
        Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|client| Self { client })
            .map_err(|_| connection_error("Grok client initialization failed."))
    }

    async fn checked_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<Value, AiProviderError> {
        let response = request
            .send()
            .await
            .map_err(|_| connection_error("Grok connection failed."))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| connection_error("Grok returned an unreadable response."))?;
        if bytes.len() > 4 * 1_024 * 1_024 {
            return Err(connection_error("Grok response exceeded the safe limit."));
        }
        if !status.is_success() {
            return Err(status_error(status));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| connection_error("Grok returned an invalid response."))
    }
}

#[async_trait]
impl AiProvider for GrokProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Grok
    }

    fn display_name(&self) -> &'static str {
        "Grok"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let api_key = required(configuration.api_key, "Grok requires an API key.")?;
        let model = required(Some(configuration.model), "Grok requires a model.")?;
        let value = self
            .checked_json(
                self.client
                    .post(format!("{XAI_BASE_URL}/responses"))
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
        let api_key = required(configuration.api_key, "Grok requires an API key.")?;
        let value = self
            .checked_json(
                self.client
                    .get(format!("{XAI_BASE_URL}/language-models"))
                    .bearer_auth(api_key),
            )
            .await?;
        Ok(parse_models(value))
    }
}

fn required<'a>(value: Option<&'a str>, message: &'static str) -> Result<&'a str, AiProviderError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AiProviderError::new(
                AiProviderId::Grok,
                AiProviderErrorCode::Configuration,
                message,
            )
        })
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
            AiProviderId::Grok,
            AiProviderErrorCode::EmptyResponse,
            "Grok returned an empty response.",
        ));
    }
    let finish_reason = match value["status"].as_str() {
        Some("cancelled") => AiFinishReason::Cancelled,
        Some("incomplete") => AiFinishReason::Length,
        _ => AiFinishReason::Stop,
    };
    let usage = value.get("usage").map(|usage| AiUsage {
        input_tokens: usage["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["output_tokens"].as_u64().unwrap_or(0),
    });
    Ok(AiResponse {
        provider_id: AiProviderId::Grok,
        content,
        finish_reason,
        usage,
    })
}

fn parse_models(value: Value) -> Vec<AiModel> {
    let mut models = Vec::new();
    for model in value["models"]
        .as_array()
        .into_iter()
        .flatten()
        .take(MAXIMUM_MODELS * 2)
    {
        if model["output_modalities"]
            .as_array()
            .is_some_and(|modalities| {
                !modalities
                    .iter()
                    .any(|value| value.as_str() == Some("text"))
            })
        {
            continue;
        }
        let Some(id) = model["id"].as_str() else {
            continue;
        };
        models.push(AiModel {
            id: id.to_owned(),
            display_name: None,
        });
        for alias in model["aliases"].as_array().into_iter().flatten() {
            let Some(alias) = alias.as_str() else {
                continue;
            };
            models.push(AiModel {
                id: alias.to_owned(),
                display_name: Some(format!("{alias} ({id})")),
            });
        }
    }
    normalize_models(models)
}

fn connection_error(message: &'static str) -> AiProviderError {
    AiProviderError::new(AiProviderId::Grok, AiProviderErrorCode::Connection, message)
}

fn status_error(status: StatusCode) -> AiProviderError {
    connection_error(match status.as_u16() {
        401 | 403 => "Grok authentication failed.",
        429 => "Grok rate limit reached.",
        _ => "Grok request failed.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_payload_preserves_final_response_metadata() {
        let response = parse_response(json!({
            "status": "completed",
            "output_text": "Hello from Grok",
            "usage": { "input_tokens": 5, "output_tokens": 3 }
        }))
        .unwrap();
        assert_eq!(response.provider_id, AiProviderId::Grok);
        assert_eq!(response.content, "Hello from Grok");
        assert_eq!(response.finish_reason, AiFinishReason::Stop);
        assert_eq!(
            response.usage,
            Some(AiUsage {
                input_tokens: 5,
                output_tokens: 3
            })
        );
    }

    #[test]
    fn language_models_preserve_text_models_and_aliases() {
        let models = parse_models(json!({
            "models": [
                {
                    "id": "grok-4",
                    "aliases": ["grok-latest"],
                    "output_modalities": ["text"]
                },
                {
                    "id": "grok-image",
                    "output_modalities": ["image"]
                }
            ]
        }));
        assert_eq!(
            models,
            [
                AiModel {
                    id: "grok-4".to_owned(),
                    display_name: None
                },
                AiModel {
                    id: "grok-latest".to_owned(),
                    display_name: Some("grok-latest (grok-4)".to_owned())
                }
            ]
        );
    }
}
