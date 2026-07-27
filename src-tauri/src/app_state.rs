use std::{path::PathBuf, sync::Arc};

use tauri::{
    webview::{PageLoadEvent, PageLoadPayload},
    App, Manager, Runtime, Webview,
};

use crate::{
    domain::{
        ai::{AiProviderId, AiRuntime, GeminiProvider, OpenAiProvider},
        pomodoro::{PomodoroEventQueue, PomodoroRuntime},
        reminders::{ReminderFiredNotification, ReminderRuntime},
        settings::SettingsState,
    },
    events::{self, DesktopEvent},
    infrastructure::{
        credentials::CredentialStore, persistence::SettingsStore, pomodoro::PomodoroStore,
    },
};

const SETTINGS_FILE_NAME: &str = "settings.json";
const POMODORO_FILE_NAME: &str = "pomodoro.json";

pub(crate) fn initialize<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);
    let legacy_settings_path = legacy_electron_data_path(app, SETTINGS_FILE_NAME)?;
    let store = SettingsStore::new(settings_path);
    let settings = store.load_with_legacy(legacy_settings_path.as_deref())?;

    let settings_state = SettingsState::new(store, settings);
    let credential_store = CredentialStore::native();
    let ai_runtime = AiRuntime::new(credential_store.clone());
    ai_runtime.ensure_running()?;
    ai_runtime.register_provider(Arc::new(OpenAiProvider::new()?))?;
    ai_runtime.register_provider(Arc::new(GeminiProvider::new()?))?;
    match settings_state.snapshot()?.ai.provider.as_str() {
        "openai" => ai_runtime.select_provider(AiProviderId::Openai)?,
        "gemini" => ai_runtime.select_provider(AiProviderId::Gemini)?,
        _ => {}
    }
    let app_handle = app.handle().clone();
    let reminder_runtime = ReminderRuntime::with_delivery(
        Arc::new(settings_state.clone()),
        Arc::new(move |notification: ReminderFiredNotification| {
            events::emit(&app_handle, DesktopEvent::ReminderFired, notification)
                .map_err(|error| error.to_string())
        }),
    );
    reminder_runtime.start()?;

    let pomodoro_path = app.path().app_data_dir()?.join(POMODORO_FILE_NAME);
    let legacy_pomodoro_path = legacy_electron_data_path(app, POMODORO_FILE_NAME)?;
    let pomodoro_store = PomodoroStore::new(pomodoro_path);
    pomodoro_store.import_legacy_if_missing(legacy_pomodoro_path.as_deref())?;
    let state_app_handle = app.handle().clone();
    let completion_app_handle = app.handle().clone();
    let custom_panel_app_handle = app.handle().clone();
    let pomodoro_events = PomodoroEventQueue::with_emitters(
        Arc::new(move |state| {
            events::emit(&state_app_handle, DesktopEvent::PomodoroStateChanged, state)
                .map_err(|error| error.to_string())
        }),
        Arc::new(move || {
            events::emit(&completion_app_handle, DesktopEvent::PomodoroCompleted, ())
                .map_err(|error| error.to_string())
        }),
        Arc::new(move || {
            events::emit(
                &custom_panel_app_handle,
                DesktopEvent::CustomPomodoroDurationRequested,
                (),
            )
            .map_err(|error| error.to_string())
        }),
    );
    let pomodoro_runtime =
        PomodoroRuntime::new(Arc::new(pomodoro_store), Arc::new(pomodoro_events.clone()));
    pomodoro_runtime.start()?;

    app.manage(credential_store);
    app.manage(ai_runtime);
    app.manage(settings_state);
    app.manage(reminder_runtime);
    app.manage(pomodoro_events);
    app.manage(pomodoro_runtime);
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
    if let Some(events) = webview.app_handle().try_state::<PomodoroEventQueue>() {
        events.deactivate();
    }
}

fn legacy_electron_data_path<R: Runtime>(
    app: &App<R>,
    file_name: &str,
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
    let candidate = directory.join(file_name);
    let native = app.path().app_data_dir()?.join(file_name);

    Ok((candidate != native).then_some(candidate))
}
