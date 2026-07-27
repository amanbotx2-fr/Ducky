use std::{path::PathBuf, sync::Arc};

use tauri::{
    webview::{PageLoadEvent, PageLoadPayload},
    App, Manager, Runtime, Webview,
};

use crate::{
    domain::{
        reminders::{ReminderFiredNotification, ReminderRuntime},
        settings::SettingsState,
    },
    events::{self, DesktopEvent},
    infrastructure::{credentials::CredentialStore, persistence::SettingsStore},
};

const SETTINGS_FILE_NAME: &str = "settings.json";

pub(crate) fn initialize<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);
    let legacy_path = legacy_electron_settings_path(app)?;
    let store = SettingsStore::new(settings_path);
    let settings = store.load_with_legacy(legacy_path.as_deref())?;

    let settings_state = SettingsState::new(store, settings);
    let app_handle = app.handle().clone();
    let reminder_runtime = ReminderRuntime::with_delivery(
        Arc::new(settings_state.clone()),
        Arc::new(move |notification: ReminderFiredNotification| {
            events::emit(&app_handle, DesktopEvent::ReminderFired, notification)
                .map_err(|error| error.to_string())
        }),
    );
    reminder_runtime.start()?;

    app.manage(CredentialStore::native());
    app.manage(settings_state);
    app.manage(reminder_runtime);
    Ok(())
}

pub(crate) fn handle_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if webview.label() != crate::authorization::COMPANION_LABEL
        || payload.event() != PageLoadEvent::Started
    {
        return;
    }

    if let Some(runtime) = webview.app_handle().try_state::<ReminderRuntime>() {
        runtime.pending_deliveries.deactivate();
    }
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
