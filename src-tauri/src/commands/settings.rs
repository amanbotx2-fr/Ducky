use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};

use crate::{
    authorization, desktop,
    domain::settings::{
        PreferencesSettings, PreferencesSettingsPatch, RuntimeSettings, SettingsMutationError,
        SettingsState, SettingsUpdate,
    },
    domain::updater::UpdaterRuntime,
    events::{self, DesktopEvent},
};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SettingsCommandError {
    UnauthorizedWindow,
    InvalidSettings,
    SettingsUnavailable,
    PersistenceFailed,
}

#[tauri::command]
pub(crate) fn get_runtime_settings<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, SettingsState>,
) -> Result<RuntimeSettings, SettingsCommandError> {
    authorize(&window, authorization::GET_RUNTIME_SETTINGS)?;

    state
        .snapshot()
        .map(|settings| settings.runtime_projection())
        .map_err(|_| SettingsCommandError::SettingsUnavailable)
}

#[tauri::command]
pub(crate) fn get_preferences_settings<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, SettingsState>,
) -> Result<PreferencesSettings, SettingsCommandError> {
    authorize(&window, authorization::GET_PREFERENCES_SETTINGS)?;

    state
        .snapshot()
        .map(|settings| settings.preferences_projection())
        .map_err(|_| SettingsCommandError::SettingsUnavailable)
}

#[tauri::command]
pub(crate) fn update_user_name<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    state: State<'_, SettingsState>,
    name: String,
) -> Result<String, SettingsCommandError> {
    authorize(&window, authorization::UPDATE_USER_NAME)?;
    let update = state.update_user_name(name).map_err(map_mutation_error)?;
    let user_name = update.settings.user_name.clone();
    finish_update(&app, update);
    Ok(user_name)
}

#[tauri::command]
pub(crate) fn update_sticky_message<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    state: State<'_, SettingsState>,
    message: Option<String>,
) -> Result<Option<String>, SettingsCommandError> {
    authorize(&window, authorization::UPDATE_STICKY_MESSAGE)?;
    let update = state
        .update_sticky_message(message)
        .map_err(map_mutation_error)?;
    let sticky_message = update.settings.sticky_message.clone();
    finish_update(&app, update);
    Ok(sticky_message)
}

#[tauri::command]
pub(crate) fn update_preferences_settings<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    state: State<'_, SettingsState>,
    patch: PreferencesSettingsPatch,
) -> Result<PreferencesSettings, SettingsCommandError> {
    authorize(&window, authorization::UPDATE_PREFERENCES_SETTINGS)?;
    let update = state
        .update_preferences(patch)
        .map_err(map_mutation_error)?;
    let preferences = update.settings.preferences_projection();
    finish_update(&app, update);
    Ok(preferences)
}

fn authorize<R: Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), SettingsCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| SettingsCommandError::UnauthorizedWindow)
}

fn map_mutation_error(error: SettingsMutationError) -> SettingsCommandError {
    match error {
        SettingsMutationError::Validation(_) => SettingsCommandError::InvalidSettings,
        SettingsMutationError::State(_) => SettingsCommandError::SettingsUnavailable,
        SettingsMutationError::Store(_) => SettingsCommandError::PersistenceFailed,
    }
}

fn finish_update<R: Runtime>(app: &AppHandle<R>, update: SettingsUpdate) {
    if !update.changed {
        return;
    }

    if let Err(error) = desktop::settings::apply_handle(app, &update.settings.general) {
        eprintln!("[settings] native settings application failed: {error}");
    }

    if let Err(error) = events::emit(
        app,
        DesktopEvent::RuntimeSettingsChanged,
        update.settings.runtime_projection(),
    ) {
        eprintln!("[settings] runtime settings notification failed: {error}");
    }

    let Some(updater) = app.try_state::<UpdaterRuntime>() else {
        eprintln!("[updates] automatic setting application failed: updater_unavailable");
        return;
    };

    match updater.set_automatic_checks_enabled(update.settings.updates.automatic) {
        Ok(true) => {
            let updater = updater.inner().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = updater.check_automatically().await {
                    eprintln!("[updates] automatic check failed: {error}");
                }
            });
        }
        Ok(false) => {}
        Err(error) => {
            eprintln!("[updates] automatic setting application failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::settings::SettingsDocument;

    #[test]
    fn runtime_projection_never_contains_deferred_or_secret_fields() {
        let serialized =
            serde_json::to_value(SettingsDocument::default().runtime_projection()).unwrap();

        assert_eq!(
            serialized.as_object().unwrap().keys().collect::<Vec<_>>(),
            [
                "general",
                "notificationSounds",
                "stickyMessage",
                "userName",
                "water"
            ]
        );
        assert!(serialized.get("ai").is_none());
        assert!(serialized.get("credential").is_none());
        assert!(serialized.get("reminders").is_none());
        assert!(serialized.get("updates").is_none());
    }

    #[test]
    fn preferences_projection_redacts_credentials() {
        let mut settings = SettingsDocument::default();
        settings.credential = Some(serde_json::json!({
            "version": 1,
            "ciphertext": "secret-ciphertext"
        }));
        let serialized = serde_json::to_value(settings.preferences_projection()).unwrap();

        assert_eq!(serialized["ai"]["apiKeyConfigured"], true);
        assert!(serialized.get("credential").is_none());
        assert!(serialized["ai"].get("apiKey").is_none());
        assert!(serialized.get("reminders").is_none());
        assert!(serialized.get("stickyMessage").is_none());
    }

    #[test]
    fn preferences_projection_contains_the_existing_preferences_contract() {
        let serialized =
            serde_json::to_value(SettingsDocument::default().preferences_projection()).unwrap();

        assert_eq!(
            serialized.as_object().unwrap().keys().collect::<Vec<_>>(),
            [
                "ai",
                "aiModelExplorer",
                "general",
                "notificationSounds",
                "updates",
                "userName",
                "water"
            ]
        );
    }
}
