use std::io::{Error as IoError, ErrorKind};

use image::imageops::FilterType;
use tauri::{
    image::Image,
    menu::Menu,
    tray::{TrayIcon, TrayIconBuilder},
    App, AppHandle, Runtime,
};

pub const ID: &str = "ducky-tray";

const TOOLTIP: &str = "Ducky";
const SOURCE_ICON: &[u8] = include_bytes!("../../../assets/icons/icon.png");

#[cfg(target_os = "macos")]
const ICON_SIZE: u32 = 18;
#[cfg(not(target_os = "macos"))]
const ICON_SIZE: u32 = 20;

pub fn create<R: Runtime>(app: &App<R>) -> tauri::Result<TrayIcon<R>> {
    if let Some(existing) = app.tray_by_id(ID) {
        return Ok(existing);
    }

    // Linux status notifier implementations may hide icons without a menu.
    // Task 4.4 replaces this empty native menu with Ducky's static tray menu.
    let menu = Menu::new(app)?;
    let icon = load_icon()?;

    TrayIconBuilder::with_id(ID)
        .tooltip(TOOLTIP)
        .menu(&menu)
        .icon(icon)
        .icon_as_template(false)
        .build(app)
}

pub fn destroy<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray) = app.remove_tray_by_id(ID) {
        drop(tray);
    }
}

fn load_icon() -> tauri::Result<Image<'static>> {
    let source = image::load_from_memory(SOURCE_ICON).map_err(invalid_icon)?;
    let resized = source.resize_exact(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
    let rgba = resized.into_rgba8();

    Ok(Image::new_owned(rgba.into_raw(), ICON_SIZE, ICON_SIZE))
}

fn invalid_icon(error: image::ImageError) -> tauri::Error {
    tauri::Error::InvalidIcon(IoError::new(ErrorKind::InvalidData, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{load_icon, ICON_SIZE, ID, SOURCE_ICON, TOOLTIP};

    #[test]
    fn tray_identity_is_stable() {
        assert_eq!(ID, "ducky-tray");
        assert_eq!(TOOLTIP, "Ducky");
    }

    #[test]
    fn reuses_and_resizes_the_existing_application_icon() {
        assert!(SOURCE_ICON.starts_with(b"\x89PNG\r\n\x1a\n"));

        let icon = load_icon().expect("embedded application icon must decode");

        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert_eq!(icon.rgba().len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }
}
