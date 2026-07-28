import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { PomodoroState } from '../shared/pomodoro';
import type { ReminderFiredNotification } from '../shared/reminders';
import type { RuntimeSettings } from '../shared/settings';
import type { UpdateStatus } from '../shared/updates';

export type TauriRendererTarget = 'companion' | 'preferences';

interface TauriEventPayloads {
  readonly runtimeSettingsChanged: RuntimeSettings;
  readonly userNamePanelRequested: null;
  readonly stickyMessagePanelRequested: null;
  readonly reminderCreationPanelRequested: null;
  readonly reminderManagerPanelRequested: null;
  readonly dailyPlannerPanelRequested: null;
  readonly reminderFired: ReminderFiredNotification;
  readonly updateStatusChanged: UpdateStatus;
  readonly customPomodoroDurationRequested: null;
  readonly pomodoroStateChanged: PomodoroState;
  readonly pomodoroCompleted: null;
}

type TauriEventKey = keyof TauriEventPayloads;

interface TauriEventRoute {
  readonly name: string;
  readonly targets: readonly TauriRendererTarget[];
}

/**
 * Typed routing table for low-frequency Tauri events.
 *
 * Event names are part of the stable renderer contract. The ordered cursor
 * stream uses its dedicated channel transport.
 */
export const TAURI_EVENTS = Object.freeze({
  runtimeSettingsChanged: {
    name: 'runtime-settings:changed',
    targets: ['companion', 'preferences'],
  },
  userNamePanelRequested: {
    name: 'personal-assistant:user-name-requested',
    targets: ['companion'],
  },
  stickyMessagePanelRequested: {
    name: 'personal-assistant:sticky-message-requested',
    targets: ['companion'],
  },
  reminderCreationPanelRequested: {
    name: 'reminders:creation-panel-requested',
    targets: ['companion'],
  },
  reminderManagerPanelRequested: {
    name: 'reminders:manager-panel-requested',
    targets: ['companion'],
  },
  dailyPlannerPanelRequested: {
    name: 'daily-planner:panel-requested',
    targets: ['companion'],
  },
  reminderFired: {
    name: 'reminders:fired',
    targets: ['companion'],
  },
  updateStatusChanged: {
    name: 'updates:status-changed',
    targets: ['preferences'],
  },
  customPomodoroDurationRequested: {
    name: 'pomodoro:custom-duration-requested',
    targets: ['companion'],
  },
  pomodoroStateChanged: {
    name: 'pomodoro:state-changed',
    targets: ['companion'],
  },
  pomodoroCompleted: {
    name: 'pomodoro:completed',
    targets: ['companion'],
  },
} as const satisfies Record<TauriEventKey, TauriEventRoute>);

type TauriEventListener<Event extends TauriEventKey> = (
  payload: TauriEventPayloads[Event],
) => void;

/**
 * Registers one exact-label Tauri event listener behind DesktopBridge.
 *
 * The synchronous disposer remains safe when React unmounts before Tauri's
 * asynchronous registration resolves.
 */
export const subscribeToTauriEvent = <Event extends TauriEventKey>(
  target: TauriRendererTarget,
  event: Event,
  listener: TauriEventListener<Event>,
  onRegistered?: () => void | Promise<void>,
): (() => void) => {
  const route = TAURI_EVENTS[event];

  if (!route.targets.some((allowedTarget) => allowedTarget === target)) {
    throw new Error(
      `Tauri event "${route.name}" is unavailable to "${target}".`,
    );
  }

  let active = true;
  let unlisten: UnlistenFn | undefined;

  void listen<TauriEventPayloads[Event]>(
    route.name,
    (tauriEvent) => {
      listener(tauriEvent.payload);
    },
    {
      target: {
        kind: 'WebviewWindow',
        label: target,
      },
    },
  )
    .then((registeredUnlisten) => {
      if (!active) {
        registeredUnlisten();
        return;
      }

      unlisten = registeredUnlisten;

      if (onRegistered !== undefined) {
        void Promise.resolve(onRegistered()).catch(
          (error: unknown) => {
            console.error(
              `[tauri] Unable to activate "${route.name}".`,
              error,
            );
          },
        );
      }
    })
    .catch((error: unknown) => {
      console.error(
        `[tauri] Unable to subscribe to "${route.name}".`,
        error,
      );
    });

  return () => {
    active = false;
    unlisten?.();
    unlisten = undefined;
  };
};
