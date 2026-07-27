use tauri::{
    menu::{
        AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
        SubmenuBuilder,
    },
    App, AppHandle, Manager, Runtime, WebviewWindow,
};

use crate::{
    domain::pomodoro::{PomodoroEventQueue, PomodoroRuntime, PomodoroState},
    events::{self, DesktopEvent},
};

use super::windows::{companion, preferences};

pub const SHOW_COMPANION_ID: &str = "ducky.show-companion";
pub const SHOW_PREFERENCES_ID: &str = "ducky.show-preferences";
pub const NEW_REMINDER_ID: &str = "ducky.reminders.new";
pub const MANAGE_REMINDERS_ID: &str = "ducky.reminders.manage";
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMenuAction {
    ShowCompanion,
    ShowPreferences,
    NewReminder,
    ManageReminders,
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

pub fn create_tray_menu<R: Runtime>(app: &App<R>) -> tauri::Result<Menu<R>> {
    create_static_menu(app, &TRAY_MENU_ENTRIES)
}

pub fn show_companion_context_menu<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let state = window
        .state::<PomodoroRuntime>()
        .state()
        .map_err(native_runtime_error)?;
    let menu = create_companion_context_menu(window, &state)?;
    window.popup_menu(&menu)
}

fn create_companion_context_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    pomodoro_state: &PomodoroState,
) -> tauri::Result<Menu<R>> {
    let pomodoro_menu = create_pomodoro_menu(manager, pomodoro_state)?;
    let reminder_menu = create_submenu(manager, "Reminders", &REMINDER_CONTEXT_MENU_ENTRIES)?;
    let personal_assistant_menu = SubmenuBuilder::new(manager, "Personal Assistant")
        .item(&reminder_menu)
        .build()?;
    let menu = Menu::new(manager)?;

    menu.append(&pomodoro_menu)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    menu.append(&personal_assistant_menu)?;
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

fn native_runtime_error(error: impl std::fmt::Display) -> tauri::Error {
    std::io::Error::other(error.to_string()).into()
}

fn action_for_id(id: &str) -> Option<NativeMenuAction> {
    match id {
        SHOW_COMPANION_ID => Some(NativeMenuAction::ShowCompanion),
        SHOW_PREFERENCES_ID => Some(NativeMenuAction::ShowPreferences),
        NEW_REMINDER_ID => Some(NativeMenuAction::NewReminder),
        MANAGE_REMINDERS_ID => Some(NativeMenuAction::ManageReminders),
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
        action_for_id, pomodoro_menu_presentation, NativeMenuAction, PomodoroMenuPresentation,
        StaticMenuEntry, COMPANION_CONTEXT_MENU_ENTRIES, MANAGE_REMINDERS_ID, NEW_REMINDER_ID,
        POMODORO_25_ID, POMODORO_50_ID, POMODORO_90_ID, POMODORO_CUSTOM_ID, POMODORO_PAUSE_ID,
        POMODORO_RESUME_ID, POMODORO_STOP_ID, QUIT_ID, REMINDER_CONTEXT_MENU_ENTRIES, RESTART_ID,
        SHOW_COMPANION_ID, SHOW_PREFERENCES_ID, TRAY_MENU_ENTRIES,
    };
    use crate::domain::pomodoro::PomodoroState;

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
}
