import type { DesktopBridge } from './contracts';

const ELECTRON_PREFERENCES_SETTINGS_CAPABILITIES = Object.freeze({
  general: true,
  notificationSounds: true,
  water: true,
  updates: true,
  ai: true,
  aiModelExplorer: true,
});

/**
 * Adapts the existing, role-specific Electron preload APIs to the runtime-
 * neutral DesktopBridge contract. Electron remains authoritative until the
 * corresponding Tauri implementation reaches feature parity.
 */
export const electronDesktopBridge: DesktopBridge = Object.freeze({
  getCompanionBridge: () => window.psyduck,
  getCompanionSettingsBridge: () => window.psyduck,
  getRuntimeSettingsBridge: () => window.psyduck,
  getCompanionWindowBridge: () => window.psyduck,
  getPreferencesBridge: () => window.psyduckPreferences,
  getPreferencesSettingsBridge: () => window.psyduckPreferences,
  getPreferencesSettingsCapabilities: () =>
    ELECTRON_PREFERENCES_SETTINGS_CAPABILITIES,
});
