import type {
  CompanionBridge,
  CompanionSettingsBridge,
  CompanionWindowBridge,
  PreferencesBridge,
  PreferencesSettingsBridge,
} from '../shared/types';

/** Native capabilities available to the companion renderer. */
export interface CompanionDesktopBridge {
  readonly getCompanionBridge: () => CompanionBridge | undefined;
  readonly getCompanionSettingsBridge: () =>
    | CompanionSettingsBridge
    | undefined;
  readonly getCompanionWindowBridge: () =>
    | CompanionWindowBridge
    | undefined;
}

/** Native capabilities available to the Preferences renderer. */
export interface PreferencesDesktopBridge {
  readonly getPreferencesBridge: () => PreferencesBridge | undefined;
  readonly getPreferencesSettingsBridge: () =>
    | PreferencesSettingsBridge
    | undefined;
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
