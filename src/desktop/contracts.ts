import type {
  CompanionBridge,
  CompanionSettingsBridge,
  CompanionWindowBridge,
  CredentialBridge,
  PreferencesBridge,
  PreferencesSettingsBridge,
  PomodoroBridge,
  ReminderBridge,
  RuntimeSettingsBridge,
} from '../shared/types';

export interface PreferencesSettingsCapabilities {
  readonly general: boolean;
  readonly notificationSounds: boolean;
  readonly water: boolean;
  readonly updates: boolean;
  readonly ai: boolean;
  readonly aiModelExplorer: boolean;
  readonly credentials: boolean;
}

/** Native capabilities available to the companion renderer. */
export interface CompanionDesktopBridge {
  readonly getCompanionBridge: () => CompanionBridge | undefined;
  readonly getCompanionSettingsBridge: () =>
    | CompanionSettingsBridge
    | undefined;
  readonly getRuntimeSettingsBridge: () =>
    | RuntimeSettingsBridge
    | undefined;
  readonly getCompanionWindowBridge: () =>
    | CompanionWindowBridge
    | undefined;
  readonly getReminderBridge: () => ReminderBridge | undefined;
  readonly getPomodoroBridge: () => PomodoroBridge | undefined;
}

/** Native capabilities available to the Preferences renderer. */
export interface PreferencesDesktopBridge {
  readonly getPreferencesBridge: () => PreferencesBridge | undefined;
  readonly getPreferencesSettingsBridge: () =>
    | PreferencesSettingsBridge
    | undefined;
  readonly getCredentialBridge: () => CredentialBridge | undefined;
  readonly getPreferencesSettingsCapabilities: () =>
    PreferencesSettingsCapabilities;
}

/**
 * Internal adapter contract implemented by each desktop runtime.
 *
 * Renderer entry points receive one of the role-scoped views above rather
 * than this complete privileged surface.
 */
export interface DesktopBridge
  extends CompanionDesktopBridge,
    PreferencesDesktopBridge {}
