use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::{
    webview::PageLoadEvent, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const LABEL: &str = crate::authorization::PREFERENCES_LABEL;

const TITLE: &str = "Ducky Preferences";
const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 650.0;
const MINIMUM_WIDTH: f64 = 520.0;
const MINIMUM_HEIGHT: f64 = 480.0;

pub fn show<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<WebviewWindow<R>> {
    if let Some(window) = manager.get_webview_window(LABEL) {
        window.show()?;
        window.set_focus()?;
        return Ok(window);
    }

    create(manager)
}

fn create<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<WebviewWindow<R>> {
    let startup_handled = Arc::new(AtomicBool::new(false));
    let window =
        WebviewWindowBuilder::new(manager, LABEL, WebviewUrl::App("preferences.html".into()))
            .title(TITLE)
            .inner_size(WIDTH, HEIGHT)
            .min_inner_size(MINIMUM_WIDTH, MINIMUM_HEIGHT)
            .resizable(true)
            .fullscreen(false)
            .visible(false)
            .on_page_load(move |window, payload| {
                if payload.event() == PageLoadEvent::Finished
                    && startup_handled
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                {
                    show_after_page_load(&window);
                }
            })
            .build()?;

    Ok(window)
}

fn show_after_page_load<R: Runtime>(window: &WebviewWindow<R>) {
    if let Err(error) = window.show() {
        eprintln!("[window] preferences_show_failed: {error}");
        return;
    }

    if let Err(error) = window.set_focus() {
        eprintln!("[window] preferences_focus_failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, LABEL, MINIMUM_HEIGHT, MINIMUM_WIDTH, TITLE, WIDTH};

    #[test]
    fn recreated_preferences_window_matches_the_declarative_window() {
        assert_eq!(LABEL, "preferences");
        assert_eq!(TITLE, "Ducky Preferences");
        assert_eq!((WIDTH, HEIGHT), (640.0, 650.0));
        assert_eq!((MINIMUM_WIDTH, MINIMUM_HEIGHT), (520.0, 480.0));
    }
}
