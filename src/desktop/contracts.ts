import type {
  CompanionAiBridge,
  CompanionBridge,
  CompanionSettingsBridge,
  CompanionWindowBridge,
  CredentialBridge,
  PreferencesAiBridge,
  PreferencesSettingsBridge,
  PreferencesUpdateBridge,
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
  readonly getCompanionAiBridge: () => CompanionAiBridge | undefined;
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
  readonly getPreferencesAiBridge: () => PreferencesAiBridge | undefined;
  readonly getPreferencesSettingsBridge: () =>
    | PreferencesSettingsBridge
    | undefined;
  readonly getPreferencesUpdateBridge: () =>
    | PreferencesUpdateBridge
    | undefined;
  readonly getCredentialBridge: () => CredentialBridge | undefined;
  readonly getPreferencesSettingsCapabilities: () =>
    PreferencesSettingsCapabilities;
}

/** Complete native adapter kept private behind the role-scoped views. */
export interface DesktopBridge
  extends CompanionDesktopBridge,
    PreferencesDesktopBridge {}
