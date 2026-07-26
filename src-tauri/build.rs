fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_cursor_position",
            "move_companion_window",
            "set_companion_content_height",
            "stream_cursor_positions",
        ]),
    ))
    .expect("failed to build Tauri application");
}
