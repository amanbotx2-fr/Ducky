pub(crate) const COMPANION_LABEL: &str = "companion";
pub(crate) const PREFERENCES_LABEL: &str = "preferences";

const COMPANION_ONLY: &[RendererRole] = &[RendererRole::Companion];
const PREFERENCES_ONLY: &[RendererRole] = &[RendererRole::Preferences];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum RendererRole {
    Companion,
    Preferences,
}

impl RendererRole {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Companion => COMPANION_LABEL,
            Self::Preferences => PREFERENCES_LABEL,
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            COMPANION_LABEL => Some(Self::Companion),
            PREFERENCES_LABEL => Some(Self::Preferences),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CommandAuthorization {
    name: &'static str,
    allowed_roles: &'static [RendererRole],
}

impl CommandAuthorization {
    const fn companion_only(name: &'static str) -> Self {
        Self {
            name,
            allowed_roles: COMPANION_ONLY,
        }
    }

    const fn preferences_only(name: &'static str) -> Self {
        Self {
            name,
            allowed_roles: PREFERENCES_ONLY,
        }
    }

    // The build script compiles this module separately to generate command
    // permissions, so the application library does not call this directly.
    #[allow(dead_code)]
    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    pub(crate) const fn allowed_roles(self) -> &'static [RendererRole] {
        self.allowed_roles
    }
}

pub(crate) const GET_CURSOR_POSITION: CommandAuthorization =
    CommandAuthorization::companion_only("get_cursor_position");
pub(crate) const GET_RUNTIME_SETTINGS: CommandAuthorization =
    CommandAuthorization::companion_only("get_runtime_settings");
pub(crate) const GET_PREFERENCES_SETTINGS: CommandAuthorization =
    CommandAuthorization::preferences_only("get_preferences_settings");
pub(crate) const UPDATE_USER_NAME: CommandAuthorization =
    CommandAuthorization::companion_only("update_user_name");
pub(crate) const UPDATE_STICKY_MESSAGE: CommandAuthorization =
    CommandAuthorization::companion_only("update_sticky_message");
pub(crate) const UPDATE_PREFERENCES_SETTINGS: CommandAuthorization =
    CommandAuthorization::preferences_only("update_preferences_settings");
pub(crate) const GET_CREDENTIAL_STATUS: CommandAuthorization =
    CommandAuthorization::preferences_only("get_credential_status");
pub(crate) const SAVE_CREDENTIAL: CommandAuthorization =
    CommandAuthorization::preferences_only("save_credential");
pub(crate) const DELETE_CREDENTIAL: CommandAuthorization =
    CommandAuthorization::preferences_only("delete_credential");
pub(crate) const CREATE_REMINDER: CommandAuthorization =
    CommandAuthorization::companion_only("create_reminder");
pub(crate) const UPDATE_REMINDER: CommandAuthorization =
    CommandAuthorization::companion_only("update_reminder");
pub(crate) const DELETE_REMINDER: CommandAuthorization =
    CommandAuthorization::companion_only("delete_reminder");
pub(crate) const GET_REMINDER: CommandAuthorization =
    CommandAuthorization::companion_only("get_reminder");
pub(crate) const LIST_REMINDERS: CommandAuthorization =
    CommandAuthorization::companion_only("list_reminders");
pub(crate) const MARK_REMINDER_COMPLETED: CommandAuthorization =
    CommandAuthorization::companion_only("mark_reminder_completed");
pub(crate) const ACTIVATE_REMINDER_EVENTS: CommandAuthorization =
    CommandAuthorization::companion_only("activate_reminder_events");
pub(crate) const START_POMODORO: CommandAuthorization =
    CommandAuthorization::companion_only("start_pomodoro");
pub(crate) const CUSTOM_POMODORO_PANEL_CLOSED: CommandAuthorization =
    CommandAuthorization::companion_only("custom_pomodoro_panel_closed");
pub(crate) const ACTIVATE_POMODORO_EVENTS: CommandAuthorization =
    CommandAuthorization::companion_only("activate_pomodoro_events");
pub(crate) const GET_COMPANION_WINDOW_POSITION: CommandAuthorization =
    CommandAuthorization::companion_only("get_companion_window_position");
pub(crate) const MOVE_COMPANION_WINDOW: CommandAuthorization =
    CommandAuthorization::companion_only("move_companion_window");
pub(crate) const SET_COMPANION_CONTENT_HEIGHT: CommandAuthorization =
    CommandAuthorization::companion_only("set_companion_content_height");
pub(crate) const SHOW_COMPANION_CONTEXT_MENU: CommandAuthorization =
    CommandAuthorization::companion_only("show_companion_context_menu");
pub(crate) const STREAM_CURSOR_POSITIONS: CommandAuthorization =
    CommandAuthorization::companion_only("stream_cursor_positions");
pub(crate) const STOP_CURSOR_POSITIONS: CommandAuthorization =
    CommandAuthorization::companion_only("stop_cursor_positions");
pub(crate) const ASK_AI: CommandAuthorization = CommandAuthorization::companion_only("ask_ai");
pub(crate) const UPDATE_AI_CONFIGURATION: CommandAuthorization =
    CommandAuthorization::preferences_only("update_ai_configuration");
pub(crate) const LIST_AI_MODELS: CommandAuthorization =
    CommandAuthorization::preferences_only("list_ai_models");
pub(crate) const TEST_AI_CONNECTION: CommandAuthorization =
    CommandAuthorization::preferences_only("test_ai_connection");
pub(crate) const GET_UPDATE_STATUS: CommandAuthorization =
    CommandAuthorization::preferences_only("get_update_status");
pub(crate) const CHECK_FOR_UPDATES: CommandAuthorization =
    CommandAuthorization::preferences_only("check_for_updates");
pub(crate) const GET_DAILY_PLANNER: CommandAuthorization =
    CommandAuthorization::companion_only("get_daily_planner");

#[cfg(test)]
pub(crate) const MIGRATED_COMMANDS: &[CommandAuthorization] = &[
    GET_CURSOR_POSITION,
    GET_RUNTIME_SETTINGS,
    GET_PREFERENCES_SETTINGS,
    UPDATE_USER_NAME,
    UPDATE_STICKY_MESSAGE,
    UPDATE_PREFERENCES_SETTINGS,
    GET_CREDENTIAL_STATUS,
    SAVE_CREDENTIAL,
    DELETE_CREDENTIAL,
    CREATE_REMINDER,
    UPDATE_REMINDER,
    DELETE_REMINDER,
    GET_REMINDER,
    LIST_REMINDERS,
    MARK_REMINDER_COMPLETED,
    ACTIVATE_REMINDER_EVENTS,
    START_POMODORO,
    CUSTOM_POMODORO_PANEL_CLOSED,
    ACTIVATE_POMODORO_EVENTS,
    GET_COMPANION_WINDOW_POSITION,
    MOVE_COMPANION_WINDOW,
    SET_COMPANION_CONTENT_HEIGHT,
    SHOW_COMPANION_CONTEXT_MENU,
    STREAM_CURSOR_POSITIONS,
    STOP_CURSOR_POSITIONS,
    ASK_AI,
    UPDATE_AI_CONFIGURATION,
    LIST_AI_MODELS,
    TEST_AI_CONNECTION,
    GET_UPDATE_STATUS,
    CHECK_FOR_UPDATES,
    GET_DAILY_PLANNER,
];

// Consumed by build.rs through a path module; retained here as the single
// source for generated application-command permissions.
#[allow(dead_code)]
pub(crate) const MIGRATED_COMMAND_NAMES: &[&str] = &[
    GET_CURSOR_POSITION.name(),
    GET_RUNTIME_SETTINGS.name(),
    GET_PREFERENCES_SETTINGS.name(),
    UPDATE_USER_NAME.name(),
    UPDATE_STICKY_MESSAGE.name(),
    UPDATE_PREFERENCES_SETTINGS.name(),
    GET_CREDENTIAL_STATUS.name(),
    SAVE_CREDENTIAL.name(),
    DELETE_CREDENTIAL.name(),
    CREATE_REMINDER.name(),
    UPDATE_REMINDER.name(),
    DELETE_REMINDER.name(),
    GET_REMINDER.name(),
    LIST_REMINDERS.name(),
    MARK_REMINDER_COMPLETED.name(),
    ACTIVATE_REMINDER_EVENTS.name(),
    START_POMODORO.name(),
    CUSTOM_POMODORO_PANEL_CLOSED.name(),
    ACTIVATE_POMODORO_EVENTS.name(),
    GET_COMPANION_WINDOW_POSITION.name(),
    MOVE_COMPANION_WINDOW.name(),
    SET_COMPANION_CONTENT_HEIGHT.name(),
    SHOW_COMPANION_CONTEXT_MENU.name(),
    STREAM_CURSOR_POSITIONS.name(),
    STOP_CURSOR_POSITIONS.name(),
    ASK_AI.name(),
    UPDATE_AI_CONFIGURATION.name(),
    LIST_AI_MODELS.name(),
    TEST_AI_CONNECTION.name(),
    GET_UPDATE_STATUS.name(),
    CHECK_FOR_UPDATES.name(),
    GET_DAILY_PLANNER.name(),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthorizationError {
    UnknownRenderer,
    CapabilityDenied,
}

pub(crate) fn authorize_command(
    renderer_label: &str,
    command: CommandAuthorization,
) -> Result<RendererRole, AuthorizationError> {
    let role =
        RendererRole::from_label(renderer_label).ok_or(AuthorizationError::UnknownRenderer)?;

    if command.allowed_roles().contains(&role) {
        Ok(role)
    } else {
        Err(AuthorizationError::CapabilityDenied)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn command_manifest_is_unique_and_matches_the_completed_scope() {
        let names = MIGRATED_COMMANDS
            .iter()
            .map(|command| command.name())
            .collect::<Vec<_>>();

        assert_eq!(names, MIGRATED_COMMAND_NAMES);
        assert_eq!(
            names.iter().copied().collect::<HashSet<_>>().len(),
            names.len(),
        );
        assert_eq!(
            names,
            [
                "get_cursor_position",
                "get_runtime_settings",
                "get_preferences_settings",
                "update_user_name",
                "update_sticky_message",
                "update_preferences_settings",
                "get_credential_status",
                "save_credential",
                "delete_credential",
                "create_reminder",
                "update_reminder",
                "delete_reminder",
                "get_reminder",
                "list_reminders",
                "mark_reminder_completed",
                "activate_reminder_events",
                "start_pomodoro",
                "custom_pomodoro_panel_closed",
                "activate_pomodoro_events",
                "get_companion_window_position",
                "move_companion_window",
                "set_companion_content_height",
                "show_companion_context_menu",
                "stream_cursor_positions",
                "stop_cursor_positions",
                "ask_ai",
                "update_ai_configuration",
                "list_ai_models",
                "test_ai_connection",
                "get_update_status",
                "check_for_updates",
                "get_daily_planner",
            ],
        );
    }

    #[test]
    fn migrated_commands_have_exact_renderer_roles() {
        for command in MIGRATED_COMMANDS {
            if command == &GET_PREFERENCES_SETTINGS
                || command == &UPDATE_PREFERENCES_SETTINGS
                || command == &GET_CREDENTIAL_STATUS
                || command == &SAVE_CREDENTIAL
                || command == &DELETE_CREDENTIAL
                || command == &UPDATE_AI_CONFIGURATION
                || command == &LIST_AI_MODELS
                || command == &TEST_AI_CONNECTION
                || command == &GET_UPDATE_STATUS
                || command == &CHECK_FOR_UPDATES
            {
                assert_eq!(command.allowed_roles(), [RendererRole::Preferences]);
                assert_eq!(
                    authorize_command(PREFERENCES_LABEL, *command),
                    Ok(RendererRole::Preferences),
                );
                assert_eq!(
                    authorize_command(COMPANION_LABEL, *command),
                    Err(AuthorizationError::CapabilityDenied),
                );
                continue;
            }

            assert_eq!(command.allowed_roles(), [RendererRole::Companion]);
            assert_eq!(
                authorize_command(COMPANION_LABEL, *command),
                Ok(RendererRole::Companion),
            );
            assert_eq!(
                authorize_command(PREFERENCES_LABEL, *command),
                Err(AuthorizationError::CapabilityDenied),
            );
        }
    }

    #[test]
    fn unknown_renderer_labels_have_no_command_authority() {
        for label in ["", "*", "main", "companion-child", "preferences-child"] {
            assert_eq!(
                authorize_command(label, GET_CURSOR_POSITION),
                Err(AuthorizationError::UnknownRenderer),
            );
        }
    }
}
