import { invoke, type Channel } from '@tauri-apps/api/core';

import type { ScreenPoint } from '../shared/types';

/**
 * Registry for commands whose native behavior is complete in the migration.
 * Later domain commands belong to their owning migration phases and must not
 * be added here as placeholders.
 */
export const TAURI_COMMANDS = Object.freeze({
  getCursorPosition: 'get_cursor_position',
  getCompanionWindowPosition: 'get_companion_window_position',
  moveCompanionWindow: 'move_companion_window',
  setCompanionContentHeight: 'set_companion_content_height',
  showCompanionContextMenu: 'show_companion_context_menu',
  streamCursorPositions: 'stream_cursor_positions',
  stopCursorPositions: 'stop_cursor_positions',
} as const);

type TauriCommandName =
  (typeof TAURI_COMMANDS)[keyof typeof TAURI_COMMANDS];

interface TauriCommandArguments {
  readonly get_cursor_position: Record<string, never>;
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
}

interface TauriCommandResults {
  readonly get_cursor_position: ScreenPoint;
  readonly get_companion_window_position: ScreenPoint;
  readonly move_companion_window: void;
  readonly set_companion_content_height: void;
  readonly show_companion_context_menu: void;
  readonly stream_cursor_positions: void;
  readonly stop_cursor_positions: void;
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
