use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::settings::SettingsState,
    infrastructure::credentials::{
        CredentialId as NativeCredentialId, CredentialStore, CredentialStoreError,
    },
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CredentialId {
    AiApiKey,
}

impl CredentialId {
    const fn native(self) -> NativeCredentialId {
        match self {
            Self::AiApiKey => NativeCredentialId::AiApiKey,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CredentialState {
    Configured,
    Missing,
    RequiresReentry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CredentialStatus {
    id: CredentialId,
    state: CredentialState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CredentialCommandError {
    UnauthorizedWindow,
    InvalidCredential,
    SecureStorageUnavailable,
    SettingsUnavailable,
}

#[tauri::command]
pub(crate) fn get_credential_status<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    credentials: State<'_, CredentialStore>,
    settings: State<'_, SettingsState>,
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    authorize(&window, authorization::GET_CREDENTIAL_STATUS)?;
    status(credentials.inner(), settings.inner(), id)
}

#[tauri::command]
pub(crate) fn save_credential<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    credentials: State<'_, CredentialStore>,
    id: CredentialId,
    secret: String,
) -> Result<CredentialStatus, CredentialCommandError> {
    authorize(&window, authorization::SAVE_CREDENTIAL)?;
    credentials
        .save(id.native(), secret)
        .map_err(map_store_error)?;
    Ok(CredentialStatus {
        id,
        state: CredentialState::Configured,
    })
}

#[tauri::command]
pub(crate) fn delete_credential<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    credentials: State<'_, CredentialStore>,
    settings: State<'_, SettingsState>,
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    authorize(&window, authorization::DELETE_CREDENTIAL)?;
    credentials.delete(id.native()).map_err(map_store_error)?;
    status(credentials.inner(), settings.inner(), id)
}

fn authorize<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), CredentialCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| CredentialCommandError::UnauthorizedWindow)
}

fn status(
    credentials: &CredentialStore,
    settings: &SettingsState,
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    let configured = credentials
        .is_configured(id.native())
        .map_err(map_store_error)?;
    let state = if configured {
        CredentialState::Configured
    } else if legacy_credential_is_present(settings)? {
        CredentialState::RequiresReentry
    } else {
        CredentialState::Missing
    };
    Ok(CredentialStatus { id, state })
}

fn legacy_credential_is_present(settings: &SettingsState) -> Result<bool, CredentialCommandError> {
    settings
        .snapshot()
        .map(|settings| {
            settings.credential.is_some()
                || settings
                    .ai
                    .api_key
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty())
        })
        .map_err(|_| CredentialCommandError::SettingsUnavailable)
}

fn map_store_error(error: CredentialStoreError) -> CredentialCommandError {
    match error {
        CredentialStoreError::InvalidSecret => CredentialCommandError::InvalidCredential,
        CredentialStoreError::Unavailable => CredentialCommandError::SecureStorageUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;
    use crate::{domain::settings::SettingsDocument, infrastructure::persistence::SettingsStore};

    fn settings_state(settings: SettingsDocument) -> SettingsState {
        SettingsState::new(
            SettingsStore::new(tempdir().unwrap().path().join("settings.json")),
            settings,
        )
    }

    #[test]
    fn credential_status_is_secret_free() {
        let serialized = serde_json::to_value(CredentialStatus {
            id: CredentialId::AiApiKey,
            state: CredentialState::Configured,
        })
        .unwrap();

        assert_eq!(
            serialized,
            json!({ "id": "aiApiKey", "state": "configured" })
        );
        assert!(serialized.get("secret").is_none());
        assert!(serialized.get("value").is_none());
    }

    #[test]
    fn imported_electron_credentials_require_safe_reentry() {
        let mut settings = SettingsDocument::default();
        settings.credential = Some(json!({
            "version": 1,
            "ciphertext": "opaque-electron-value"
        }));
        let settings = settings_state(settings);

        assert!(legacy_credential_is_present(&settings).unwrap());
    }

    #[test]
    fn absent_and_empty_legacy_credentials_do_not_require_reentry() {
        let mut settings = SettingsDocument::default();
        settings.ai.api_key = Some("  ".to_owned());
        let settings = settings_state(settings);

        assert!(!legacy_credential_is_present(&settings).unwrap());
    }
}
