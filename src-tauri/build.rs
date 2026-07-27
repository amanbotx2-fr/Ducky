#[path = "src/commands/manifest.rs"]
mod command_manifest;

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(command_manifest::PHASE_1_TO_3_COMMANDS),
    ))
    .expect("failed to build Tauri application");
}
