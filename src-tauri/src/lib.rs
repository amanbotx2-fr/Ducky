mod commands;
mod desktop;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::companion::move_companion_window,
            commands::companion::set_companion_content_height
        ])
        .setup(|app| {
            desktop::windows::companion::create(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
