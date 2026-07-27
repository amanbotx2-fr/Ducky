pub(crate) mod ai;
pub(crate) mod companion;
pub(crate) mod credentials;
pub(crate) mod pomodoro;
pub(crate) mod reminders;
pub(crate) mod settings;

use tauri::{Builder, Runtime};

/// Registers the complete Tauri command surface available through Phase 3.
///
/// Keeping command state and dispatch here prevents the application
/// composition root from becoming a second command registry.
pub(crate) fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder
        .manage(companion::CursorStreamState::default())
        .invoke_handler(tauri::generate_handler![
            ai::ask_ai,
            ai::update_ai_configuration,
            ai::list_ai_models,
            ai::test_ai_connection,
            companion::get_cursor_position,
            companion::get_companion_window_position,
            companion::move_companion_window,
            companion::set_companion_content_height,
            companion::show_companion_context_menu,
            companion::stream_cursor_positions,
            companion::stop_cursor_positions,
            credentials::get_credential_status,
            credentials::save_credential,
            credentials::delete_credential,
            reminders::create_reminder,
            reminders::update_reminder,
            reminders::delete_reminder,
            reminders::get_reminder,
            reminders::list_reminders,
            reminders::mark_reminder_completed,
            reminders::activate_reminder_events,
            pomodoro::start_pomodoro,
            pomodoro::custom_pomodoro_panel_closed,
            pomodoro::activate_pomodoro_events,
            settings::get_runtime_settings,
            settings::get_preferences_settings,
            settings::update_user_name,
            settings::update_sticky_message,
            settings::update_preferences_settings
        ])
}
