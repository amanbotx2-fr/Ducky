use serde::Serialize;
use tauri::{AppHandle, Emitter, EventTarget, Runtime};

use crate::desktop::windows::companion;

const PREFERENCES_LABEL: &str = "preferences";

const COMPANION_ONLY: &[RendererTarget] = &[RendererTarget::Companion];
const PREFERENCES_ONLY: &[RendererTarget] = &[RendererTarget::Preferences];
const BOTH_RENDERERS: &[RendererTarget] = &[RendererTarget::Companion, RendererTarget::Preferences];

/// Exact renderer destinations supported by Ducky's desktop event bridge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum RendererTarget {
    Companion,
    Preferences,
}

impl RendererTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Companion => companion::LABEL,
            Self::Preferences => PREFERENCES_LABEL,
        }
    }
}

/// Low-frequency backend events from the existing Electron IPC contract.
///
/// This registry contains routing metadata only. The owning feature phases
/// remain responsible for producing validated payloads and recovery
/// snapshots. Cursor samples are intentionally excluded because Task 3.3
/// retains their ordered Tauri channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DesktopEvent {
    RuntimeSettingsChanged,
    UserNamePanelRequested,
    StickyMessagePanelRequested,
    ReminderCreationPanelRequested,
    ReminderManagerPanelRequested,
    DailyPlannerPanelRequested,
    ReminderFired,
    UpdateStatusChanged,
    CustomPomodoroDurationRequested,
    PomodoroStateChanged,
    PomodoroCompleted,
}

pub(crate) const LOW_FREQUENCY_EVENTS: &[DesktopEvent] = &[
    DesktopEvent::RuntimeSettingsChanged,
    DesktopEvent::UserNamePanelRequested,
    DesktopEvent::StickyMessagePanelRequested,
    DesktopEvent::ReminderCreationPanelRequested,
    DesktopEvent::ReminderManagerPanelRequested,
    DesktopEvent::DailyPlannerPanelRequested,
    DesktopEvent::ReminderFired,
    DesktopEvent::UpdateStatusChanged,
    DesktopEvent::CustomPomodoroDurationRequested,
    DesktopEvent::PomodoroStateChanged,
    DesktopEvent::PomodoroCompleted,
];

impl DesktopEvent {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::RuntimeSettingsChanged => "runtime-settings:changed",
            Self::UserNamePanelRequested => "personal-assistant:user-name-requested",
            Self::StickyMessagePanelRequested => "personal-assistant:sticky-message-requested",
            Self::ReminderCreationPanelRequested => "reminders:creation-panel-requested",
            Self::ReminderManagerPanelRequested => "reminders:manager-panel-requested",
            Self::DailyPlannerPanelRequested => "daily-planner:panel-requested",
            Self::ReminderFired => "reminders:fired",
            Self::UpdateStatusChanged => "updates:status-changed",
            Self::CustomPomodoroDurationRequested => "pomodoro:custom-duration-requested",
            Self::PomodoroStateChanged => "pomodoro:state-changed",
            Self::PomodoroCompleted => "pomodoro:completed",
        }
    }

    pub(crate) fn targets(self) -> &'static [RendererTarget] {
        match self {
            Self::RuntimeSettingsChanged => BOTH_RENDERERS,
            Self::UpdateStatusChanged => PREFERENCES_ONLY,
            Self::UserNamePanelRequested
            | Self::StickyMessagePanelRequested
            | Self::ReminderCreationPanelRequested
            | Self::ReminderManagerPanelRequested
            | Self::DailyPlannerPanelRequested
            | Self::ReminderFired
            | Self::CustomPomodoroDurationRequested
            | Self::PomodoroStateChanged
            | Self::PomodoroCompleted => COMPANION_ONLY,
        }
    }
}

/// Emits one immutable, serializable payload only to the event's approved
/// WebviewWindow labels.
///
/// Feature phases call this after their authoritative state mutation succeeds;
/// renderers never receive a global broadcast.
pub(crate) fn emit<R, Payload>(
    app: &AppHandle<R>,
    event: DesktopEvent,
    payload: Payload,
) -> tauri::Result<()>
where
    R: Runtime,
    Payload: Clone + Serialize,
{
    for target in event.targets() {
        app.emit_to(
            EventTarget::webview_window(target.label()),
            event.name(),
            payload.clone(),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{DesktopEvent, RendererTarget, LOW_FREQUENCY_EVENTS};

    #[test]
    fn event_registry_matches_the_existing_low_frequency_contract() {
        let names = LOW_FREQUENCY_EVENTS
            .iter()
            .map(|event| event.name())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "runtime-settings:changed",
                "personal-assistant:user-name-requested",
                "personal-assistant:sticky-message-requested",
                "reminders:creation-panel-requested",
                "reminders:manager-panel-requested",
                "daily-planner:panel-requested",
                "reminders:fired",
                "updates:status-changed",
                "pomodoro:custom-duration-requested",
                "pomodoro:state-changed",
                "pomodoro:completed",
            ],
        );
        assert_eq!(
            names.iter().copied().collect::<HashSet<_>>().len(),
            names.len(),
        );
        assert!(!names.contains(&"psyduck:cursor-position"));
    }

    #[test]
    fn events_are_routed_only_to_their_existing_renderer_roles() {
        assert_eq!(
            DesktopEvent::RuntimeSettingsChanged.targets(),
            [RendererTarget::Companion, RendererTarget::Preferences,],
        );
        assert_eq!(
            DesktopEvent::UpdateStatusChanged.targets(),
            [RendererTarget::Preferences],
        );

        for event in LOW_FREQUENCY_EVENTS {
            if matches!(
                event,
                DesktopEvent::RuntimeSettingsChanged | DesktopEvent::UpdateStatusChanged
            ) {
                continue;
            }

            assert_eq!(event.targets(), [RendererTarget::Companion]);
        }
    }
}
