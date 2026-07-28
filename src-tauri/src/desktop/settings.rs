use tauri::{App, AppHandle, Manager, Runtime};

use crate::domain::settings::GeneralSettings;

pub(crate) fn apply<R: Runtime>(
    app: &App<R>,
    settings: &GeneralSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    apply_handle(app.handle(), settings)
}

pub(crate) fn apply_handle<R: Runtime>(
    app: &AppHandle<R>,
    settings: &GeneralSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(companion) = app.get_webview_window("companion") {
        companion.set_always_on_top(settings.always_on_top)?;
    }

    apply_launch_at_startup(app, settings.launch_at_startup)?;
    Ok(())
}

#[cfg(all(desktop, not(debug_assertions)))]
fn apply_launch_at_startup<R: Runtime>(
    app: &AppHandle<R>,
    enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    if enabled {
        manager.enable()?;
    } else {
        manager.disable()?;
    }

    Ok(())
}

#[cfg(any(not(desktop), debug_assertions))]
fn apply_launch_at_startup<R: Runtime>(
    _app: &AppHandle<R>,
    _enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
