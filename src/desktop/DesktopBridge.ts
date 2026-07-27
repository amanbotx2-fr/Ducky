import { isTauri } from '@tauri-apps/api/core';

import type {
  CompanionDesktopBridge,
  PreferencesDesktopBridge,
} from './contracts';
import { electronDesktopBridge } from './electronBridge';
import { tauriDesktopBridge } from './tauriBridge';

/**
 * Runtime adapter selected once for the current desktop shell.
 *
 * The complete adapter remains private so a renderer cannot request native
 * capabilities belonging to the other window role.
 */
const runtimeDesktopBridge = isTauri()
  ? tauriDesktopBridge
  : electronDesktopBridge;

/** Electron/Tauri-neutral native surface for the companion renderer. */
export const companionDesktopBridge: CompanionDesktopBridge = Object.freeze({
  getCompanionBridge: runtimeDesktopBridge.getCompanionBridge,
  getCompanionSettingsBridge:
    runtimeDesktopBridge.getCompanionSettingsBridge,
  getRuntimeSettingsBridge:
    runtimeDesktopBridge.getRuntimeSettingsBridge,
  getCompanionWindowBridge:
    runtimeDesktopBridge.getCompanionWindowBridge,
});

/** Electron/Tauri-neutral native surface for the Preferences renderer. */
export const preferencesDesktopBridge: PreferencesDesktopBridge =
  Object.freeze({
    getPreferencesBridge: runtimeDesktopBridge.getPreferencesBridge,
    getPreferencesSettingsBridge:
      runtimeDesktopBridge.getPreferencesSettingsBridge,
  });
