import { invoke, type Channel } from '@tauri-apps/api/core';

import type {
  CredentialId,
  CredentialStatus,
} from '../shared/credentials';
import type {
  AiConfigurationUpdate,
  PreferencesSettings,
  PreferencesSettingsPatch,
  RuntimeSettings,
} from '../shared/settings';
import type { AIConversationRequest } from '../shared/aiConversation';
import type {
  AIConnectionTestResult,
  AIModelListResult,
} from '../shared/types';
import type {
  CreateReminderInput,
  Reminder,
  UpdateReminderInput,
} from '../shared/reminders';
import type { AIAskResult, ScreenPoint } from '../shared/types';
import type { UpdateStatus } from '../shared/updates';

/**
 * Registry for commands whose native behavior is complete in the migration.
 * Later domain commands belong to their owning migration phases and must not
 * be added here as placeholders.
 */
export const TAURI_COMMANDS = Object.freeze({
  askAI: 'ask_ai',
  updateAiConfiguration: 'update_ai_configuration',
  listAIModels: 'list_ai_models',
  testAIConnection: 'test_ai_connection',
  activatePomodoroEvents: 'activate_pomodoro_events',
  activateReminderEvents: 'activate_reminder_events',
  createReminder: 'create_reminder',
  customPomodoroPanelClosed: 'custom_pomodoro_panel_closed',
  deleteReminder: 'delete_reminder',
  getCursorPosition: 'get_cursor_position',
  getCredentialStatus: 'get_credential_status',
  getPreferencesSettings: 'get_preferences_settings',
  getUpdateStatus: 'get_update_status',
  getRuntimeSettings: 'get_runtime_settings',
  getCompanionWindowPosition: 'get_companion_window_position',
  getReminder: 'get_reminder',
  listReminders: 'list_reminders',
  markReminderCompleted: 'mark_reminder_completed',
  moveCompanionWindow: 'move_companion_window',
  setCompanionContentHeight: 'set_companion_content_height',
  showCompanionContextMenu: 'show_companion_context_menu',
  saveCredential: 'save_credential',
  startPomodoro: 'start_pomodoro',
  streamCursorPositions: 'stream_cursor_positions',
  stopCursorPositions: 'stop_cursor_positions',
  checkForUpdates: 'check_for_updates',
  updatePreferencesSettings: 'update_preferences_settings',
  updateReminder: 'update_reminder',
  updateStickyMessage: 'update_sticky_message',
  updateUserName: 'update_user_name',
  deleteCredential: 'delete_credential',
} as const);

type TauriCommandName =
  (typeof TAURI_COMMANDS)[keyof typeof TAURI_COMMANDS];

interface TauriCommandArguments {
  readonly ask_ai: {
    readonly request: AIConversationRequest;
  };
  readonly update_ai_configuration: {
    readonly configuration: AiConfigurationUpdate;
  };
  readonly list_ai_models: Record<string, never>;
  readonly test_ai_connection: Record<string, never>;
  readonly activate_pomodoro_events: Record<string, never>;
  readonly activate_reminder_events: Record<string, never>;
  readonly create_reminder: {
    readonly input: CreateReminderInput;
  };
  readonly custom_pomodoro_panel_closed: Record<string, never>;
  readonly delete_reminder: {
    readonly id: string;
  };
  readonly get_cursor_position: Record<string, never>;
  readonly get_credential_status: {
    readonly id: CredentialId;
  };
  readonly get_preferences_settings: Record<string, never>;
  readonly get_update_status: Record<string, never>;
  readonly get_runtime_settings: Record<string, never>;
  readonly get_companion_window_position: Record<string, never>;
  readonly get_reminder: {
    readonly id: string;
  };
  readonly list_reminders: Record<string, never>;
  readonly mark_reminder_completed: {
    readonly id: string;
  };
  readonly move_companion_window: {
    readonly position: ScreenPoint;
  };
  readonly set_companion_content_height: {
    readonly height: number;
  };
  readonly show_companion_context_menu: Record<string, never>;
  readonly save_credential: {
    readonly id: CredentialId;
    readonly secret: string;
  };
  readonly start_pomodoro: {
    readonly durationMinutes: number;
  };
  readonly stream_cursor_positions: {
    readonly onPosition: Channel<ScreenPoint>;
  };
  readonly stop_cursor_positions: Record<string, never>;
  readonly check_for_updates: Record<string, never>;
  readonly update_preferences_settings: {
    readonly patch: PreferencesSettingsPatch;
  };
  readonly update_reminder: {
    readonly id: string;
    readonly input: UpdateReminderInput;
  };
  readonly update_sticky_message: {
    readonly message: string | null;
  };
  readonly update_user_name: {
    readonly name: string;
  };
  readonly delete_credential: {
    readonly id: CredentialId;
  };
}

interface TauriCommandResults {
  readonly ask_ai: AIAskResult;
  readonly update_ai_configuration: PreferencesSettings;
  readonly list_ai_models: AIModelListResult;
  readonly test_ai_connection: AIConnectionTestResult;
  readonly activate_pomodoro_events: void;
  readonly activate_reminder_events: void;
  readonly create_reminder: Reminder;
  readonly custom_pomodoro_panel_closed: void;
  readonly delete_reminder: boolean;
  readonly get_cursor_position: ScreenPoint;
  readonly get_credential_status: CredentialStatus;
  readonly get_preferences_settings: PreferencesSettings;
  readonly get_update_status: UpdateStatus;
  readonly get_runtime_settings: RuntimeSettings;
  readonly get_companion_window_position: ScreenPoint;
  readonly get_reminder: Reminder | null;
  readonly list_reminders: readonly Reminder[];
  readonly mark_reminder_completed: Reminder;
  readonly move_companion_window: void;
  readonly set_companion_content_height: void;
  readonly show_companion_context_menu: void;
  readonly save_credential: CredentialStatus;
  readonly start_pomodoro: void;
  readonly stream_cursor_positions: void;
  readonly stop_cursor_positions: void;
  readonly check_for_updates: UpdateStatus;
  readonly update_preferences_settings: PreferencesSettings;
  readonly update_reminder: Reminder;
  readonly update_sticky_message: string | null;
  readonly update_user_name: string;
  readonly delete_credential: CredentialStatus;
}

/**
 * Typed dispatch boundary for the Tauri adapter.
 *
 * React components continue to consume DesktopBridge and never receive raw
 * command names or access to Tauri's generic invoke function.
 */
export const dispatchTauriCommand = <Command extends TauriCommandName>(
  command: Command,
  arguments_: TauriCommandArguments[Command],
): Promise<TauriCommandResults[Command]> =>
  invoke<TauriCommandResults[Command]>(command, arguments_);
