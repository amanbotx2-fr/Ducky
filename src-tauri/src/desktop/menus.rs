use tauri::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, SubmenuBuilder},
    App, AppHandle, Manager, Runtime, WebviewWindow,
};

use super::windows::{companion, preferences};

pub const SHOW_COMPANION_ID: &str = "ducky.show-companion";
pub const SHOW_PREFERENCES_ID: &str = "ducky.show-preferences";
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMenuAction {
    ShowCompanion,
    ShowPreferences,
    Restart,
    Quit,
}

pub fn create_tray_menu<R: Runtime>(app: &App<R>) -> tauri::Result<Menu<R>> {
    create_static_menu(app, &TRAY_MENU_ENTRIES)
}

pub fn show_companion_context_menu<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let menu = create_static_menu(window, &COMPANION_CONTEXT_MENU_ENTRIES)?;
    window.popup_menu(&menu)
}

fn create_static_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    entries: &[StaticMenuEntry],
) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(manager)?;

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

    Ok(menu)
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
        NativeMenuAction::Restart => {
            app.request_restart();
        }
        NativeMenuAction::Quit => {
            app.exit(0);
        }
    }

    Ok(())
}

fn action_for_id(id: &str) -> Option<NativeMenuAction> {
    match id {
        SHOW_COMPANION_ID => Some(NativeMenuAction::ShowCompanion),
        SHOW_PREFERENCES_ID => Some(NativeMenuAction::ShowPreferences),
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
        action_for_id, NativeMenuAction, StaticMenuEntry, COMPANION_CONTEXT_MENU_ENTRIES, QUIT_ID,
        RESTART_ID, SHOW_COMPANION_ID, SHOW_PREFERENCES_ID, TRAY_MENU_ENTRIES,
    };

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
    fn companion_context_menu_contains_only_the_migrated_static_slice() {
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
        let ids = [SHOW_COMPANION_ID, SHOW_PREFERENCES_ID, RESTART_ID, QUIT_ID];
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
        assert_eq!(action_for_id(RESTART_ID), Some(NativeMenuAction::Restart));
        assert_eq!(action_for_id(QUIT_ID), Some(NativeMenuAction::Quit));
        assert_eq!(action_for_id("ducky.settings"), None);
        assert_eq!(action_for_id(""), None);
    }
}
