import type { PomodoroBridge } from '../shared/types';
import type {
  PomodoroCompletionListener,
  PomodoroCustomDurationRequestListener,
  PomodoroState,
  PomodoroStateListener,
} from '../shared/pomodoro';
import {
  dispatchTauriCommand,
  TAURI_COMMANDS,
} from './tauriCommands';
import { subscribeToTauriEvent } from './tauriEvents';

const stateListeners = new Set<PomodoroStateListener>();
const completionListeners = new Set<PomodoroCompletionListener>();
const customDurationListeners =
  new Set<PomodoroCustomDurationRequestListener>();

let latestState: PomodoroState | null = null;
let pendingCompletion = false;
let pendingCustomDurationRequest = false;
let transportStarted = false;
let registeredListenerCount = 0;
let activationStarted = false;

const notifyRegistered = (): void => {
  registeredListenerCount += 1;

  if (registeredListenerCount !== 3 || activationStarted) {
    return;
  }

  activationStarted = true;
  void dispatchTauriCommand(
    TAURI_COMMANDS.activatePomodoroEvents,
    {},
  ).catch((error: unknown) => {
    activationStarted = false;
    console.error('[tauri] Unable to activate Pomodoro events.', error);
  });
};

const startTransport = (): void => {
  if (transportStarted) {
    return;
  }
  transportStarted = true;

  subscribeToTauriEvent(
    'companion',
    'pomodoroStateChanged',
    (state) => {
      latestState = Object.freeze({ ...state });

      for (const listener of stateListeners) {
        listener(latestState);
      }
    },
    notifyRegistered,
  );
  subscribeToTauriEvent(
    'companion',
    'pomodoroCompleted',
    () => {
      if (completionListeners.size === 0) {
        pendingCompletion = true;
        return;
      }

      for (const listener of completionListeners) {
        listener();
      }
    },
    notifyRegistered,
  );
  subscribeToTauriEvent(
    'companion',
    'customPomodoroDurationRequested',
    () => {
      if (customDurationListeners.size === 0) {
        pendingCustomDurationRequest = true;
        return;
      }

      for (const listener of customDurationListeners) {
        listener();
      }
    },
    notifyRegistered,
  );
};

export const tauriPomodoroBridge: PomodoroBridge = Object.freeze({
  startPomodoro: (durationMinutes: number) =>
    dispatchTauriCommand(TAURI_COMMANDS.startPomodoro, {
      durationMinutes,
    }),
  notifyCustomPomodoroPanelClosed: () => {
    void dispatchTauriCommand(
      TAURI_COMMANDS.customPomodoroPanelClosed,
      {},
    ).catch((error: unknown) => {
      console.error(
        '[tauri] Unable to close the custom Pomodoro panel.',
        error,
      );
    });
  },
  onCustomPomodoroDurationRequested: (
    listener: PomodoroCustomDurationRequestListener,
  ) => {
    customDurationListeners.add(listener);
    startTransport();

    if (pendingCustomDurationRequest) {
      queueMicrotask(() => {
        if (customDurationListeners.has(listener)) {
          pendingCustomDurationRequest = false;
          listener();
        }
      });
    }

    return () => {
      customDurationListeners.delete(listener);
    };
  },
  getPomodoroState: () => latestState,
  onPomodoroStateChanged: (listener: PomodoroStateListener) => {
    stateListeners.add(listener);
    startTransport();

    return () => {
      stateListeners.delete(listener);
    };
  },
  onPomodoroCompleted: (listener: PomodoroCompletionListener) => {
    completionListeners.add(listener);
    startTransport();

    if (pendingCompletion) {
      queueMicrotask(() => {
        if (completionListeners.has(listener)) {
          pendingCompletion = false;
          listener();
        }
      });
    }

    return () => {
      completionListeners.delete(listener);
    };
  },
});
