#[allow(dead_code)]
#[path = "src/authorization.rs"]
mod authorization;

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(authorization::PHASE_1_TO_3_COMMAND_NAMES),
    ))
    .expect("failed to build Tauri application");
}
