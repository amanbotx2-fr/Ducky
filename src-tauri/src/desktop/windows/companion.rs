use tauri::{App, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const LABEL: &str = "companion";

const TITLE: &str = "Ducky";
const WIDTH: f64 = 220.0;
const HEIGHT: f64 = 220.0;

pub fn create<R: Runtime>(app: &App<R>) -> tauri::Result<WebviewWindow<R>> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title(TITLE)
        .inner_size(WIDTH, HEIGHT)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .fullscreen(false)
        .skip_taskbar(true)
        .build()
}
