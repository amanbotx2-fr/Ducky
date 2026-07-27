import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { PomodoroState } from '../shared/pomodoro';
import type { ReminderFiredNotification } from '../shared/reminders';
import type { RuntimeSettings } from '../shared/settings';
import type { UpdateStatus } from '../shared/updates';
import { IPC_CHANNELS } from '../shared/events';

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
 * Event names reuse the frozen Electron contract while payload production and
 * recovery semantics remain owned by their later feature phases. The ordered
 * cursor channel is intentionally absent until Task 3.3.
 */
export const TAURI_EVENTS = Object.freeze({
  runtimeSettingsChanged: {
    name: IPC_CHANNELS.runtimeSettingsChanged,
    targets: ['companion', 'preferences'],
  },
  userNamePanelRequested: {
    name: IPC_CHANNELS.userNamePanelRequested,
    targets: ['companion'],
  },
  stickyMessagePanelRequested: {
    name: IPC_CHANNELS.stickyMessagePanelRequested,
    targets: ['companion'],
  },
  reminderCreationPanelRequested: {
    name: IPC_CHANNELS.reminderCreationPanelRequested,
    targets: ['companion'],
  },
  reminderManagerPanelRequested: {
    name: IPC_CHANNELS.reminderManagerPanelRequested,
    targets: ['companion'],
  },
  dailyPlannerPanelRequested: {
    name: IPC_CHANNELS.dailyPlannerPanelRequested,
    targets: ['companion'],
  },
  reminderFired: {
    name: IPC_CHANNELS.reminderFired,
    targets: ['companion'],
  },
  updateStatusChanged: {
    name: IPC_CHANNELS.updateStatusChanged,
    targets: ['preferences'],
  },
  customPomodoroDurationRequested: {
    name: IPC_CHANNELS.customPomodoroDurationRequested,
    targets: ['companion'],
  },
  pomodoroStateChanged: {
    name: IPC_CHANNELS.pomodoroStateChanged,
    targets: ['companion'],
  },
  pomodoroCompleted: {
    name: IPC_CHANNELS.pomodoroCompleted,
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
