import { Channel } from '@tauri-apps/api/core';

import type {
  CursorPositionListener,
  ScreenPoint,
} from '../shared/types';
import {
  dispatchTauriCommand,
  TAURI_COMMANDS,
} from './tauriCommands';

const cursorListeners = new Set<CursorPositionListener>();
let cursorChannel: Channel<ScreenPoint> | null = null;
let latestCursorPosition: ScreenPoint | null = null;
let cursorLifecycle = Promise.resolve();

const startCursorStream = async (): Promise<void> => {
  const channel = new Channel<ScreenPoint>((position) => {
    if (cursorChannel !== channel) {
      return;
    }

    latestCursorPosition = position;

    for (const listener of cursorListeners) {
      listener(position);
    }
  });

  cursorChannel = channel;

  try {
    await dispatchTauriCommand(TAURI_COMMANDS.streamCursorPositions, {
      onPosition: channel,
    });
  } catch (error: unknown) {
    if (cursorChannel === channel) {
      cursorChannel = null;
    }

    throw error;
  }
};

const stopCursorStream = async (): Promise<void> => {
  cursorChannel = null;
  await dispatchTauriCommand(TAURI_COMMANDS.stopCursorPositions, {});
};

const reconcileCursorStream = async (): Promise<void> => {
  if (cursorListeners.size > 0 && cursorChannel === null) {
    await startCursorStream();
    return;
  }

  if (cursorListeners.size === 0 && cursorChannel !== null) {
    await stopCursorStream();
  }
};

const scheduleCursorReconciliation = (): void => {
  cursorLifecycle = cursorLifecycle
    .then(reconcileCursorStream)
    .catch((error: unknown) => {
      console.error('[tauri] Unable to reconcile cursor streaming.', error);
    });
};

export const getTauriCursorPosition = (): Promise<ScreenPoint> => {
  if (latestCursorPosition !== null) {
    return Promise.resolve({ ...latestCursorPosition });
  }

  return dispatchTauriCommand(TAURI_COMMANDS.getCursorPosition, {});
};

export const subscribeToTauriCursorPositions = (
  listener: CursorPositionListener,
): (() => void) => {
  cursorListeners.add(listener);
  scheduleCursorReconciliation();

  return () => {
    cursorListeners.delete(listener);
    scheduleCursorReconciliation();
  };
};
