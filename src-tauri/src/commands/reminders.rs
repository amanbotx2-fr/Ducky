use serde::Serialize;
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::reminders::{
        CreateReminderInput, Reminder, ReminderRuntime, ReminderServiceError, UpdateReminderInput,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReminderCommandError {
    UnauthorizedWindow,
    InvalidReminder,
    ReminderNotFound,
    RemindersUnavailable,
    PersistenceFailed,
}

#[tauri::command]
pub(crate) fn create_reminder<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
    input: CreateReminderInput,
) -> Result<Reminder, ReminderCommandError> {
    authorize(&window, authorization::CREATE_REMINDER)?;
    runtime.service.create(input).map_err(map_service_error)
}

#[tauri::command]
pub(crate) fn update_reminder<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
    id: String,
    input: UpdateReminderInput,
) -> Result<Reminder, ReminderCommandError> {
    authorize(&window, authorization::UPDATE_REMINDER)?;
    runtime
        .service
        .update(&id, input)
        .map_err(map_service_error)
}

#[tauri::command]
pub(crate) fn delete_reminder<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
    id: String,
) -> Result<bool, ReminderCommandError> {
    authorize(&window, authorization::DELETE_REMINDER)?;
    runtime.service.delete(&id).map_err(map_service_error)
}

#[tauri::command]
pub(crate) fn get_reminder<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
    id: String,
) -> Result<Option<Reminder>, ReminderCommandError> {
    authorize(&window, authorization::GET_REMINDER)?;
    runtime.service.get(&id).map_err(map_service_error)
}

#[tauri::command]
pub(crate) fn list_reminders<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
) -> Result<Vec<Reminder>, ReminderCommandError> {
    authorize(&window, authorization::LIST_REMINDERS)?;
    runtime.service.list().map_err(map_service_error)
}

#[tauri::command]
pub(crate) fn mark_reminder_completed<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, ReminderRuntime>,
    id: String,
) -> Result<Reminder, ReminderCommandError> {
    authorize(&window, authorization::MARK_REMINDER_COMPLETED)?;
    runtime
        .service
        .mark_completed(&id)
        .map_err(map_service_error)
}

fn authorize<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    command: authorization::CommandAuthorization,
) -> Result<(), ReminderCommandError> {
    authorization::authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| ReminderCommandError::UnauthorizedWindow)
}

fn map_service_error(error: ReminderServiceError) -> ReminderCommandError {
    match error {
        ReminderServiceError::Validation(_) => ReminderCommandError::InvalidReminder,
        ReminderServiceError::NotFound => ReminderCommandError::ReminderNotFound,
        ReminderServiceError::Repository(_) => ReminderCommandError::PersistenceFailed,
        ReminderServiceError::StateUnavailable
        | ReminderServiceError::ClockUnavailable
        | ReminderServiceError::IdUnavailable
        | ReminderServiceError::RecurrenceUnavailable => ReminderCommandError::RemindersUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_errors_map_to_stable_command_errors() {
        assert_eq!(
            map_service_error(ReminderServiceError::NotFound),
            ReminderCommandError::ReminderNotFound
        );
        assert_eq!(
            map_service_error(ReminderServiceError::IdUnavailable),
            ReminderCommandError::RemindersUnavailable
        );
    }
}
