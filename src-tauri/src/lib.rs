mod commands;
mod desktop;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::companion::CursorStreamState::default())
        .invoke_handler(tauri::generate_handler![
            commands::companion::get_cursor_position,
            commands::companion::move_companion_window,
            commands::companion::set_companion_content_height,
            commands::companion::stream_cursor_positions
        ])
        .setup(|app| {
            desktop::windows::companion::create(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
