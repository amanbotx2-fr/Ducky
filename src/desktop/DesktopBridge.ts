import { isTauri } from '@tauri-apps/api/core';

import { electronDesktopBridge } from './electronBridge';
import { tauriDesktopBridge } from './tauriBridge';

/**
 * Renderer-facing desktop integration boundary.
 *
 * Callers depend on this module rather than Electron or Tauri. Each shell
 * exposes only native capabilities whose migration is complete.
 */
export const desktopBridge = isTauri()
  ? tauriDesktopBridge
  : electronDesktopBridge;
