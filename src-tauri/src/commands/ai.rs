use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};
use zeroize::Zeroizing;

use crate::{
    authorization,
    domain::{
        ai::{
            AiCancellationReason, AiConversationRequest, AiExecutionError, AiOperation,
            AiProviderConfiguration, AiProviderHttpDiagnostics, AiProviderId, AiRendererRole,
            AiRequest, AiRequestManager, AiResponse, AiRuntime, AssistantActionError,
            AssistantActionProcessor,
        },
        settings::{
            AiSettingsPatch, PreferencesSettings, SettingsMutationError, SettingsState,
            StoredAiSettings,
        },
    },
    infrastructure::credentials::{CredentialId, CredentialStore, CredentialStoreError},
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiModelListResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    models: Option<Vec<crate::domain::ai::AiModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiConnectionTestResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<AiProviderHttpDiagnostics>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiConfigurationUpdate {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    api_key: Option<String>,
}

impl AiConfigurationUpdate {
    fn settings_patch(&self) -> AiSettingsPatch {
        AiSettingsPatch {
            enabled: self.enabled,
            provider: self.provider.clone(),
            model: self.model.clone(),
            endpoint: self.endpoint.clone(),
            base_url: self.base_url.clone(),
        }
    }
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
    InvalidConfiguration,
    SecureStorageUnavailable,
    PersistenceFailed,
}

#[tauri::command]
pub(crate) async fn ask_ai<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, AiRuntime>,
    requests: State<'_, AiRequestManager>,
    actions: State<'_, AssistantActionProcessor>,
    settings: State<'_, SettingsState>,
    request: AiConversationRequest,
) -> Result<AiAskResult, AiCommandError> {
    authorize(&window, authorization::ASK_AI)?;
    match requests
        .run(AiRendererRole::Companion, AiOperation::Chat, async {
            let request = request
                .validate()
                .map_err(|_| AiCommandError::InvalidRequest)?;
            let prompt = request
                .to_provider_prompt()
                .map_err(|_| AiCommandError::InvalidRequest)?;
            let resolved = resolve_configuration(runtime.inner(), settings.inner())?;

            if !resolved.ai.enabled {
                return Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE));
            }

            match runtime
                .send_message(resolved.borrowed(), AiRequest { prompt })
                .await
            {
                Ok(response) => match actions.process(response) {
                    Ok(response) => Ok(AiAskResult::success(response)),
                    Err(error) => {
                        eprintln!("[ai-action] action_rejected: {}", action_error_code(error));
                        Ok(AiAskResult::failure("I couldn't complete that action."))
                    }
                },
                Err(AiExecutionError::Provider(_)) => {
                    Ok(AiAskResult::failure(PROVIDER_FAILED_MESSAGE))
                }
                Err(_) => Ok(AiAskResult::failure(AI_UNAVAILABLE_MESSAGE)),
            }
        })
        .await
    {
        Ok(result) => result,
        Err(error) => Ok(AiAskResult::failure(error.message())),
    }
}

#[tauri::command]
pub(crate) fn update_ai_configuration<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, AiRuntime>,
    requests: State<'_, AiRequestManager>,
    credentials: State<'_, CredentialStore>,
    settings: State<'_, SettingsState>,
    configuration: AiConfigurationUpdate,
) -> Result<PreferencesSettings, AiCommandError> {
    authorize(&window, authorization::UPDATE_AI_CONFIGURATION)?;
    let current = settings
        .snapshot()
        .map_err(|_| AiCommandError::SettingsUnavailable)?;
    let next_provider = configuration
        .provider
        .as_deref()
        .unwrap_or(&current.ai.provider);
    let next_endpoint = configuration
        .endpoint
        .as_deref()
        .unwrap_or(&current.ai.endpoint);
    if next_provider == "ollama" && !crate::domain::ai::is_valid_ollama_endpoint(next_endpoint) {
        return Err(AiCommandError::InvalidConfiguration);
    }
    let configuration_changed = configuration
        .enabled
        .is_some_and(|value| value != current.ai.enabled)
        || configuration
            .provider
            .as_ref()
            .is_some_and(|value| value != &current.ai.provider)
        || configuration
            .model
            .as_ref()
            .is_some_and(|value| value != &current.ai.model)
        || configuration
            .endpoint
            .as_ref()
            .is_some_and(|value| value != &current.ai.endpoint)
        || configuration
            .base_url
            .as_ref()
            .is_some_and(|value| value != &current.ai.base_url)
        || configuration.api_key.is_some();

    let previous_credential = if configuration.api_key.is_some() {
        Some(
            credentials
                .load(CredentialId::AiApiKey)
                .map_err(map_credential_error)?,
        )
    } else {
        None
    };
    if let Some(api_key) = configuration.api_key.as_ref() {
        update_credential(credentials.inner(), api_key)?;
    }

    let update = settings.update_ai_configuration(
        configuration.settings_patch(),
        configuration.api_key.is_some(),
    );
    let update = match update {
        Ok(update) => update,
        Err(error) => {
            if let Some(previous) = previous_credential {
                if let Err(rollback_error) = restore_credential(credentials.inner(), previous) {
                    eprintln!(
                        "[security] ai_credential_rollback_failed: {}",
                        credential_error_code(rollback_error)
                    );
                }
            }
            return Err(map_settings_error(error));
        }
    };

    if configuration_changed {
        requests.cancel_all(AiCancellationReason::ProviderChanged);
    }

    match AiProviderId::parse(&update.settings.ai.provider) {
        Some(provider) => runtime
            .select_provider(provider)
            .map_err(|_| AiCommandError::InvalidConfiguration)?,
        None if update.settings.ai.provider.trim().is_empty() => runtime
            .clear_provider()
            .map_err(|_| AiCommandError::SettingsUnavailable)?,
        None => return Err(AiCommandError::InvalidConfiguration),
    }

    let mut preferences = update.settings.preferences_projection();
    preferences.ai.api_key_configured |= credentials
        .is_configured(CredentialId::AiApiKey)
        .map_err(map_credential_error)?;
    Ok(preferences)
}

#[tauri::command]
pub(crate) async fn list_ai_models<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, AiRuntime>,
    requests: State<'_, AiRequestManager>,
    settings: State<'_, SettingsState>,
) -> Result<AiModelListResult, AiCommandError> {
    authorize(&window, authorization::LIST_AI_MODELS)?;
    match requests
        .run(
            AiRendererRole::Preferences,
            AiOperation::ModelDiscovery,
            async {
                let resolved = resolve_configuration(runtime.inner(), settings.inner())?;
                Ok(match runtime.list_models(resolved.borrowed()).await {
                    Ok(models) => AiModelListResult {
                        ok: true,
                        models: Some(models),
                        message: None,
                    },
                    Err(error) => AiModelListResult {
                        ok: false,
                        models: None,
                        message: Some(configuration_error_message(&error)),
                    },
                })
            },
        )
        .await
    {
        Ok(result) => result,
        Err(error) => Ok(AiModelListResult {
            ok: false,
            models: None,
            message: Some(error.message().to_owned()),
        }),
    }
}

#[tauri::command]
pub(crate) async fn test_ai_connection<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, AiRuntime>,
    requests: State<'_, AiRequestManager>,
    settings: State<'_, SettingsState>,
) -> Result<AiConnectionTestResult, AiCommandError> {
    authorize(&window, authorization::TEST_AI_CONNECTION)?;
    match requests
        .run(
            AiRendererRole::Preferences,
            AiOperation::ConnectionTest,
            async {
                let resolved = resolve_configuration(runtime.inner(), settings.inner())?;
                Ok(match runtime.test_connection(resolved.borrowed()).await {
                    Ok(message) => AiConnectionTestResult {
                        ok: true,
                        message: Some(message),
                        diagnostics: None,
                    },
                    Err(error) => {
                        let diagnostics = match &error {
                            AiExecutionError::Provider(error)
                                if error.provider_id == AiProviderId::Grok =>
                            {
                                if let Some(diagnostics) = error.diagnostics() {
                                    eprintln!(
                                        "[ai] grok_connection_test_failed: request_url={:?} \
                                         http_status_code={:?} response_body={:?} error_code={:?} \
                                         error_message={:?}",
                                        diagnostics.request_url,
                                        diagnostics.http_status_code,
                                        diagnostics.response_body,
                                        diagnostics.error_code,
                                        diagnostics.error_message
                                    );
                                } else {
                                    eprintln!(
                                        "[ai] grok_connection_test_failed: error_code={:?} \
                                         error_message={:?}",
                                        error.code,
                                        error.message()
                                    );
                                }
                                error.diagnostics().cloned()
                            }
                            _ => None,
                        };
                        AiConnectionTestResult {
                            ok: false,
                            message: Some(configuration_error_message(&error)),
                            diagnostics,
                        }
                    }
                })
            },
        )
        .await
    {
        Ok(result) => result,
        Err(error) => Ok(AiConnectionTestResult {
            ok: false,
            message: Some(error.message().to_owned()),
            diagnostics: None,
        }),
    }
}

struct ResolvedAiConfiguration {
    ai: StoredAiSettings,
    credential: Option<Zeroizing<String>>,
}

impl ResolvedAiConfiguration {
    fn borrowed(&self) -> AiProviderConfiguration<'_> {
        AiProviderConfiguration {
            api_key: self.credential.as_ref().map(|value| value.as_str()),
            base_url: &self.ai.base_url,
            endpoint: &self.ai.endpoint,
            model: &self.ai.model,
        }
    }
}

fn resolve_configuration(
    runtime: &AiRuntime,
    settings: &SettingsState,
) -> Result<ResolvedAiConfiguration, AiCommandError> {
    let ai = settings
        .snapshot()
        .map_err(|_| AiCommandError::SettingsUnavailable)?
        .ai;
    let provider = AiProviderId::parse(&ai.provider).ok_or(AiCommandError::InvalidConfiguration)?;
    let credential = runtime
        .load_provider_credential(provider)
        .map_err(map_credential_error)?;
    Ok(ResolvedAiConfiguration { ai, credential })
}

fn configuration_error_message(error: &AiExecutionError) -> String {
    match error {
        AiExecutionError::Provider(error)
            if matches!(error.provider_id, AiProviderId::Custom | AiProviderId::Grok) =>
        {
            error.message().to_owned()
        }
        AiExecutionError::Provider(_) => PROVIDER_FAILED_MESSAGE.to_owned(),
        _ => AI_UNAVAILABLE_MESSAGE.to_owned(),
    }
}

fn update_credential(credentials: &CredentialStore, api_key: &str) -> Result<(), AiCommandError> {
    if api_key.trim().is_empty() {
        credentials
            .delete(CredentialId::AiApiKey)
            .map(|_| ())
            .map_err(map_credential_error)
    } else {
        credentials
            .save(CredentialId::AiApiKey, api_key.to_owned())
            .map(|_| ())
            .map_err(map_credential_error)
    }
}

fn restore_credential(
    credentials: &CredentialStore,
    credential: Option<Zeroizing<String>>,
) -> Result<(), CredentialStoreError> {
    match credential {
        Some(credential) => credentials
            .save(CredentialId::AiApiKey, credential.to_string())
            .map(|_| ()),
        None => credentials.delete(CredentialId::AiApiKey).map(|_| ()),
    }
}

fn map_credential_error(error: CredentialStoreError) -> AiCommandError {
    match error {
        CredentialStoreError::InvalidSecret => AiCommandError::InvalidConfiguration,
        CredentialStoreError::Unavailable => AiCommandError::SecureStorageUnavailable,
    }
}

fn credential_error_code(error: CredentialStoreError) -> &'static str {
    match error {
        CredentialStoreError::InvalidSecret => "invalid_secret",
        CredentialStoreError::Unavailable => "secure_storage_unavailable",
    }
}

fn map_settings_error(error: SettingsMutationError) -> AiCommandError {
    match error {
        SettingsMutationError::Validation(_) => AiCommandError::InvalidConfiguration,
        SettingsMutationError::State(_) => AiCommandError::SettingsUnavailable,
        SettingsMutationError::Store(_) => AiCommandError::PersistenceFailed,
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
