use tauri::{
    menu::{
        AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
        SubmenuBuilder,
    },
    App, AppHandle, Manager, Runtime, WebviewWindow,
};

use crate::{
    commands,
    domain::{
        personal_assistant::{PersonalAssistantEventQueue, PersonalAssistantPanel},
        pomodoro::{PomodoroEventQueue, PomodoroRuntime, PomodoroState},
        settings::{
            GeneralSettingsPatch, PreferencesSettingsPatch, SettingsDocument, SettingsState,
            WaterSettingsPatch,
        },
    },
    events::{self, DesktopEvent},
};

use super::windows::{companion, preferences};

pub const SHOW_COMPANION_ID: &str = "ducky.show-companion";
pub const SHOW_PREFERENCES_ID: &str = "ducky.show-preferences";
pub const NEW_REMINDER_ID: &str = "ducky.reminders.new";
pub const MANAGE_REMINDERS_ID: &str = "ducky.reminders.manage";
pub const SET_USER_NAME_ID: &str = "ducky.personal-assistant.set-user-name";
pub const DAILY_PLANNER_ID: &str = "ducky.personal-assistant.daily-planner";
pub const SET_STICKY_MESSAGE_ID: &str = "ducky.personal-assistant.set-sticky-message";
pub const CLEAR_STICKY_MESSAGE_ID: &str = "ducky.personal-assistant.clear-sticky-message";
pub const WATER_ENABLED_ID: &str = "ducky.water.enabled";
pub const WATER_INTERVAL_15_ID: &str = "ducky.water.interval.15";
pub const WATER_INTERVAL_30_ID: &str = "ducky.water.interval.30";
pub const WATER_INTERVAL_45_ID: &str = "ducky.water.interval.45";
pub const WATER_INTERVAL_60_ID: &str = "ducky.water.interval.60";
pub const WATER_INTERVAL_90_ID: &str = "ducky.water.interval.90";
pub const WATER_INTERVAL_120_ID: &str = "ducky.water.interval.120";
pub const EYE_TRACKING_ID: &str = "ducky.general.eye-tracking";
pub const ALWAYS_ON_TOP_ID: &str = "ducky.general.always-on-top";
pub const POMODORO_25_ID: &str = "ducky.pomodoro.25";
pub const POMODORO_50_ID: &str = "ducky.pomodoro.50";
pub const POMODORO_90_ID: &str = "ducky.pomodoro.90";
pub const POMODORO_CUSTOM_ID: &str = "ducky.pomodoro.custom";
pub const POMODORO_PAUSE_ID: &str = "ducky.pomodoro.pause";
pub const POMODORO_RESUME_ID: &str = "ducky.pomodoro.resume";
pub const POMODORO_STOP_ID: &str = "ducky.pomodoro.stop";
pub const RESTART_ID: &str = "ducky.restart";
pub const QUIT_ID: &str = "ducky.quit";

const APP_NAME: &str = "Ducky";
const APP_DESCRIPTION: &str = "Desktop AI Companion";
const APP_COPYRIGHT: &str = "Copyright © 2026 Aman";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StaticMenuEntry {
    Action {
        id: &'static str,
        label: &'static str,
    },
    About,
    Separator,
}

const TRAY_MENU_ENTRIES: [StaticMenuEntry; 6] = [
    StaticMenuEntry::Action {
        id: SHOW_COMPANION_ID,
        label: "Show Ducky",
    },
    StaticMenuEntry::Action {
        id: SHOW_PREFERENCES_ID,
        label: "Preferences…",
    },
    StaticMenuEntry::About,
    StaticMenuEntry::Separator,
    StaticMenuEntry::Action {
        id: RESTART_ID,
        label: "Restart",
    },
    StaticMenuEntry::Action {
        id: QUIT_ID,
        label: "Quit",
    },
];

const COMPANION_CONTEXT_MENU_ENTRIES: [StaticMenuEntry; 5] = [
    StaticMenuEntry::Action {
        id: SHOW_PREFERENCES_ID,
        label: "Preferences…",
    },
    StaticMenuEntry::About,
    StaticMenuEntry::Separator,
    StaticMenuEntry::Action {
        id: RESTART_ID,
        label: "Restart",
    },
    StaticMenuEntry::Action {
        id: QUIT_ID,
        label: "Quit",
    },
];

const REMINDER_CONTEXT_MENU_ENTRIES: [StaticMenuEntry; 2] = [
    StaticMenuEntry::Action {
        id: NEW_REMINDER_ID,
        label: "New Reminder…",
    },
    StaticMenuEntry::Action {
        id: MANAGE_REMINDERS_ID,
        label: "Manage Reminders…",
    },
];

const WATER_INTERVAL_MENU_ENTRIES: [(&str, u16); 6] = [
    (WATER_INTERVAL_15_ID, 15),
    (WATER_INTERVAL_30_ID, 30),
    (WATER_INTERVAL_45_ID, 45),
    (WATER_INTERVAL_60_ID, 60),
    (WATER_INTERVAL_90_ID, 90),
    (WATER_INTERVAL_120_ID, 120),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMenuAction {
    ShowCompanion,
    ShowPreferences,
    NewReminder,
    ManageReminders,
    RequestUserName,
    RequestDailyPlanner,
    RequestStickyMessage,
    ClearStickyMessage,
    ToggleWaterReminders,
    SetWaterInterval(u16),
    ToggleEyeTracking,
    ToggleAlwaysOnTop,
    StartPomodoro(u32),
    RequestCustomPomodoroDuration,
    PausePomodoro,
    ResumePomodoro,
    StopPomodoro,
    Restart,
    Quit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PomodoroMenuPresentation {
    checked_preset_id: Option<&'static str>,
    custom_checked: bool,
    pause_enabled: bool,
    resume_enabled: bool,
    stop_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SettingsMenuPresentation {
    water_enabled: bool,
    water_interval: u16,
    eye_tracking: bool,
    always_on_top: bool,
    clear_sticky_message_enabled: bool,
}

pub fn create_tray_menu<R: Runtime>(app: &App<R>) -> tauri::Result<Menu<R>> {
    create_static_menu(app, &TRAY_MENU_ENTRIES)
}

pub fn show_companion_context_menu<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let state = window
        .state::<PomodoroRuntime>()
        .state()
        .map_err(native_runtime_error)?;
    let settings = window
        .state::<SettingsState>()
        .snapshot()
        .map_err(native_runtime_error)?;
    let menu = create_companion_context_menu(window, &state, &settings)?;
    window.popup_menu(&menu)
}

fn create_companion_context_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    pomodoro_state: &PomodoroState,
    settings: &SettingsDocument,
) -> tauri::Result<Menu<R>> {
    let pomodoro_menu = create_pomodoro_menu(manager, pomodoro_state)?;
    let settings_presentation = settings_menu_presentation(settings);
    let reminder_menu = create_submenu(manager, "Reminders", &REMINDER_CONTEXT_MENU_ENTRIES)?;
    let sticky_message_menu = Submenu::new(manager, "Sticky Message", true)?;
    sticky_message_menu.append(&MenuItem::with_id(
        manager,
        SET_STICKY_MESSAGE_ID,
        "Set Sticky Message…",
        true,
        None::<&str>,
    )?)?;
    sticky_message_menu.append(&MenuItem::with_id(
        manager,
        CLEAR_STICKY_MESSAGE_ID,
        "Clear Sticky Message",
        settings_presentation.clear_sticky_message_enabled,
        None::<&str>,
    )?)?;
    let personal_assistant_menu = SubmenuBuilder::new(manager, "Personal Assistant")
        .item(&MenuItem::with_id(
            manager,
            SET_USER_NAME_ID,
            "Set My Name…",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&reminder_menu)
        .item(&MenuItem::with_id(
            manager,
            DAILY_PLANNER_ID,
            "Daily Planner…",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&sticky_message_menu)
        .build()?;
    let water_interval_menu = Submenu::new(manager, "Reminder Interval", true)?;
    for (id, interval) in WATER_INTERVAL_MENU_ENTRIES {
        water_interval_menu.append(&CheckMenuItem::with_id(
            manager,
            id,
            format!("{interval} min"),
            true,
            settings_presentation.water_interval == interval,
            None::<&str>,
        )?)?;
    }
    let water_menu = Submenu::new(manager, "Water Reminders", true)?;
    water_menu.append(&CheckMenuItem::with_id(
        manager,
        WATER_ENABLED_ID,
        "Enabled",
        true,
        settings_presentation.water_enabled,
        None::<&str>,
    )?)?;
    water_menu.append(&water_interval_menu)?;
    let menu = Menu::new(manager)?;

    menu.append(&pomodoro_menu)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    menu.append(&personal_assistant_menu)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    menu.append(&water_menu)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    menu.append(&CheckMenuItem::with_id(
        manager,
        EYE_TRACKING_ID,
        "Eye Tracking",
        true,
        settings_presentation.eye_tracking,
        None::<&str>,
    )?)?;
    menu.append(&CheckMenuItem::with_id(
        manager,
        ALWAYS_ON_TOP_ID,
        "Always On Top",
        true,
        settings_presentation.always_on_top,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    append_static_entries(manager, &menu, &COMPANION_CONTEXT_MENU_ENTRIES)?;

    Ok(menu)
}

fn create_pomodoro_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    state: &PomodoroState,
) -> tauri::Result<Submenu<R>> {
    let submenu = Submenu::new(manager, "Pomodoro", true)?;
    let presentation = pomodoro_menu_presentation(state);
    let presets = [
        (POMODORO_25_ID, 25_u32),
        (POMODORO_50_ID, 50_u32),
        (POMODORO_90_ID, 90_u32),
    ];

    for (id, duration) in presets {
        submenu.append(&CheckMenuItem::with_id(
            manager,
            id,
            format!("{duration} min"),
            true,
            presentation.checked_preset_id == Some(id),
            None::<&str>,
        )?)?;
    }

    submenu.append(&PredefinedMenuItem::separator(manager)?)?;
    submenu.append(&CheckMenuItem::with_id(
        manager,
        POMODORO_CUSTOM_ID,
        "Custom…",
        true,
        presentation.custom_checked,
        None::<&str>,
    )?)?;
    submenu.append(&PredefinedMenuItem::separator(manager)?)?;
    submenu.append(&MenuItem::with_id(
        manager,
        POMODORO_PAUSE_ID,
        "Pause",
        presentation.pause_enabled,
        None::<&str>,
    )?)?;
    submenu.append(&MenuItem::with_id(
        manager,
        POMODORO_RESUME_ID,
        "Resume",
        presentation.resume_enabled,
        None::<&str>,
    )?)?;
    submenu.append(&MenuItem::with_id(
        manager,
        POMODORO_STOP_ID,
        "Stop",
        presentation.stop_enabled,
        None::<&str>,
    )?)?;

    Ok(submenu)
}

fn pomodoro_menu_presentation(state: &PomodoroState) -> PomodoroMenuPresentation {
    let checked_preset_id = match state.selected_duration_minutes {
        25 => Some(POMODORO_25_ID),
        50 => Some(POMODORO_50_ID),
        90 => Some(POMODORO_90_ID),
        _ => None,
    };

    PomodoroMenuPresentation {
        checked_preset_id,
        custom_checked: checked_preset_id.is_none(),
        pause_enabled: state.running && !state.paused,
        resume_enabled: state.running && state.paused,
        stop_enabled: state.running,
    }
}

fn settings_menu_presentation(settings: &SettingsDocument) -> SettingsMenuPresentation {
    SettingsMenuPresentation {
        water_enabled: settings.water.enabled,
        water_interval: settings.water.interval,
        eye_tracking: settings.general.eye_tracking,
        always_on_top: settings.general.always_on_top,
        clear_sticky_message_enabled: settings.sticky_message.is_some(),
    }
}

fn create_static_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    entries: &[StaticMenuEntry],
) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(manager)?;
    append_static_entries(manager, &menu, entries)?;
    Ok(menu)
}

fn append_static_entries<R: Runtime, M: Manager<R>>(
    manager: &M,
    menu: &Menu<R>,
    entries: &[StaticMenuEntry],
) -> tauri::Result<()> {
    for entry in entries {
        match entry {
            StaticMenuEntry::Action { id, label } => {
                menu.append(&MenuItem::with_id(
                    manager,
                    *id,
                    *label,
                    true,
                    None::<&str>,
                )?)?;
            }
            StaticMenuEntry::About => {
                menu.append(&PredefinedMenuItem::about(
                    manager,
                    Some("About Ducky"),
                    Some(about_metadata(manager)),
                )?)?;
            }
            StaticMenuEntry::Separator => {
                menu.append(&PredefinedMenuItem::separator(manager)?)?;
            }
        }
    }

    Ok(())
}

fn create_submenu<R: Runtime, M: Manager<R>>(
    manager: &M,
    label: &str,
    entries: &[StaticMenuEntry],
) -> tauri::Result<tauri::menu::Submenu<R>> {
    let submenu = tauri::menu::Submenu::new(manager, label, true)?;

    for entry in entries {
        let StaticMenuEntry::Action { id, label } = entry else {
            continue;
        };

        submenu.append(&MenuItem::with_id(
            manager,
            *id,
            *label,
            true,
            None::<&str>,
        )?)?;
    }

    Ok(submenu)
}

#[cfg(target_os = "macos")]
pub fn install_application_menu<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let application_menu = SubmenuBuilder::new(app, APP_NAME)
        .about_with_text("About Ducky", Some(about_metadata(app)))
        .separator()
        .services()
        .separator()
        .hide_with_text("Hide Ducky")
        .hide_others()
        .show_all()
        .separator()
        .quit_with_text("Quit Ducky")
        .build()?;
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let menu = Menu::with_items(app, &[&application_menu, &edit_menu])?;

    app.set_menu(menu)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn install_application_menu<R: Runtime>(_app: &App<R>) -> tauri::Result<()> {
    Ok(())
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) -> tauri::Result<()> {
    let Some(action) = action_for_id(event.id().as_ref()) else {
        return Ok(());
    };

    match action {
        NativeMenuAction::ShowCompanion => {
            companion::show(app)?;
        }
        NativeMenuAction::ShowPreferences => {
            preferences::show(app)?;
        }
        NativeMenuAction::NewReminder => {
            request_reminder_panel(app, DesktopEvent::ReminderCreationPanelRequested)?;
        }
        NativeMenuAction::ManageReminders => {
            request_reminder_panel(app, DesktopEvent::ReminderManagerPanelRequested)?;
        }
        NativeMenuAction::RequestUserName => {
            request_personal_assistant_panel(app, PersonalAssistantPanel::UserName)?;
        }
        NativeMenuAction::RequestDailyPlanner => {
            request_personal_assistant_panel(app, PersonalAssistantPanel::DailyPlanner)?;
        }
        NativeMenuAction::RequestStickyMessage => {
            request_personal_assistant_panel(app, PersonalAssistantPanel::StickyMessage)?;
        }
        NativeMenuAction::ClearStickyMessage => {
            let update = app
                .state::<SettingsState>()
                .update_sticky_message(None)
                .map_err(native_runtime_error)?;
            commands::settings::finish_update(app, update);
        }
        NativeMenuAction::ToggleWaterReminders => {
            let enabled = !app
                .state::<SettingsState>()
                .snapshot()
                .map_err(native_runtime_error)?
                .water
                .enabled;
            update_preferences(
                app,
                PreferencesSettingsPatch {
                    water: Some(WaterSettingsPatch {
                        enabled: Some(enabled),
                        ..WaterSettingsPatch::default()
                    }),
                    ..PreferencesSettingsPatch::default()
                },
            )?;
        }
        NativeMenuAction::SetWaterInterval(interval) => {
            update_preferences(
                app,
                PreferencesSettingsPatch {
                    water: Some(WaterSettingsPatch {
                        interval: Some(interval),
                        ..WaterSettingsPatch::default()
                    }),
                    ..PreferencesSettingsPatch::default()
                },
            )?;
        }
        NativeMenuAction::ToggleEyeTracking => {
            let eye_tracking = !app
                .state::<SettingsState>()
                .snapshot()
                .map_err(native_runtime_error)?
                .general
                .eye_tracking;
            update_preferences(
                app,
                PreferencesSettingsPatch {
                    general: Some(GeneralSettingsPatch {
                        eye_tracking: Some(eye_tracking),
                        ..GeneralSettingsPatch::default()
                    }),
                    ..PreferencesSettingsPatch::default()
                },
            )?;
        }
        NativeMenuAction::ToggleAlwaysOnTop => {
            let always_on_top = !app
                .state::<SettingsState>()
                .snapshot()
                .map_err(native_runtime_error)?
                .general
                .always_on_top;
            update_preferences(
                app,
                PreferencesSettingsPatch {
                    general: Some(GeneralSettingsPatch {
                        always_on_top: Some(always_on_top),
                        ..GeneralSettingsPatch::default()
                    }),
                    ..PreferencesSettingsPatch::default()
                },
            )?;
        }
        NativeMenuAction::StartPomodoro(duration_minutes) => {
            app.state::<PomodoroRuntime>()
                .start_session(duration_minutes)
                .map_err(native_runtime_error)?;
        }
        NativeMenuAction::RequestCustomPomodoroDuration => {
            companion::show(app)?;
            app.state::<PomodoroEventQueue>().request_custom_panel();
        }
        NativeMenuAction::PausePomodoro => {
            app.state::<PomodoroRuntime>()
                .pause()
                .map_err(native_runtime_error)?;
        }
        NativeMenuAction::ResumePomodoro => {
            app.state::<PomodoroRuntime>()
                .resume()
                .map_err(native_runtime_error)?;
        }
        NativeMenuAction::StopPomodoro => {
            app.state::<PomodoroRuntime>()
                .stop_session()
                .map_err(native_runtime_error)?;
        }
        NativeMenuAction::Restart => {
            app.request_restart();
        }
        NativeMenuAction::Quit => {
            app.exit(0);
        }
    }

    Ok(())
}

fn request_reminder_panel<R: Runtime>(
    app: &AppHandle<R>,
    event: DesktopEvent,
) -> tauri::Result<()> {
    companion::show(app)?;
    events::emit(app, event, ())
}

fn request_personal_assistant_panel<R: Runtime>(
    app: &AppHandle<R>,
    panel: PersonalAssistantPanel,
) -> tauri::Result<()> {
    companion::show(app)?;
    app.state::<PersonalAssistantEventQueue>().request(panel);
    Ok(())
}

fn update_preferences<R: Runtime>(
    app: &AppHandle<R>,
    patch: PreferencesSettingsPatch,
) -> tauri::Result<()> {
    let update = app
        .state::<SettingsState>()
        .update_preferences(patch)
        .map_err(native_runtime_error)?;
    commands::settings::finish_update(app, update);
    Ok(())
}

fn native_runtime_error(error: impl std::fmt::Display) -> tauri::Error {
    std::io::Error::other(error.to_string()).into()
}

fn action_for_id(id: &str) -> Option<NativeMenuAction> {
    match id {
        SHOW_COMPANION_ID => Some(NativeMenuAction::ShowCompanion),
        SHOW_PREFERENCES_ID => Some(NativeMenuAction::ShowPreferences),
        NEW_REMINDER_ID => Some(NativeMenuAction::NewReminder),
        MANAGE_REMINDERS_ID => Some(NativeMenuAction::ManageReminders),
        SET_USER_NAME_ID => Some(NativeMenuAction::RequestUserName),
        DAILY_PLANNER_ID => Some(NativeMenuAction::RequestDailyPlanner),
        SET_STICKY_MESSAGE_ID => Some(NativeMenuAction::RequestStickyMessage),
        CLEAR_STICKY_MESSAGE_ID => Some(NativeMenuAction::ClearStickyMessage),
        WATER_ENABLED_ID => Some(NativeMenuAction::ToggleWaterReminders),
        WATER_INTERVAL_15_ID => Some(NativeMenuAction::SetWaterInterval(15)),
        WATER_INTERVAL_30_ID => Some(NativeMenuAction::SetWaterInterval(30)),
        WATER_INTERVAL_45_ID => Some(NativeMenuAction::SetWaterInterval(45)),
        WATER_INTERVAL_60_ID => Some(NativeMenuAction::SetWaterInterval(60)),
        WATER_INTERVAL_90_ID => Some(NativeMenuAction::SetWaterInterval(90)),
        WATER_INTERVAL_120_ID => Some(NativeMenuAction::SetWaterInterval(120)),
        EYE_TRACKING_ID => Some(NativeMenuAction::ToggleEyeTracking),
        ALWAYS_ON_TOP_ID => Some(NativeMenuAction::ToggleAlwaysOnTop),
        POMODORO_25_ID => Some(NativeMenuAction::StartPomodoro(25)),
        POMODORO_50_ID => Some(NativeMenuAction::StartPomodoro(50)),
        POMODORO_90_ID => Some(NativeMenuAction::StartPomodoro(90)),
        POMODORO_CUSTOM_ID => Some(NativeMenuAction::RequestCustomPomodoroDuration),
        POMODORO_PAUSE_ID => Some(NativeMenuAction::PausePomodoro),
        POMODORO_RESUME_ID => Some(NativeMenuAction::ResumePomodoro),
        POMODORO_STOP_ID => Some(NativeMenuAction::StopPomodoro),
        RESTART_ID => Some(NativeMenuAction::Restart),
        QUIT_ID => Some(NativeMenuAction::Quit),
        _ => None,
    }
}

fn about_metadata<R: Runtime, M: Manager<R>>(manager: &M) -> AboutMetadata<'static> {
    AboutMetadata {
        name: Some(APP_NAME.to_string()),
        version: Some(manager.package_info().version.to_string()),
        comments: Some(APP_DESCRIPTION.to_string()),
        copyright: Some(APP_COPYRIGHT.to_string()),
        credits: Some("Built with Tauri".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        action_for_id, pomodoro_menu_presentation, settings_menu_presentation, NativeMenuAction,
        PomodoroMenuPresentation, SettingsMenuPresentation, StaticMenuEntry, ALWAYS_ON_TOP_ID,
        CLEAR_STICKY_MESSAGE_ID, COMPANION_CONTEXT_MENU_ENTRIES, DAILY_PLANNER_ID, EYE_TRACKING_ID,
        MANAGE_REMINDERS_ID, NEW_REMINDER_ID, POMODORO_25_ID, POMODORO_50_ID, POMODORO_90_ID,
        POMODORO_CUSTOM_ID, POMODORO_PAUSE_ID, POMODORO_RESUME_ID, POMODORO_STOP_ID, QUIT_ID,
        REMINDER_CONTEXT_MENU_ENTRIES, RESTART_ID, SET_STICKY_MESSAGE_ID, SET_USER_NAME_ID,
        SHOW_COMPANION_ID, SHOW_PREFERENCES_ID, TRAY_MENU_ENTRIES, WATER_ENABLED_ID,
        WATER_INTERVAL_120_ID, WATER_INTERVAL_15_ID, WATER_INTERVAL_30_ID, WATER_INTERVAL_45_ID,
        WATER_INTERVAL_60_ID, WATER_INTERVAL_90_ID, WATER_INTERVAL_MENU_ENTRIES,
    };
    use crate::domain::pomodoro::PomodoroState;
    use crate::domain::settings::SettingsDocument;

    #[test]
    fn static_tray_menu_matches_electron_order_and_labels() {
        assert_eq!(
            TRAY_MENU_ENTRIES,
            [
                StaticMenuEntry::Action {
                    id: SHOW_COMPANION_ID,
                    label: "Show Ducky",
                },
                StaticMenuEntry::Action {
                    id: SHOW_PREFERENCES_ID,
                    label: "Preferences…",
                },
                StaticMenuEntry::About,
                StaticMenuEntry::Separator,
                StaticMenuEntry::Action {
                    id: RESTART_ID,
                    label: "Restart",
                },
                StaticMenuEntry::Action {
                    id: QUIT_ID,
                    label: "Quit",
                },
            ]
        );
    }

    #[test]
    fn companion_context_menu_preserves_the_existing_reminder_submenu() {
        assert_eq!(
            REMINDER_CONTEXT_MENU_ENTRIES,
            [
                StaticMenuEntry::Action {
                    id: NEW_REMINDER_ID,
                    label: "New Reminder…",
                },
                StaticMenuEntry::Action {
                    id: MANAGE_REMINDERS_ID,
                    label: "Manage Reminders…",
                },
            ]
        );
        assert_eq!(
            COMPANION_CONTEXT_MENU_ENTRIES,
            [
                StaticMenuEntry::Action {
                    id: SHOW_PREFERENCES_ID,
                    label: "Preferences…",
                },
                StaticMenuEntry::About,
                StaticMenuEntry::Separator,
                StaticMenuEntry::Action {
                    id: RESTART_ID,
                    label: "Restart",
                },
                StaticMenuEntry::Action {
                    id: QUIT_ID,
                    label: "Quit",
                },
            ]
        );
    }

    #[test]
    fn native_action_ids_are_unique() {
        let ids = [
            SHOW_COMPANION_ID,
            SHOW_PREFERENCES_ID,
            NEW_REMINDER_ID,
            MANAGE_REMINDERS_ID,
            SET_USER_NAME_ID,
            DAILY_PLANNER_ID,
            SET_STICKY_MESSAGE_ID,
            CLEAR_STICKY_MESSAGE_ID,
            WATER_ENABLED_ID,
            WATER_INTERVAL_15_ID,
            WATER_INTERVAL_30_ID,
            WATER_INTERVAL_45_ID,
            WATER_INTERVAL_60_ID,
            WATER_INTERVAL_90_ID,
            WATER_INTERVAL_120_ID,
            EYE_TRACKING_ID,
            ALWAYS_ON_TOP_ID,
            POMODORO_25_ID,
            POMODORO_50_ID,
            POMODORO_90_ID,
            POMODORO_CUSTOM_ID,
            POMODORO_PAUSE_ID,
            POMODORO_RESUME_ID,
            POMODORO_STOP_ID,
            RESTART_ID,
            QUIT_ID,
        ];
        let unique = ids.into_iter().collect::<std::collections::HashSet<_>>();

        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn native_action_dispatch_is_closed_to_unknown_ids() {
        assert_eq!(
            action_for_id(SHOW_COMPANION_ID),
            Some(NativeMenuAction::ShowCompanion)
        );
        assert_eq!(
            action_for_id(SHOW_PREFERENCES_ID),
            Some(NativeMenuAction::ShowPreferences)
        );
        assert_eq!(
            action_for_id(NEW_REMINDER_ID),
            Some(NativeMenuAction::NewReminder)
        );
        assert_eq!(
            action_for_id(MANAGE_REMINDERS_ID),
            Some(NativeMenuAction::ManageReminders)
        );
        assert_eq!(
            action_for_id(SET_USER_NAME_ID),
            Some(NativeMenuAction::RequestUserName)
        );
        assert_eq!(
            action_for_id(DAILY_PLANNER_ID),
            Some(NativeMenuAction::RequestDailyPlanner)
        );
        assert_eq!(
            action_for_id(SET_STICKY_MESSAGE_ID),
            Some(NativeMenuAction::RequestStickyMessage)
        );
        assert_eq!(
            action_for_id(CLEAR_STICKY_MESSAGE_ID),
            Some(NativeMenuAction::ClearStickyMessage)
        );
        assert_eq!(
            action_for_id(WATER_ENABLED_ID),
            Some(NativeMenuAction::ToggleWaterReminders)
        );
        assert_eq!(
            action_for_id(WATER_INTERVAL_90_ID),
            Some(NativeMenuAction::SetWaterInterval(90))
        );
        assert_eq!(
            action_for_id(EYE_TRACKING_ID),
            Some(NativeMenuAction::ToggleEyeTracking)
        );
        assert_eq!(
            action_for_id(ALWAYS_ON_TOP_ID),
            Some(NativeMenuAction::ToggleAlwaysOnTop)
        );
        assert_eq!(
            action_for_id(POMODORO_25_ID),
            Some(NativeMenuAction::StartPomodoro(25))
        );
        assert_eq!(
            action_for_id(POMODORO_CUSTOM_ID),
            Some(NativeMenuAction::RequestCustomPomodoroDuration)
        );
        assert_eq!(
            action_for_id(POMODORO_PAUSE_ID),
            Some(NativeMenuAction::PausePomodoro)
        );
        assert_eq!(
            action_for_id(POMODORO_RESUME_ID),
            Some(NativeMenuAction::ResumePomodoro)
        );
        assert_eq!(
            action_for_id(POMODORO_STOP_ID),
            Some(NativeMenuAction::StopPomodoro)
        );
        assert_eq!(action_for_id(RESTART_ID), Some(NativeMenuAction::Restart));
        assert_eq!(action_for_id(QUIT_ID), Some(NativeMenuAction::Quit));
        assert_eq!(action_for_id("ducky.settings"), None);
        assert_eq!(action_for_id(""), None);
    }

    #[test]
    fn pomodoro_menu_state_matches_electron_for_idle_running_and_paused_sessions() {
        let idle = PomodoroState::default();
        assert_eq!(
            pomodoro_menu_presentation(&idle),
            PomodoroMenuPresentation {
                checked_preset_id: Some(POMODORO_25_ID),
                custom_checked: false,
                pause_enabled: false,
                resume_enabled: false,
                stop_enabled: false,
            }
        );

        let running_custom = PomodoroState {
            running: true,
            paused: false,
            selected_duration_minutes: 12,
            duration_minutes: 12,
            remaining_seconds: 720,
            started_at: Some(1_000),
        };
        assert_eq!(
            pomodoro_menu_presentation(&running_custom),
            PomodoroMenuPresentation {
                checked_preset_id: None,
                custom_checked: true,
                pause_enabled: true,
                resume_enabled: false,
                stop_enabled: true,
            }
        );

        let paused = PomodoroState {
            paused: true,
            started_at: Some(1_000),
            ..running_custom
        };
        assert_eq!(
            pomodoro_menu_presentation(&paused),
            PomodoroMenuPresentation {
                checked_preset_id: None,
                custom_checked: true,
                pause_enabled: false,
                resume_enabled: true,
                stop_enabled: true,
            }
        );
    }

    #[test]
    fn settings_menu_state_and_intervals_match_electron() {
        assert_eq!(
            WATER_INTERVAL_MENU_ENTRIES,
            [
                (WATER_INTERVAL_15_ID, 15),
                (WATER_INTERVAL_30_ID, 30),
                (WATER_INTERVAL_45_ID, 45),
                (WATER_INTERVAL_60_ID, 60),
                (WATER_INTERVAL_90_ID, 90),
                (WATER_INTERVAL_120_ID, 120),
            ]
        );

        let defaults = SettingsDocument::default();
        assert_eq!(
            settings_menu_presentation(&defaults),
            SettingsMenuPresentation {
                water_enabled: true,
                water_interval: 30,
                eye_tracking: true,
                always_on_top: true,
                clear_sticky_message_enabled: false,
            }
        );

        let mut changed = defaults;
        changed.water.enabled = false;
        changed.water.interval = 90;
        changed.general.eye_tracking = false;
        changed.general.always_on_top = false;
        changed.sticky_message = Some("Stay focused".to_owned());
        assert_eq!(
            settings_menu_presentation(&changed),
            SettingsMenuPresentation {
                water_enabled: false,
                water_interval: 90,
                eye_tracking: false,
                always_on_top: false,
                clear_sticky_message_enabled: true,
            }
        );
    }
}
