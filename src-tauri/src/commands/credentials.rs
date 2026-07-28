use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
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
}

#[tauri::command]
pub(crate) fn get_credential_status<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    credentials: State<'_, CredentialStore>,
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    authorize(&window, authorization::GET_CREDENTIAL_STATUS)?;
    status(credentials.inner(), id)
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
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    authorize(&window, authorization::DELETE_CREDENTIAL)?;
    credentials.delete(id.native()).map_err(map_store_error)?;
    status(credentials.inner(), id)
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
    id: CredentialId,
) -> Result<CredentialStatus, CredentialCommandError> {
    let configured = credentials
        .is_configured(id.native())
        .map_err(map_store_error)?;
    let state = if configured {
        CredentialState::Configured
    } else {
        CredentialState::Missing
    };
    Ok(CredentialStatus { id, state })
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

    use super::*;

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
}
