mod app_state;
mod authorization;
mod commands;
mod desktop;
mod domain;
// Event producers are connected by their owning feature phases. Compile and
// test the complete infrastructure now without registering placeholder events.
#[allow(dead_code)]
mod events;
mod infrastructure;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None,
    ));

    let app = commands::register(builder)
        .on_page_load(app_state::handle_page_load)
        .setup(|app| {
            app_state::initialize(app)?;
            desktop::menus::install_application_menu(app)?;
            desktop::windows::companion::create(app)?;
            let settings = app.state::<domain::settings::SettingsState>().snapshot()?;
            desktop::settings::apply(app, &settings.general)?;
            desktop::tray::create(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(desktop::lifecycle::handle_run_event);
}
