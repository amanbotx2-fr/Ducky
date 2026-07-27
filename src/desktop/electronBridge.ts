import type { DesktopBridge } from './contracts';
import type {
  CredentialId,
  CredentialStatus,
} from '../shared/credentials';
import type { CredentialBridge } from '../shared/types';

const ELECTRON_PREFERENCES_SETTINGS_CAPABILITIES = Object.freeze({
  general: true,
  notificationSounds: true,
  water: true,
  updates: true,
  ai: true,
  aiModelExplorer: true,
  credentials: true,
});

const assertAiCredential = (id: CredentialId): void => {
  if (id !== 'aiApiKey') {
    throw new TypeError('Unsupported credential.');
  }
};

const toCredentialStatus = (
  id: CredentialId,
  configured: boolean,
): CredentialStatus =>
  Object.freeze({
    id,
    state: configured ? 'configured' : 'missing',
  });

const getPreferencesPreload = () => {
  const preferences = window.psyduckPreferences;

  if (preferences === undefined) {
    throw new Error('Credential storage is unavailable in this window.');
  }

  return preferences;
};

const electronCredentialBridge: CredentialBridge = Object.freeze({
  getCredentialStatus: async (id: CredentialId) => {
    assertAiCredential(id);
    const settings = await getPreferencesPreload().getPreferencesSettings();
    return toCredentialStatus(id, settings.ai.apiKeyConfigured);
  },
  saveCredential: async (id: CredentialId, secret: string) => {
    assertAiCredential(id);
    const preferences = getPreferencesPreload();
    const settings = await preferences.getPreferencesSettings();
    const saved = await preferences.updateAiConfiguration({
      enabled: settings.ai.enabled,
      provider: settings.ai.provider,
      model: settings.ai.model,
      endpoint: settings.ai.endpoint,
      baseUrl: settings.ai.baseUrl,
      apiKey: secret,
    });
    return toCredentialStatus(id, saved.ai.apiKeyConfigured);
  },
  deleteCredential: async (id: CredentialId) => {
    assertAiCredential(id);
    const preferences = getPreferencesPreload();
    const settings = await preferences.getPreferencesSettings();
    const saved = await preferences.updateAiConfiguration({
      enabled: settings.ai.enabled,
      provider: settings.ai.provider,
      model: settings.ai.model,
      endpoint: settings.ai.endpoint,
      baseUrl: settings.ai.baseUrl,
      apiKey: '',
    });
    return toCredentialStatus(id, saved.ai.apiKeyConfigured);
  },
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
  getReminderBridge: () => window.psyduck,
  getPomodoroBridge: () => window.psyduck,
  getPreferencesBridge: () => window.psyduckPreferences,
  getPreferencesSettingsBridge: () => window.psyduckPreferences,
  getCredentialBridge: () => electronCredentialBridge,
  getPreferencesSettingsCapabilities: () =>
    ELECTRON_PREFERENCES_SETTINGS_CAPABILITIES,
});
