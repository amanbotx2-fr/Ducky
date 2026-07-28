import type {
  CompanionDesktopBridge,
  PreferencesDesktopBridge,
} from './contracts';
import { tauriDesktopBridge } from './tauriBridge';

/**
 * Role-scoped native surface for the companion renderer.
 *
 * The complete adapter remains private so a renderer cannot request native
 * capabilities belonging to the other window role.
 */
export const companionDesktopBridge: CompanionDesktopBridge = Object.freeze({
  getCompanionBridge: tauriDesktopBridge.getCompanionBridge,
  getCompanionAiBridge: tauriDesktopBridge.getCompanionAiBridge,
  getCompanionSettingsBridge:
    tauriDesktopBridge.getCompanionSettingsBridge,
  getRuntimeSettingsBridge:
    tauriDesktopBridge.getRuntimeSettingsBridge,
  getCompanionWindowBridge:
    tauriDesktopBridge.getCompanionWindowBridge,
  getReminderBridge: tauriDesktopBridge.getReminderBridge,
  getPomodoroBridge: tauriDesktopBridge.getPomodoroBridge,
});

/** Role-scoped native surface for the Preferences renderer. */
export const preferencesDesktopBridge: PreferencesDesktopBridge =
  Object.freeze({
    getPreferencesAiBridge: tauriDesktopBridge.getPreferencesAiBridge,
    getPreferencesSettingsBridge:
      tauriDesktopBridge.getPreferencesSettingsBridge,
    getPreferencesUpdateBridge:
      tauriDesktopBridge.getPreferencesUpdateBridge,
    getCredentialBridge: tauriDesktopBridge.getCredentialBridge,
    getPreferencesSettingsCapabilities:
      tauriDesktopBridge.getPreferencesSettingsCapabilities,
  });
