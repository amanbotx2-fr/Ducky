import { invoke, type Channel } from '@tauri-apps/api/core';

import type {
  PreferencesSettings,
  PreferencesSettingsPatch,
  RuntimeSettings,
} from '../shared/settings';
import type { ScreenPoint } from '../shared/types';

/**
 * Registry for commands whose native behavior is complete in the migration.
 * Later domain commands belong to their owning migration phases and must not
 * be added here as placeholders.
 */
export const TAURI_COMMANDS = Object.freeze({
  getCursorPosition: 'get_cursor_position',
  getPreferencesSettings: 'get_preferences_settings',
  getRuntimeSettings: 'get_runtime_settings',
  getCompanionWindowPosition: 'get_companion_window_position',
  moveCompanionWindow: 'move_companion_window',
  setCompanionContentHeight: 'set_companion_content_height',
  showCompanionContextMenu: 'show_companion_context_menu',
  streamCursorPositions: 'stream_cursor_positions',
  stopCursorPositions: 'stop_cursor_positions',
  updatePreferencesSettings: 'update_preferences_settings',
  updateStickyMessage: 'update_sticky_message',
  updateUserName: 'update_user_name',
} as const);

type TauriCommandName =
  (typeof TAURI_COMMANDS)[keyof typeof TAURI_COMMANDS];

interface TauriCommandArguments {
  readonly get_cursor_position: Record<string, never>;
  readonly get_preferences_settings: Record<string, never>;
  readonly get_runtime_settings: Record<string, never>;
  readonly get_companion_window_position: Record<string, never>;
  readonly move_companion_window: {
    readonly position: ScreenPoint;
  };
  readonly set_companion_content_height: {
    readonly height: number;
  };
  readonly show_companion_context_menu: Record<string, never>;
  readonly stream_cursor_positions: {
    readonly onPosition: Channel<ScreenPoint>;
  };
  readonly stop_cursor_positions: Record<string, never>;
  readonly update_preferences_settings: {
    readonly patch: PreferencesSettingsPatch;
  };
  readonly update_sticky_message: {
    readonly message: string | null;
  };
  readonly update_user_name: {
    readonly name: string;
  };
}

interface TauriCommandResults {
  readonly get_cursor_position: ScreenPoint;
  readonly get_preferences_settings: PreferencesSettings;
  readonly get_runtime_settings: RuntimeSettings;
  readonly get_companion_window_position: ScreenPoint;
  readonly move_companion_window: void;
  readonly set_companion_content_height: void;
  readonly show_companion_context_menu: void;
  readonly stream_cursor_positions: void;
  readonly stop_cursor_positions: void;
  readonly update_preferences_settings: PreferencesSettings;
  readonly update_sticky_message: string | null;
  readonly update_user_name: string;
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
