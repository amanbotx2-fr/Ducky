use tauri::{AppHandle, Manager, RunEvent, Runtime, WindowEvent};

use super::{menus, tray, windows::companion};
use crate::domain::{
    ai::{AiCancellationReason, AiRendererRole, AiRequestManager, AiRuntime},
    pomodoro::PomodoroRuntime,
    reminders::ReminderRuntime,
};

pub fn handle_run_event<R: Runtime>(app: &AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::ExitRequested { code, api, .. } if should_prevent_exit(code) => {
            api.prevent_exit();
        }
        RunEvent::Exit => {
            if let Some(requests) = app.try_state::<AiRequestManager>() {
                requests.cancel_all(AiCancellationReason::ApplicationQuit);
            }
            if let Some(ai) = app.try_state::<AiRuntime>() {
                if let Err(error) = ai.shutdown() {
                    eprintln!("[ai] shutdown_failed: {error}");
                }
            }
            if let Some(reminders) = app.try_state::<ReminderRuntime>() {
                if let Err(error) = reminders.stop() {
                    eprintln!("[reminder-scheduler] shutdown_failed: {error}");
                }
            }
            if let Some(pomodoro) = app.try_state::<PomodoroRuntime>() {
                if let Err(error) = pomodoro.stop() {
                    eprintln!("[pomodoro] shutdown_failed: {error}");
                }
            }
            tray::destroy(app);
        }
        RunEvent::Resumed => {
            if let Some(reminders) = app.try_state::<ReminderRuntime>() {
                reminders.resynchronize();
            }
        }
        RunEvent::MenuEvent(event) => {
            if let Err(error) = menus::handle_menu_event(app, &event) {
                eprintln!("[menu] native_action_failed: {error}");
            }
        }
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::Destroyed,
            ..
        } => {
            let role = match label.as_str() {
                crate::authorization::COMPANION_LABEL => Some(AiRendererRole::Companion),
                crate::authorization::PREFERENCES_LABEL => Some(AiRendererRole::Preferences),
                _ => None,
            };
            if let (Some(requests), Some(role)) = (app.try_state::<AiRequestManager>(), role) {
                requests.cancel_role(role, AiCancellationReason::WindowClosed);
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            if let Err(error) = companion::show(app) {
                eprintln!("[menu] companion_reopen_failed: {error}");
            }
        }
        _ => {}
    }
}

fn should_prevent_exit(code: Option<i32>) -> bool {
    // Closing the final window exits the event loop by default on Windows and
    // Linux. Ducky stays resident while its tray is alive, so preserve that
    // lifecycle. Explicit AppHandle::exit/request_restart calls carry an exit
    // code and are never prevented. macOS already keeps an application alive
    // after its final window closes, and native Quit must remain authoritative.
    !cfg!(target_os = "macos") && code.is_none()
}

#[cfg(test)]
mod tests {
    use super::should_prevent_exit;

    #[test]
    fn explicit_exit_is_never_prevented() {
        assert!(!should_prevent_exit(Some(0)));
        assert!(!should_prevent_exit(Some(1)));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn final_window_close_keeps_non_macos_tray_application_alive() {
        assert!(should_prevent_exit(None));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_native_quit_remains_authoritative() {
        assert!(!should_prevent_exit(None));
    }
}
