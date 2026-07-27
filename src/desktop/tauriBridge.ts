import type {
  CompanionWindowBridge,
  RuntimeSettingsChangeListener,
  RuntimeSettingsBridge,
  ScreenPoint,
} from '../shared/types';
import type { DesktopBridge } from './contracts';
import {
  dispatchTauriCommand,
  TAURI_COMMANDS,
} from './tauriCommands';
import {
  getTauriCursorPosition,
  subscribeToTauriCursorPositions,
} from './tauriCursorStream';
import { subscribeToTauriEvent } from './tauriEvents';

const companionWindowBridge: CompanionWindowBridge = Object.freeze({
  getCursorPosition: getTauriCursorPosition,
  getWindowPosition: () =>
    dispatchTauriCommand(
      TAURI_COMMANDS.getCompanionWindowPosition,
      {},
    ),
  onCursorPosition: subscribeToTauriCursorPositions,
  moveWindow: (position: ScreenPoint) => {
    void dispatchTauriCommand(TAURI_COMMANDS.moveCompanionWindow, {
      position,
    }).catch((error: unknown) => {
      console.error('[tauri] Unable to move companion window.', error);
    });
  },
  setCompanionContentHeight: (height: number) => {
    void dispatchTauriCommand(
      TAURI_COMMANDS.setCompanionContentHeight,
      { height },
    ).catch((error: unknown) => {
      console.error('[tauri] Unable to resize companion window.', error);
    });
  },
  showCompanionContextMenu: () => {
    void dispatchTauriCommand(
      TAURI_COMMANDS.showCompanionContextMenu,
      {},
    ).catch((error: unknown) => {
      console.error(
        '[tauri] Unable to show companion context menu.',
        error,
      );
    });
  },
});

const runtimeSettingsBridge: RuntimeSettingsBridge = Object.freeze({
  getRuntimeSettings: () =>
    dispatchTauriCommand(TAURI_COMMANDS.getRuntimeSettings, {}),
  onRuntimeSettingsChanged: (
    listener: RuntimeSettingsChangeListener,
  ) =>
    subscribeToTauriEvent(
      'companion',
      'runtimeSettingsChanged',
      listener,
    ),
});

/**
 * Exposes only Tauri capabilities that have completed their migration.
 * Unmigrated domain bridges intentionally remain unavailable until their
 * backend commands and event recovery semantics reach parity.
 */
export const tauriDesktopBridge: DesktopBridge = Object.freeze({
  getCompanionBridge: () => undefined,
  getCompanionSettingsBridge: () => undefined,
  getRuntimeSettingsBridge: () => runtimeSettingsBridge,
  getCompanionWindowBridge: () => companionWindowBridge,
  getPreferencesBridge: () => undefined,
  getPreferencesSettingsBridge: () => undefined,
});
