use std::path::PathBuf;

use tauri::{App, Manager, Runtime};

use crate::{
    domain::settings::SettingsState,
    infrastructure::{credentials::CredentialStore, persistence::SettingsStore},
};

const SETTINGS_FILE_NAME: &str = "settings.json";

pub(crate) fn initialize<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);
    let legacy_path = legacy_electron_settings_path(app)?;
    let store = SettingsStore::new(settings_path);
    let settings = store.load_with_legacy(legacy_path.as_deref())?;

    app.manage(CredentialStore::native());
    app.manage(SettingsState::new(store, settings));
    Ok(())
}

fn legacy_electron_settings_path<R: Runtime>(
    app: &App<R>,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let directory = if cfg!(target_os = "macos") {
        app.path()
            .home_dir()?
            .join("Library")
            .join("Application Support")
            .join("Ducky")
    } else {
        app.path().config_dir()?.join("Ducky")
    };
    let candidate = directory.join(SETTINGS_FILE_NAME);
    let native = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);

    Ok((candidate != native).then_some(candidate))
}
