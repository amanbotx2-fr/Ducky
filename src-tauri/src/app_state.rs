use std::sync::Arc;

use tauri::{
    webview::{PageLoadEvent, PageLoadPayload},
    App, Manager, Runtime, Webview,
};

use crate::{
    domain::{
        ai::{
            AiCancellationReason, AiProviderId, AiRendererRole, AiRequestManager, AiRuntime,
            AssistantActionProcessor, ClaudeProvider, CustomProvider, GeminiProvider, GrokProvider,
            OllamaProvider, OpenAiProvider,
        },
        personal_assistant::{PersonalAssistantEventQueue, PersonalAssistantPanel},
        pomodoro::{PomodoroEventQueue, PomodoroRuntime},
        reminders::{ReminderFiredNotification, ReminderRuntime},
        settings::SettingsState,
        updater::UpdaterRuntime,
    },
    events::{self, DesktopEvent},
    infrastructure::{
        credentials::CredentialStore, persistence::SettingsStore, pomodoro::PomodoroStore,
        updater::TauriUpdateBackend,
    },
};

const SETTINGS_FILE_NAME: &str = "settings.json";
const POMODORO_FILE_NAME: &str = "pomodoro.json";

pub(crate) fn initialize<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);
    let store = SettingsStore::new(settings_path);
    let settings = store.load()?;

    let settings_state = SettingsState::new(store, settings);
    let updater_event_app_handle = app.handle().clone();
    let updater_runtime = UpdaterRuntime::new(
        app.package_info().version.to_string(),
        !cfg!(debug_assertions),
        Arc::new(TauriUpdateBackend::new(app.handle().clone())),
        Arc::new(move |status| {
            if let Err(error) = events::emit(
                &updater_event_app_handle,
                DesktopEvent::UpdateStatusChanged,
                status,
            ) {
                eprintln!("[updates] status notification failed: {error}");
            }
        }),
    );
    let automatic_update_checks = settings_state.snapshot()?.updates.automatic;
    updater_runtime.set_automatic_checks_enabled(automatic_update_checks)?;
    if automatic_update_checks {
        let startup_updater = updater_runtime.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = startup_updater.check_automatically().await {
                eprintln!("[updates] automatic check failed: {error}");
            }
        });
    }
    let credential_store = CredentialStore::native();
    let ai_runtime = AiRuntime::new(credential_store.clone());
    let ai_requests = AiRequestManager::default();
    ai_runtime.ensure_running()?;
    ai_runtime.register_provider(Arc::new(OpenAiProvider::new()?))?;
    ai_runtime.register_provider(Arc::new(GeminiProvider::new()?))?;
    ai_runtime.register_provider(Arc::new(ClaudeProvider::new()?))?;
    ai_runtime.register_provider(Arc::new(GrokProvider::new()?))?;
    ai_runtime.register_provider(Arc::new(OllamaProvider::new()))?;
    ai_runtime.register_provider(Arc::new(CustomProvider::new()?))?;
    match settings_state.snapshot()?.ai.provider.as_str() {
        "openai" => ai_runtime.select_provider(AiProviderId::Openai)?,
        "gemini" => ai_runtime.select_provider(AiProviderId::Gemini)?,
        "claude" => ai_runtime.select_provider(AiProviderId::Claude)?,
        "grok" => ai_runtime.select_provider(AiProviderId::Grok)?,
        "ollama" => ai_runtime.select_provider(AiProviderId::Ollama)?,
        "custom" => ai_runtime.select_provider(AiProviderId::Custom)?,
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
    let settings_event_app_handle = app.handle().clone();
    let assistant_actions = AssistantActionProcessor::new(
        reminder_runtime.service.clone(),
        settings_state.clone(),
        Arc::new(move |settings| {
            events::emit(
                &settings_event_app_handle,
                DesktopEvent::RuntimeSettingsChanged,
                settings,
            )
            .map_err(|error| error.to_string())
        }),
    );
    let panel_event_app_handle = app.handle().clone();
    let personal_assistant_events =
        PersonalAssistantEventQueue::with_emitter(Arc::new(move |panel| {
            let event = match panel {
                PersonalAssistantPanel::UserName => DesktopEvent::UserNamePanelRequested,
                PersonalAssistantPanel::StickyMessage => DesktopEvent::StickyMessagePanelRequested,
                PersonalAssistantPanel::DailyPlanner => DesktopEvent::DailyPlannerPanelRequested,
            };

            events::emit(&panel_event_app_handle, event, ()).map_err(|error| error.to_string())
        }));

    let pomodoro_path = app.path().app_data_dir()?.join(POMODORO_FILE_NAME);
    let pomodoro_store = PomodoroStore::new(pomodoro_path);
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
    app.manage(ai_requests);
    app.manage(assistant_actions);
    app.manage(personal_assistant_events);
    app.manage(settings_state);
    app.manage(updater_runtime);
    app.manage(reminder_runtime);
    app.manage(pomodoro_events);
    app.manage(pomodoro_runtime);
    Ok(())
}

pub(crate) fn handle_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if payload.event() != PageLoadEvent::Started {
        return;
    }

    if let Some(requests) = webview.app_handle().try_state::<AiRequestManager>() {
        let role = match webview.label() {
            crate::authorization::COMPANION_LABEL => Some(AiRendererRole::Companion),
            crate::authorization::PREFERENCES_LABEL => Some(AiRendererRole::Preferences),
            _ => None,
        };
        if let Some(role) = role {
            requests.cancel_role(role, AiCancellationReason::RendererReloaded);
        }
    }

    if webview.label() != crate::authorization::COMPANION_LABEL {
        return;
    }
    if let Some(runtime) = webview.app_handle().try_state::<ReminderRuntime>() {
        runtime.pending_deliveries.deactivate();
    }
    if let Some(events) = webview.app_handle().try_state::<PomodoroEventQueue>() {
        events.deactivate();
    }
    if let Some(events) = webview
        .app_handle()
        .try_state::<PersonalAssistantEventQueue>()
    {
        events.deactivate();
    }
}
