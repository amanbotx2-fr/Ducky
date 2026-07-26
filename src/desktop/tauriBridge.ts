import { Channel, invoke } from '@tauri-apps/api/core';

import type {
  CompanionWindowBridge,
  CursorPositionListener,
  ScreenPoint,
} from '../shared/types';
import type { DesktopBridge } from './contracts';

const cursorListeners = new Set<CursorPositionListener>();
let cursorChannel: Channel<ScreenPoint> | null = null;
let cursorStreamStarted = false;
let latestCursorPosition: ScreenPoint | null = null;

const ensureCursorStream = (): void => {
  if (cursorStreamStarted) {
    return;
  }

  cursorStreamStarted = true;
  cursorChannel = new Channel<ScreenPoint>((position) => {
    latestCursorPosition = position;

    for (const listener of cursorListeners) {
      listener(position);
    }
  });

  void invoke('stream_cursor_positions', {
    onPosition: cursorChannel,
  }).catch((error: unknown) => {
    cursorStreamStarted = false;
    cursorChannel = null;
    console.error('[tauri] Unable to stream cursor positions.', error);
  });
};

const companionWindowBridge: CompanionWindowBridge = Object.freeze({
  getCursorPosition: () => {
    if (latestCursorPosition !== null) {
      return Promise.resolve({ ...latestCursorPosition });
    }

    return invoke<ScreenPoint>('get_cursor_position');
  },
  onCursorPosition: (listener: CursorPositionListener) => {
    cursorListeners.add(listener);
    ensureCursorStream();

    return () => {
      cursorListeners.delete(listener);
    };
  },
  moveWindow: (position: ScreenPoint) => {
    void invoke('move_companion_window', { position }).catch((error: unknown) => {
      console.error('[tauri] Unable to move companion window.', error);
    });
  },
  setCompanionContentHeight: (height: number) => {
    void invoke('set_companion_content_height', { height }).catch(
      (error: unknown) => {
        console.error('[tauri] Unable to resize companion window.', error);
      },
    );
  },
});

/**
 * Exposes only Tauri capabilities that have completed their migration.
 * Unmigrated domain bridges intentionally remain unavailable until their
 * backend commands and event recovery semantics reach parity.
 */
export const tauriDesktopBridge: DesktopBridge = Object.freeze({
  getCompanionBridge: () => undefined,
  getCompanionWindowBridge: () => companionWindowBridge,
  getPreferencesBridge: () => undefined,
});
