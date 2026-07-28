use serde::Serialize;
use tauri::{Runtime, State, WebviewWindow};

use crate::{
    authorization,
    domain::updater::{UpdateStatus, UpdaterRuntime},
};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UpdateCommandError {
    UnauthorizedWindow,
    UpdaterUnavailable,
}

#[tauri::command]
pub(crate) fn get_update_status<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, UpdaterRuntime>,
) -> Result<UpdateStatus, UpdateCommandError> {
    authorize(&window, authorization::GET_UPDATE_STATUS)?;
    runtime
        .get_status()
        .map_err(|_| UpdateCommandError::UpdaterUnavailable)
}

#[tauri::command]
pub(crate) async fn check_for_updates<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, UpdaterRuntime>,
) -> Result<UpdateStatus, UpdateCommandError> {
    authorize(&window, authorization::CHECK_FOR_UPDATES)?;
    runtime
        .inner()
        .clone()
        .check_for_updates()
        .await
        .map_err(|_| UpdateCommandError::UpdaterUnavailable)
}

fn authorize<R: Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), UpdateCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| UpdateCommandError::UnauthorizedWindow)
}
