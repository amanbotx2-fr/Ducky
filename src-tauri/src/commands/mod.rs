pub(crate) mod companion;
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
            companion::get_cursor_position,
            companion::get_companion_window_position,
            companion::move_companion_window,
            companion::set_companion_content_height,
            companion::show_companion_context_menu,
            companion::stream_cursor_positions,
            companion::stop_cursor_positions,
            settings::get_runtime_settings
        ])
}
