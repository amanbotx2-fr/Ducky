import type {
  DailyPlannerPanelRequestListener,
  StickyMessagePanelRequestListener,
  UserNamePanelRequestListener,
} from '../shared/types';
import {
  dispatchTauriCommand,
  TAURI_COMMANDS,
} from './tauriCommands';
import { subscribeToTauriEvent } from './tauriEvents';

type PanelListener = () => void;

const userNameListeners = new Set<UserNamePanelRequestListener>();
const stickyMessageListeners =
  new Set<StickyMessagePanelRequestListener>();
const dailyPlannerListeners =
  new Set<DailyPlannerPanelRequestListener>();

let pendingUserNameRequest = false;
let pendingStickyMessageRequest = false;
let pendingDailyPlannerRequest = false;
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
    TAURI_COMMANDS.activatePersonalAssistantEvents,
    {},
  ).catch((error: unknown) => {
    activationStarted = false;
    console.error(
      '[tauri] Unable to activate Personal Assistant events.',
      error,
    );
  });
};

const deliverOrRetain = (
  listeners: ReadonlySet<PanelListener>,
  retain: () => void,
): void => {
  if (listeners.size === 0) {
    retain();
    return;
  }

  for (const listener of listeners) {
    listener();
  }
};

const startTransport = (): void => {
  if (transportStarted) {
    return;
  }
  transportStarted = true;

  subscribeToTauriEvent(
    'companion',
    'userNamePanelRequested',
    () => {
      deliverOrRetain(userNameListeners, () => {
        pendingUserNameRequest = true;
      });
    },
    notifyRegistered,
  );
  subscribeToTauriEvent(
    'companion',
    'stickyMessagePanelRequested',
    () => {
      deliverOrRetain(stickyMessageListeners, () => {
        pendingStickyMessageRequest = true;
      });
    },
    notifyRegistered,
  );
  subscribeToTauriEvent(
    'companion',
    'dailyPlannerPanelRequested',
    () => {
      deliverOrRetain(dailyPlannerListeners, () => {
        pendingDailyPlannerRequest = true;
      });
    },
    notifyRegistered,
  );
};

const subscribe = (
  listeners: Set<PanelListener>,
  listener: PanelListener,
  hasPending: () => boolean,
  clearPending: () => void,
): (() => void) => {
  listeners.add(listener);
  startTransport();

  if (hasPending()) {
    queueMicrotask(() => {
      if (listeners.has(listener)) {
        clearPending();
        listener();
      }
    });
  }

  return () => {
    listeners.delete(listener);
  };
};

export const tauriPersonalAssistantBridge = Object.freeze({
  onUserNamePanelRequested: (
    listener: UserNamePanelRequestListener,
  ) =>
    subscribe(
      userNameListeners,
      listener,
      () => pendingUserNameRequest,
      () => {
        pendingUserNameRequest = false;
      },
    ),
  onStickyMessagePanelRequested: (
    listener: StickyMessagePanelRequestListener,
  ) =>
    subscribe(
      stickyMessageListeners,
      listener,
      () => pendingStickyMessageRequest,
      () => {
        pendingStickyMessageRequest = false;
      },
    ),
  onDailyPlannerPanelRequested: (
    listener: DailyPlannerPanelRequestListener,
  ) =>
    subscribe(
      dailyPlannerListeners,
      listener,
      () => pendingDailyPlannerRequest,
      () => {
        pendingDailyPlannerRequest = false;
      },
    ),
});
