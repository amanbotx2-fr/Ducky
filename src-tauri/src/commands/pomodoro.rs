use serde::Serialize;
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::pomodoro::{PomodoroError, PomodoroEventQueue, PomodoroRuntime},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PomodoroCommandError {
    UnauthorizedWindow,
    InvalidDuration,
    PomodoroUnavailable,
}

#[tauri::command]
pub(crate) fn start_pomodoro<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, PomodoroRuntime>,
    duration_minutes: u32,
) -> Result<(), PomodoroCommandError> {
    authorize(&window, authorization::START_POMODORO)?;
    runtime
        .start_session(duration_minutes)
        .map(|_| ())
        .map_err(map_runtime_error)
}

#[tauri::command]
pub(crate) fn custom_pomodoro_panel_closed<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    events: State<'_, PomodoroEventQueue>,
) -> Result<(), PomodoroCommandError> {
    authorize(&window, authorization::CUSTOM_POMODORO_PANEL_CLOSED)?;
    events
        .close_custom_panel()
        .map_err(|_| PomodoroCommandError::PomodoroUnavailable)
}

#[tauri::command]
pub(crate) fn activate_pomodoro_events<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    events: State<'_, PomodoroEventQueue>,
) -> Result<(), PomodoroCommandError> {
    authorize(&window, authorization::ACTIVATE_POMODORO_EVENTS)?;
    events
        .activate()
        .map_err(|_| PomodoroCommandError::PomodoroUnavailable)
}

fn authorize<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), PomodoroCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| PomodoroCommandError::UnauthorizedWindow)
}

fn map_runtime_error(error: PomodoroError) -> PomodoroCommandError {
    match error {
        PomodoroError::InvalidDuration => PomodoroCommandError::InvalidDuration,
        PomodoroError::InvalidState
        | PomodoroError::InvalidDocument
        | PomodoroError::StateUnavailable
        | PomodoroError::RuntimeUnavailable => PomodoroCommandError::PomodoroUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_errors_map_to_stable_command_errors() {
        assert_eq!(
            map_runtime_error(PomodoroError::InvalidDuration),
            PomodoroCommandError::InvalidDuration
        );
        assert_eq!(
            map_runtime_error(PomodoroError::StateUnavailable),
            PomodoroCommandError::PomodoroUnavailable
        );
    }
}
