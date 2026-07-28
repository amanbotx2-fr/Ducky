import type {
  CompanionAiBridge,
  CompanionSettingsBridge,
  CompanionWindowBridge,
  CredentialBridge,
  PreferencesAiBridge,
  PreferencesSettingsBridge,
  PreferencesUpdateBridge,
  ReminderBridge,
  RuntimeSettingsChangeListener,
  RuntimeSettingsBridge,
  ScreenPoint,
} from '../shared/types';
import type { PreferencesSettingsPatch } from '../shared/settings';
import type { CredentialId } from '../shared/credentials';
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
import { tauriPomodoroBridge } from './tauriPomodoroBridge';

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

const companionAiBridge: CompanionAiBridge = Object.freeze({
  askAI: (request: Parameters<CompanionAiBridge['askAI']>[0]) =>
    dispatchTauriCommand(TAURI_COMMANDS.askAI, { request }),
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

const companionSettingsBridge: CompanionSettingsBridge = Object.freeze({
  ...runtimeSettingsBridge,
  updateUserName: (name: string) =>
    dispatchTauriCommand(TAURI_COMMANDS.updateUserName, { name }),
  updateStickyMessage: (message: string | null) =>
    dispatchTauriCommand(TAURI_COMMANDS.updateStickyMessage, {
      message,
    }),
});

const preferencesSettingsBridge: PreferencesSettingsBridge =
  Object.freeze({
    getPreferencesSettings: () =>
      dispatchTauriCommand(
        TAURI_COMMANDS.getPreferencesSettings,
        {},
      ),
    updatePreferencesSettings: (patch: PreferencesSettingsPatch) =>
      dispatchTauriCommand(
        TAURI_COMMANDS.updatePreferencesSettings,
        { patch },
      ),
    onRuntimeSettingsChanged: (
      listener: RuntimeSettingsChangeListener,
    ) =>
      subscribeToTauriEvent(
        'preferences',
        'runtimeSettingsChanged',
        listener,
      ),
  });

const preferencesAiBridge: PreferencesAiBridge = Object.freeze({
  updateAiConfiguration: (
    configuration: Parameters<
      PreferencesAiBridge['updateAiConfiguration']
    >[0],
  ) =>
    dispatchTauriCommand(TAURI_COMMANDS.updateAiConfiguration, {
      configuration,
    }),
  listAIModels: () =>
    dispatchTauriCommand(TAURI_COMMANDS.listAIModels, {}),
  testAIConnection: () =>
    dispatchTauriCommand(TAURI_COMMANDS.testAIConnection, {}),
});

const preferencesUpdateBridge: PreferencesUpdateBridge = Object.freeze({
  getUpdateStatus: () =>
    dispatchTauriCommand(TAURI_COMMANDS.getUpdateStatus, {}),
  checkForUpdates: () =>
    dispatchTauriCommand(TAURI_COMMANDS.checkForUpdates, {}),
  onUpdateStatusChanged: (
    listener: Parameters<
      PreferencesUpdateBridge['onUpdateStatusChanged']
    >[0],
  ) =>
    subscribeToTauriEvent(
      'preferences',
      'updateStatusChanged',
      listener,
    ),
});

const credentialBridge: CredentialBridge = Object.freeze({
  getCredentialStatus: (id: CredentialId) =>
    dispatchTauriCommand(TAURI_COMMANDS.getCredentialStatus, { id }),
  saveCredential: (id: CredentialId, secret: string) =>
    dispatchTauriCommand(TAURI_COMMANDS.saveCredential, {
      id,
      secret,
    }),
  deleteCredential: (id: CredentialId) =>
    dispatchTauriCommand(TAURI_COMMANDS.deleteCredential, { id }),
});

const reminderBridge = Object.freeze({
  createReminder: (input) =>
    dispatchTauriCommand(TAURI_COMMANDS.createReminder, { input }),
  updateReminder: (id, input) =>
    dispatchTauriCommand(TAURI_COMMANDS.updateReminder, {
      id,
      input,
    }),
  deleteReminder: (id) =>
    dispatchTauriCommand(TAURI_COMMANDS.deleteReminder, { id }),
  getReminder: (id) =>
    dispatchTauriCommand(TAURI_COMMANDS.getReminder, { id }),
  listReminders: () =>
    dispatchTauriCommand(TAURI_COMMANDS.listReminders, {}),
  markReminderCompleted: (id) =>
    dispatchTauriCommand(TAURI_COMMANDS.markReminderCompleted, {
      id,
    }),
  onReminderCreationPanelRequested: (listener) =>
    subscribeToTauriEvent(
      'companion',
      'reminderCreationPanelRequested',
      listener,
    ),
  onReminderManagerPanelRequested: (listener) =>
    subscribeToTauriEvent(
      'companion',
      'reminderManagerPanelRequested',
      listener,
    ),
  onReminderFired: (listener) =>
    subscribeToTauriEvent(
      'companion',
      'reminderFired',
      listener,
      () =>
        dispatchTauriCommand(
          TAURI_COMMANDS.activateReminderEvents,
          {},
        ),
    ),
} satisfies ReminderBridge);

const TAURI_PREFERENCES_SETTINGS_CAPABILITIES = Object.freeze({
  general: true,
  notificationSounds: true,
  water: false,
  updates: false,
  ai: true,
  aiModelExplorer: true,
  credentials: true,
});

/**
 * Exposes only Tauri capabilities that have completed their migration.
 * Unmigrated domain bridges intentionally remain unavailable until their
 * backend commands and event recovery semantics reach parity.
 */
export const tauriDesktopBridge: DesktopBridge = Object.freeze({
  getCompanionBridge: () => undefined,
  getCompanionAiBridge: () => companionAiBridge,
  getCompanionSettingsBridge: () => companionSettingsBridge,
  getRuntimeSettingsBridge: () => runtimeSettingsBridge,
  getCompanionWindowBridge: () => companionWindowBridge,
  getReminderBridge: () => reminderBridge,
  getPomodoroBridge: () => tauriPomodoroBridge,
  getPreferencesAiBridge: () => preferencesAiBridge,
  getPreferencesSettingsBridge: () => preferencesSettingsBridge,
  getPreferencesUpdateBridge: () => preferencesUpdateBridge,
  getCredentialBridge: () => credentialBridge,
  getPreferencesSettingsCapabilities: () =>
    TAURI_PREFERENCES_SETTINGS_CAPABILITIES,
});
