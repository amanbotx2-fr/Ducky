use serde::Serialize;
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::{
        ai::{
            AiConversationRequest, AiExecutionError, AiProviderConfiguration, AiProviderId,
            AiRequest, AiResponse, AiRuntime, AssistantActionError, AssistantActionProcessor,
        },
        settings::SettingsState,
    },
};

const AI_UNAVAILABLE_MESSAGE: &str = "Chat is unavailable right now.";
const PROVIDER_FAILED_MESSAGE: &str = "Couldn't reach the provider.";

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
    actions: State<'_, AssistantActionProcessor>,
    settings: State<'_, SettingsState>,
    request: AiConversationRequest,
) -> Result<AiAskResult, AiCommandError> {
    authorize(&window, authorization::ASK_AI)?;
    let request = request
        .validate()
        .map_err(|_| AiCommandError::InvalidRequest)?;
    let prompt = request
        .to_provider_prompt()
        .map_err(|_| AiCommandError::InvalidRequest)?;
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
        Ok(response) => match actions.process(response) {
            Ok(response) => Ok(AiAskResult::success(response)),
            Err(error) => {
                eprintln!("[ai-action] action_rejected: {}", action_error_code(error));
                Ok(AiAskResult::failure("I couldn't complete that action."))
            }
        },
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

fn action_error_code(error: AssistantActionError) -> &'static str {
    match error {
        AssistantActionError::InvalidRequest => "invalid_request",
        AssistantActionError::InvalidAction => "invalid_action",
        AssistantActionError::InvalidPayload => "invalid_payload",
        AssistantActionError::UnknownAction => "unknown_action",
        AssistantActionError::ExecutionFailed => "service_rejected",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::domain::ai::{AiFinishReason, AiProviderId};

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
    fn action_failures_are_logged_by_bounded_codes_only() {
        assert_eq!(
            action_error_code(AssistantActionError::InvalidPayload),
            "invalid_payload"
        );
        assert_eq!(
            action_error_code(AssistantActionError::ExecutionFailed),
            "service_rejected"
        );
    }
}
