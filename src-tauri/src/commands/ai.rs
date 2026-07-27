use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::{
        ai::{
            AiExecutionError, AiProviderConfiguration, AiProviderId, AiRequest, AiResponse,
            AiRuntime,
        },
        settings::SettingsState,
    },
};

const MAXIMUM_PROMPT_CHARACTERS: usize = 4_096;
const MAXIMUM_CONTEXT_MESSAGES: usize = 16;
const MAXIMUM_CONTEXT_CHARACTERS: usize = 24_000;
const MAXIMUM_CONTEXT_MESSAGE_CHARACTERS: usize = 12_000;
const AI_UNAVAILABLE_MESSAGE: &str = "Chat is unavailable right now.";
const PROVIDER_FAILED_MESSAGE: &str = "Couldn't reach the provider.";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AiConversationRole {
    User,
    Assistant,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiConversationContextMessage {
    role: AiConversationRole,
    content: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiConversationRequest {
    prompt: String,
    history: Vec<AiConversationContextMessage>,
}

impl AiConversationRequest {
    fn validate_and_build_prompt(self) -> Result<String, AiCommandError> {
        let prompt = normalize_text(self.prompt, MAXIMUM_PROMPT_CHARACTERS)?;
        if self.history.len() > MAXIMUM_CONTEXT_MESSAGES {
            return Err(AiCommandError::InvalidRequest);
        }

        let mut total_characters = 0;
        let mut history = Vec::with_capacity(self.history.len());
        for message in self.history {
            let content = normalize_text(message.content, MAXIMUM_CONTEXT_MESSAGE_CHARACTERS)?;
            total_characters += content.chars().count();
            if total_characters > MAXIMUM_CONTEXT_CHARACTERS {
                return Err(AiCommandError::InvalidRequest);
            }
            history.push(AiConversationContextMessage {
                role: message.role,
                content,
            });
        }

        if history.is_empty() {
            return Ok(prompt);
        }

        let serialized =
            serde_json::to_string(&history).map_err(|_| AiCommandError::InvalidRequest)?;
        Ok(format!(
            "Prior conversation, oldest to newest:\n\
             <conversation_history_json>\n{serialized}\n\
             </conversation_history_json>\n\n\
             <user_request>\n{prompt}\n</user_request>"
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiAskResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<AiResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl AiAskResult {
    fn success(response: AiResponse) -> Self {
        Self {
            ok: true,
            response: Some(response),
            message: None,
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            response: None,
            message: Some(message.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AiCommandError {
    UnauthorizedWindow,
    InvalidRequest,
    SettingsUnavailable,
}

#[tauri::command]
pub(crate) async fn ask_ai<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, AiRuntime>,
    settings: State<'_, SettingsState>,
    request: AiConversationRequest,
) -> Result<AiAskResult, AiCommandError> {
    authorize(&window, authorization::ASK_AI)?;
    let prompt = request.validate_and_build_prompt()?;
    let settings = settings
        .snapshot()
        .map_err(|_| AiCommandError::SettingsUnavailable)?;

    if !settings.ai.enabled {
        return Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE));
    }

    let Some(provider_id) = AiProviderId::parse(&settings.ai.provider) else {
        return Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE));
    };
    let credential = match runtime.load_provider_credential(provider_id) {
        Ok(credential) => credential,
        Err(_) => return Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE)),
    };
    let configuration = AiProviderConfiguration {
        api_key: credential.as_ref().map(|value| value.as_str()),
        base_url: &settings.ai.base_url,
        endpoint: &settings.ai.endpoint,
        model: &settings.ai.model,
    };

    match runtime
        .send_message(configuration, AiRequest { prompt })
        .await
    {
        Ok(response) => Ok(AiAskResult::success(response)),
        Err(AiExecutionError::Provider(_)) => Ok(AiAskResult::failure(PROVIDER_FAILED_MESSAGE)),
        Err(_) => Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE)),
    }
}

fn authorize<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), AiCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| AiCommandError::UnauthorizedWindow)
}

fn normalize_text(value: String, maximum: usize) -> Result<String, AiCommandError> {
    if value.chars().count() > maximum {
        return Err(AiCommandError::InvalidRequest);
    }
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        Err(AiCommandError::InvalidRequest)
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::domain::ai::{AiFinishReason, AiProviderId};

    #[test]
    fn conversation_validation_preserves_bounded_context_in_one_prompt() {
        let prompt = AiConversationRequest {
            prompt: " Continue ".to_owned(),
            history: vec![
                AiConversationContextMessage {
                    role: AiConversationRole::User,
                    content: " Hello ".to_owned(),
                },
                AiConversationContextMessage {
                    role: AiConversationRole::Assistant,
                    content: " Hi ".to_owned(),
                },
            ],
        }
        .validate_and_build_prompt()
        .unwrap();

        assert!(prompt.contains(r#""role":"user","content":"Hello""#));
        assert!(prompt.contains(r#""role":"assistant","content":"Hi""#));
        assert!(prompt.ends_with("<user_request>\nContinue\n</user_request>"));
    }

    #[test]
    fn result_serialization_matches_the_existing_whole_response_contract() {
        let result = AiAskResult::success(AiResponse {
            provider_id: AiProviderId::Openai,
            content: "Complete".to_owned(),
            finish_reason: AiFinishReason::Stop,
            usage: None,
        });

        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({
                "ok": true,
                "response": {
                    "providerId": "openai",
                    "content": "Complete",
                    "finishReason": "stop"
                }
            })
        );
        assert_eq!(
            serde_json::to_value(AiAskResult::failure("Unavailable")).unwrap(),
            json!({ "ok": false, "message": "Unavailable" })
        );
    }

    #[test]
    fn invalid_or_excessive_conversation_input_is_rejected() {
        assert_eq!(
            AiConversationRequest {
                prompt: " ".to_owned(),
                history: vec![],
            }
            .validate_and_build_prompt(),
            Err(AiCommandError::InvalidRequest)
        );
        assert_eq!(
            AiConversationRequest {
                prompt: "hello".to_owned(),
                history: (0..=MAXIMUM_CONTEXT_MESSAGES)
                    .map(|_| AiConversationContextMessage {
                        role: AiConversationRole::User,
                        content: "context".to_owned(),
                    })
                    .collect(),
            }
            .validate_and_build_prompt(),
            Err(AiCommandError::InvalidRequest)
        );
    }
}
