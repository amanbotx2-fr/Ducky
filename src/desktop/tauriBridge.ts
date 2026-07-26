import { invoke } from '@tauri-apps/api/core';

import type {
  CompanionWindowBridge,
  ScreenPoint,
} from '../shared/types';
import type { DesktopBridge } from './contracts';

const companionWindowBridge: CompanionWindowBridge = Object.freeze({
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
