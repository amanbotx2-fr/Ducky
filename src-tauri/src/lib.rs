mod authorization;
mod commands;
mod desktop;
// Event producers are connected by their owning feature phases. Compile and
// test the complete infrastructure now without registering placeholder events.
#[allow(dead_code)]
mod events;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = commands::register(tauri::Builder::default())
        .setup(|app| {
            desktop::windows::companion::create(app)?;
            desktop::tray::create(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(desktop::lifecycle::handle_run_event);
}
