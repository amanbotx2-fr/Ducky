use tauri::{
    menu::Menu,
    tray::{TrayIcon, TrayIconBuilder},
    App, AppHandle, Runtime,
};

pub const ID: &str = "ducky-tray";

const TOOLTIP: &str = "Ducky";

pub fn create<R: Runtime>(app: &App<R>) -> tauri::Result<TrayIcon<R>> {
    if let Some(existing) = app.tray_by_id(ID) {
        return Ok(existing);
    }

    // Linux status notifier implementations may hide icons without a menu.
    // Task 4.4 replaces this empty native menu with Ducky's static tray menu.
    let menu = Menu::new(app)?;
    let mut builder = TrayIconBuilder::with_id(ID).tooltip(TOOLTIP).menu(&menu);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).icon_as_template(false);
    }

    builder.build(app)
}

pub fn destroy<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray) = app.remove_tray_by_id(ID) {
        drop(tray);
    }
}

#[cfg(test)]
mod tests {
    use super::{ID, TOOLTIP};

    #[test]
    fn tray_identity_is_stable() {
        assert_eq!(ID, "ducky-tray");
        assert_eq!(TOOLTIP, "Ducky");
    }
}
