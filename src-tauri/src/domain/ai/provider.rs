use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::AiProviderId;

pub(crate) const MAXIMUM_OUTPUT_TOKENS: u32 = 4_096;
pub(crate) const MAXIMUM_RESPONSE_CHARS: usize = 32_768;
pub(crate) const MAXIMUM_ERROR_CHARS: usize = 512;
pub(crate) const MAXIMUM_MODELS: usize = 1_024;

#[derive(Clone, Copy)]
pub(crate) struct AiProviderConfiguration<'a> {
    pub(crate) api_key: Option<&'a str>,
    pub(crate) base_url: &'a str,
    pub(crate) endpoint: &'a str,
    pub(crate) model: &'a str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiRequest {
    pub(crate) prompt: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AiFinishReason {
    Stop,
    Length,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiUsage {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiResponse {
    pub(crate) provider_id: AiProviderId,
    pub(crate) content: String,
    pub(crate) finish_reason: AiFinishReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) usage: Option<AiUsage>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiModel {
    pub(crate) id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) display_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiProviderHttpDiagnostics {
    pub(crate) request_url: String,
    pub(crate) http_status_code: Option<u16>,
    pub(crate) http_status_text: Option<String>,
    pub(crate) response_body: String,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AiProviderErrorCode {
    Configuration,
    Connection,
    EmptyResponse,
    UnsupportedOperation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AiProviderError {
    pub(crate) provider_id: AiProviderId,
    pub(crate) code: AiProviderErrorCode,
    message: String,
    diagnostics: Option<AiProviderHttpDiagnostics>,
}

impl AiProviderError {
    pub(crate) fn new(
        provider_id: AiProviderId,
        code: AiProviderErrorCode,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        Self {
            provider_id,
            code,
            message: message.chars().take(MAXIMUM_ERROR_CHARS).collect(),
            diagnostics: None,
        }
    }

    pub(crate) fn with_diagnostics(mut self, diagnostics: AiProviderHttpDiagnostics) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn diagnostics(&self) -> Option<&AiProviderHttpDiagnostics> {
        self.diagnostics.as_ref()
    }
}

impl std::fmt::Display for AiProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AiProviderError {}

#[async_trait]
pub(crate) trait AiProvider: std::fmt::Debug + Send + Sync {
    fn id(&self) -> AiProviderId;
    fn display_name(&self) -> &'static str;

    async fn send_message(
        &self,
        _configuration: AiProviderConfiguration<'_>,
        _request: AiRequest,
    ) -> Result<AiResponse, AiProviderError> {
        Err(AiProviderError::new(
            self.id(),
            AiProviderErrorCode::UnsupportedOperation,
            format!("{} does not support messages.", self.display_name()),
        ))
    }

    async fn list_models(
        &self,
        _configuration: AiProviderConfiguration<'_>,
    ) -> Result<Vec<AiModel>, AiProviderError> {
        Err(AiProviderError::new(
            self.id(),
            AiProviderErrorCode::UnsupportedOperation,
            format!("{} does not support model discovery.", self.display_name()),
        ))
    }

    async fn test_connection(
        &self,
        configuration: AiProviderConfiguration<'_>,
    ) -> Result<String, AiProviderError> {
        self.list_models(configuration).await?;
        Ok("Connection successful.".to_owned())
    }
}

pub(crate) fn normalize_models(mut models: Vec<AiModel>) -> Vec<AiModel> {
    models.retain(|model| !model.id.trim().is_empty());
    for model in &mut models {
        model.id = model.id.trim().chars().take(256).collect();
        model.display_name = model
            .display_name
            .take()
            .map(|name| name.trim().chars().take(256).collect())
            .filter(|name: &String| !name.is_empty());
    }
    models.sort_by(|left, right| left.id.cmp(&right.id));
    models.dedup_by(|left, right| left.id == right.id);
    models.truncate(MAXIMUM_MODELS);
    models
}

pub(crate) fn create_http_diagnostics(
    request_url: &str,
    status: StatusCode,
    bytes: &[u8],
    fallback_message: &str,
) -> AiProviderHttpDiagnostics {
    let mut value = serde_json::from_slice::<Value>(bytes).ok();

    if let Some(value) = value.as_mut() {
        redact_sensitive_values(value);
    }

    let response_body = value
        .as_ref()
        .and_then(|value| serde_json::to_string(value).ok())
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        .take(4_096)
        .collect::<String>();
    let error = value.as_ref().and_then(|value| value.get("error"));
    let error_code = error
        .and_then(|error| error["code"].as_str().or_else(|| error["type"].as_str()))
        .map(|value| value.chars().take(256).collect());
    let error_message = error
        .and_then(|error| error["message"].as_str())
        .unwrap_or(fallback_message)
        .chars()
        .take(MAXIMUM_ERROR_CHARS)
        .collect();

    AiProviderHttpDiagnostics {
        request_url: request_url.chars().take(2_048).collect(),
        http_status_code: Some(status.as_u16()),
        http_status_text: status.canonical_reason().map(str::to_owned),
        response_body,
        error_code,
        error_message,
    }
}

fn redact_sensitive_values(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if ["authorization", "api_key", "apikey", "secret", "token"]
                    .iter()
                    .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
                {
                    *value = Value::String("[REDACTED]".to_owned());
                } else {
                    redact_sensitive_values(value);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_sensitive_values(value);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use super::*;

    #[test]
    fn diagnostics_preserve_safe_provider_detail_and_redact_secrets() {
        let diagnostics = create_http_diagnostics(
            "https://api.example.test/v1/messages",
            StatusCode::PAYMENT_REQUIRED,
            br#"{"error":{"type":"billing_error","message":"Credit balance is too low.","token":"secret"}}"#,
            "Provider request failed.",
        );

        assert_eq!(diagnostics.http_status_code, Some(402));
        assert_eq!(diagnostics.error_code.as_deref(), Some("billing_error"));
        assert_eq!(diagnostics.error_message, "Credit balance is too low.");
        assert!(diagnostics.response_body.contains("[REDACTED]"));
        assert!(!diagnostics.response_body.contains("secret"));
    }
}
