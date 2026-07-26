import type {
  CompanionBridge,
  PreferencesBridge,
} from '../shared/types';

/**
 * Selects the privileged bridge available to each renderer surface without
 * exposing a desktop runtime implementation to React components.
 */
export interface DesktopBridge {
  readonly getCompanionBridge: () => CompanionBridge | undefined;
  readonly getPreferencesBridge: () => PreferencesBridge | undefined;
}

