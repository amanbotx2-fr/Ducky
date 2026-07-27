use tauri::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, SubmenuBuilder},
    App, AppHandle, Runtime,
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
enum TrayMenuEntry {
    Action {
        id: &'static str,
        label: &'static str,
    },
    About,
    Separator,
}

const TRAY_MENU_ENTRIES: [TrayMenuEntry; 6] = [
    TrayMenuEntry::Action {
        id: SHOW_COMPANION_ID,
        label: "Show Ducky",
    },
    TrayMenuEntry::Action {
        id: SHOW_PREFERENCES_ID,
        label: "Preferences…",
    },
    TrayMenuEntry::About,
    TrayMenuEntry::Separator,
    TrayMenuEntry::Action {
        id: RESTART_ID,
        label: "Restart",
    },
    TrayMenuEntry::Action {
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
    let menu = Menu::new(app)?;

    for entry in TRAY_MENU_ENTRIES {
        match entry {
            TrayMenuEntry::Action { id, label } => {
                menu.append(&MenuItem::with_id(app, id, label, true, None::<&str>)?)?;
            }
            TrayMenuEntry::About => {
                menu.append(&PredefinedMenuItem::about(
                    app,
                    Some("About Ducky"),
                    Some(about_metadata(app)),
                )?)?;
            }
            TrayMenuEntry::Separator => {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
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

fn about_metadata<R: Runtime>(app: &App<R>) -> AboutMetadata<'static> {
    AboutMetadata {
        name: Some(APP_NAME.to_string()),
        version: Some(app.package_info().version.to_string()),
        comments: Some(APP_DESCRIPTION.to_string()),
        copyright: Some(APP_COPYRIGHT.to_string()),
        credits: Some("Built with Tauri".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        action_for_id, NativeMenuAction, TrayMenuEntry, QUIT_ID, RESTART_ID, SHOW_COMPANION_ID,
        SHOW_PREFERENCES_ID, TRAY_MENU_ENTRIES,
    };

    #[test]
    fn static_tray_menu_matches_electron_order_and_labels() {
        assert_eq!(
            TRAY_MENU_ENTRIES,
            [
                TrayMenuEntry::Action {
                    id: SHOW_COMPANION_ID,
                    label: "Show Ducky",
                },
                TrayMenuEntry::Action {
                    id: SHOW_PREFERENCES_ID,
                    label: "Preferences…",
                },
                TrayMenuEntry::About,
                TrayMenuEntry::Separator,
                TrayMenuEntry::Action {
                    id: RESTART_ID,
                    label: "Restart",
                },
                TrayMenuEntry::Action {
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
