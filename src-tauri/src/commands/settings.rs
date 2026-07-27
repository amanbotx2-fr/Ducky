use tauri::{Runtime, State, WebviewWindow};

use crate::{
    authorization,
    domain::settings::{RuntimeSettings, SettingsState},
};

#[tauri::command]
pub(crate) fn get_runtime_settings<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, SettingsState>,
) -> Result<RuntimeSettings, String> {
    authorization::authorize_command(window.label(), authorization::GET_RUNTIME_SETTINGS)
        .map_err(|_| "Settings are unavailable in this window.".to_owned())?;

    state
        .snapshot()
        .map(|settings| settings.runtime_projection())
        .map_err(|_| "Settings are temporarily unavailable.".to_owned())
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
}
