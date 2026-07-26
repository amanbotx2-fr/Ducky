import { electronDesktopBridge } from './electronBridge';

/**
 * Renderer-facing desktop integration boundary.
 *
 * Callers depend on this module rather than Electron or Tauri. The active
 * implementation remains Electron-backed during Phase 0.
 */
export const desktopBridge = electronDesktopBridge;

