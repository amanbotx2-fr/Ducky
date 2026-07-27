import { useCallback, useEffect, useRef, useState } from 'react';

import { preferencesDesktopBridge } from '../../desktop/DesktopBridge';
import type { PreferencesSettingsCapabilities } from '../../desktop/contracts';
import {
  type AiConfigurationUpdate,
  createDefaultPreferencesSettings,
  mergePreferencesSettings,
  type PreferencesSettings,
  type PreferencesSettingsPatch,
} from '../../shared/settings';

export type SettingsStatus =
  | 'loading'
  | 'ready'
  | 'saving'
  | 'saved'
  | 'error';

export interface PreferencesSettingsController {
  readonly settings: PreferencesSettings;
  readonly capabilities: PreferencesSettingsCapabilities;
  readonly status: SettingsStatus;
  readonly errorMessage: string | null;
  readonly update: (patch: PreferencesSettingsPatch) => Promise<boolean>;
  readonly updateAiConfiguration: (
    configuration: AiConfigurationUpdate,
  ) => Promise<boolean>;
}

const isPatchSupported = (
  patch: PreferencesSettingsPatch,
  capabilities: PreferencesSettingsCapabilities,
): boolean =>
  (patch.general === undefined || capabilities.general) &&
  (patch.notificationSounds === undefined ||
    capabilities.notificationSounds) &&
  (patch.water === undefined || capabilities.water) &&
  (patch.updates === undefined || capabilities.updates) &&
  (patch.aiModelExplorer === undefined ||
    capabilities.aiModelExplorer);

export function usePreferencesSettings(): PreferencesSettingsController {
  const capabilities =
    preferencesDesktopBridge.getPreferencesSettingsCapabilities();
  const [settings, setSettings] = useState(
    createDefaultPreferencesSettings,
  );
  const [status, setStatus] = useState<SettingsStatus>('loading');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const mountedRef = useRef(true);
  const updateRevisionRef = useRef(0);

  useEffect(() => {
    mountedRef.current = true;
    const preferencesBridge =
      preferencesDesktopBridge.getPreferencesSettingsBridge();

    if (preferencesBridge === undefined) {
      setStatus('error');
      setErrorMessage('Settings are unavailable in this window.');
      return () => {
        mountedRef.current = false;
      };
    }

    const unsubscribe = preferencesBridge.onRuntimeSettingsChanged(
      (runtimeSettings) => {
        if (!mountedRef.current) {
          return;
        }

        setSettings((currentSettings) => ({
          ...currentSettings,
          userName: runtimeSettings.userName,
          general: { ...runtimeSettings.general },
          water: { ...runtimeSettings.water },
          notificationSounds: {
            ...runtimeSettings.notificationSounds,
          },
        }));
        setStatus('saved');
        setErrorMessage(null);
      },
    );

    void preferencesBridge
      .getPreferencesSettings()
      .then((nextSettings) => {
        if (!mountedRef.current) {
          return;
        }

        setSettings(nextSettings);
        setStatus('ready');
        setErrorMessage(null);
      })
      .catch(() => {
        if (!mountedRef.current) {
          return;
        }

        setStatus('error');
        setErrorMessage('Settings could not be loaded.');
      });

    return () => {
      mountedRef.current = false;
      unsubscribe();
    };
  }, []);

  const update = useCallback(
    async (patch: PreferencesSettingsPatch): Promise<boolean> => {
      if (!isPatchSupported(patch, capabilities)) {
        setStatus('error');
        setErrorMessage(
          'This setting is not available in the current desktop runtime.',
        );
        return false;
      }

      const preferencesBridge =
        preferencesDesktopBridge.getPreferencesSettingsBridge();

      if (preferencesBridge === undefined) {
        setStatus('error');
        setErrorMessage('Settings are unavailable in this window.');
        return false;
      }

      const revision = updateRevisionRef.current + 1;
      updateRevisionRef.current = revision;
      setSettings((currentSettings) =>
        mergePreferencesSettings(currentSettings, patch),
      );
      setStatus('saving');
      setErrorMessage(null);

      try {
        const savedSettings =
          await preferencesBridge.updatePreferencesSettings(patch);

        if (
          mountedRef.current &&
          revision === updateRevisionRef.current
        ) {
          setSettings(savedSettings);
          setStatus('saved');
        }

        return true;
      } catch {
        if (!mountedRef.current) {
          return false;
        }

        setStatus('error');
        setErrorMessage('Your change could not be saved. Try again.');

        try {
          const authoritativeSettings =
            await preferencesBridge.getPreferencesSettings();

          if (
            mountedRef.current &&
            revision === updateRevisionRef.current
          ) {
            setSettings(authoritativeSettings);
          }
        } catch {
          // The actionable save error remains visible.
        }

        return false;
      }
    },
    [capabilities],
  );

  const updateAiConfiguration = useCallback(
    async (configuration: AiConfigurationUpdate): Promise<boolean> => {
      const preferencesBridge =
        preferencesDesktopBridge.getPreferencesBridge();
      const settingsBridge =
        preferencesDesktopBridge.getPreferencesSettingsBridge();

      if (
        preferencesBridge === undefined ||
        settingsBridge === undefined
      ) {
        setStatus('error');
        setErrorMessage('Settings are unavailable in this window.');
        return false;
      }

      const revision = updateRevisionRef.current + 1;
      updateRevisionRef.current = revision;
      setStatus('saving');
      setErrorMessage(null);

      try {
        const savedSettings =
          await preferencesBridge.updateAiConfiguration(configuration);

        if (
          mountedRef.current &&
          revision === updateRevisionRef.current
        ) {
          setSettings(savedSettings);
          setStatus('saved');
        }

        return true;
      } catch {
        if (!mountedRef.current) {
          return false;
        }

        setStatus('error');
        setErrorMessage('Your change could not be saved. Try again.');

        try {
          const authoritativeSettings =
            await settingsBridge.getPreferencesSettings();

          if (
            mountedRef.current &&
            revision === updateRevisionRef.current
          ) {
            setSettings(authoritativeSettings);
          }
        } catch {
          // The actionable save error remains visible.
        }

        return false;
      }
    },
    [],
  );

  return {
    settings,
    capabilities,
    status,
    errorMessage,
    update,
    updateAiConfiguration,
  };
}
