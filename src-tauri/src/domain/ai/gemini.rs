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

const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug)]
pub(crate) struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    pub(crate) fn new() -> Result<Self, AiProviderError> {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|client| Self { client })
            .map_err(|_| connection_error("Gemini client initialization failed."))
    }

    async fn checked_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<Value, AiProviderError> {
        let response = request
            .send()
            .await
            .map_err(|_| connection_error("Gemini connection failed."))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| connection_error("Gemini returned an unreadable response."))?;
        if bytes.len() > 4 * 1_024 * 1_024 {
            return Err(connection_error("Gemini response exceeded the safe limit."));
        }
        if !status.is_success() {
            return Err(connection_error(match status.as_u16() {
                401 | 403 => "Gemini authentication failed.",
                429 => "Gemini rate limit reached.",
                _ => "Gemini request failed.",
            }));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| connection_error("Gemini returned an invalid response."))
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn id(&self) -> AiProviderId {
        AiProviderId::Gemini
    }

    fn display_name(&self) -> &'static str {
        "Gemini"
    }

    async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        let api_key = required(configuration.api_key, "Gemini requires an API key.")?;
        let model = required(Some(configuration.model), "Gemini requires a model.")?;
        let value = self
            .checked_json(
                self.client
                    .post(format!("{GEMINI_BASE_URL}/models/{model}:generateContent"))
                    .query(&[("key", api_key)])
                    .json(&json!({
                        "contents": [{ "role": "user", "parts": [{ "text": request.prompt }] }],
                        "generationConfig": { "maxOutputTokens": MAXIMUM_OUTPUT_TOKENS }
                    })),
            )
            .await?;
        parse_response(value)
    }

    async fn list_models(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        let api_key = required(configuration.api_key, "Gemini requires an API key.")?;
        let value = self
            .checked_json(
                self.client
                    .get(format!("{GEMINI_BASE_URL}/models"))
                    .query(&[("key", api_key), ("pageSize", "1000")]),
            )
            .await?;
        let models = value["models"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|model| {
                model["supportedGenerationMethods"]
                    .as_array()
                    .is_some_and(|methods| {
                        methods
                            .iter()
                            .any(|method| method.as_str() == Some("generateContent"))
                    })
            })
            .filter_map(|model| {
                let id = model["name"].as_str()?.strip_prefix("models/")?;
                Some(AiModel {
                    id: id.to_owned(),
                    display_name: model["displayName"].as_str().map(str::to_owned),
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
                AiProviderId::Gemini,
                AiProviderErrorCode::Configuration,
                message,
            )
        })
}

fn parse_response(value: Value) -> Result<AiResponse, AiProviderError> {
    let candidate = value["candidates"]
        .as_array()
        .and_then(|values| values.first());
    let content = candidate
        .and_then(|candidate| candidate["content"]["parts"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|part| part["text"].as_str())
        .collect::<String>();
    let content = content
        .trim()
        .chars()
        .take(MAXIMUM_RESPONSE_CHARS)
        .collect::<String>();
    if content.is_empty() {
        return Err(AiProviderError::new(
            AiProviderId::Gemini,
            AiProviderErrorCode::EmptyResponse,
            "Gemini returned an empty response.",
        ));
    }
    let finish_reason = if candidate.and_then(|candidate| candidate["finishReason"].as_str())
        == Some("MAX_TOKENS")
    {
        AiFinishReason::Length
    } else {
        AiFinishReason::Stop
    };
    let usage = value.get("usageMetadata").map(|usage| AiUsage {
        input_tokens: usage["promptTokenCount"].as_u64().unwrap_or(0),
        output_tokens: usage["candidatesTokenCount"].as_u64().unwrap_or(0),
    });
    Ok(AiResponse {
        provider_id: AiProviderId::Gemini,
        content,
        finish_reason,
        usage,
    })
}

fn connection_error(message: &'static str) -> AiProviderError {
    AiProviderError::new(
        AiProviderId::Gemini,
        AiProviderErrorCode::Connection,
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_mapping_preserves_text_usage_and_length_finish() {
        let response = parse_response(json!({
            "candidates": [{
                "content": { "parts": [{ "text": "Hello" }, { "text": "!" }] },
                "finishReason": "MAX_TOKENS"
            }],
            "usageMetadata": {
                "promptTokenCount": 5,
                "candidatesTokenCount": 3
            }
        }))
        .unwrap();
        assert_eq!(response.content, "Hello!");
        assert_eq!(response.finish_reason, AiFinishReason::Length);
        assert_eq!(
            response.usage,
            Some(AiUsage {
                input_tokens: 5,
                output_tokens: 3
            })
        );
    }
}
