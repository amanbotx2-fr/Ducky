import { useCallback, useEffect, useRef, useState } from 'react';

import { preferencesDesktopBridge } from '../../desktop/DesktopBridge';
import type { PreferencesSettingsCapabilities } from '../../desktop/contracts';
import type { CredentialStatus } from '../../shared/credentials';
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
  readonly credentialStatus: CredentialStatus | null;
  readonly update: (patch: PreferencesSettingsPatch) => Promise<boolean>;
  readonly updateAiConfiguration: (
    configuration: AiConfigurationUpdate,
  ) => Promise<boolean>;
  readonly saveCredential: (secret: string) => Promise<boolean>;
  readonly deleteCredential: () => Promise<boolean>;
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

const applyCredentialStatus = (
  settings: PreferencesSettings,
  credentialStatus: CredentialStatus,
): PreferencesSettings => ({
  ...settings,
  ai: {
    ...settings.ai,
    apiKeyConfigured: credentialStatus.state === 'configured',
  },
});

export function usePreferencesSettings(): PreferencesSettingsController {
  const capabilities =
    preferencesDesktopBridge.getPreferencesSettingsCapabilities();
  const [settings, setSettings] = useState(
    createDefaultPreferencesSettings,
  );
  const [status, setStatus] = useState<SettingsStatus>('loading');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [credentialStatus, setCredentialStatus] =
    useState<CredentialStatus | null>(null);
  const mountedRef = useRef(true);
  const updateRevisionRef = useRef(0);

  useEffect(() => {
    mountedRef.current = true;
    const preferencesBridge =
      preferencesDesktopBridge.getPreferencesSettingsBridge();
    const credentialBridge =
      preferencesDesktopBridge.getCredentialBridge();

    if (
      preferencesBridge === undefined ||
      (capabilities.credentials && credentialBridge === undefined)
    ) {
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

    const credentialRequest =
      capabilities.credentials && credentialBridge !== undefined
        ? credentialBridge.getCredentialStatus('aiApiKey')
        : Promise.resolve<CredentialStatus | null>(null);

    void Promise.all([
      preferencesBridge.getPreferencesSettings(),
      credentialRequest,
    ])
      .then(([nextSettings, nextCredentialStatus]) => {
        if (!mountedRef.current) {
          return;
        }

        setSettings(
          nextCredentialStatus === null
            ? nextSettings
            : applyCredentialStatus(nextSettings, nextCredentialStatus),
        );
        setCredentialStatus(nextCredentialStatus);
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
  }, [capabilities.credentials]);

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
          setSettings(
            credentialStatus === null
              ? savedSettings
              : applyCredentialStatus(
                  savedSettings,
                  credentialStatus,
                ),
          );
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
            setSettings(
              credentialStatus === null
                ? authoritativeSettings
                : applyCredentialStatus(
                    authoritativeSettings,
                    credentialStatus,
                  ),
            );
          }
        } catch {
          // The actionable save error remains visible.
        }

        return false;
      }
    },
    [capabilities, credentialStatus],
  );

  const updateAiConfiguration = useCallback(
    async (configuration: AiConfigurationUpdate): Promise<boolean> => {
      const preferencesBridge =
        preferencesDesktopBridge.getPreferencesAiBridge();
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
          setCredentialStatus({
            id: 'aiApiKey',
            state: savedSettings.ai.apiKeyConfigured
              ? 'configured'
              : 'missing',
          });
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

  const mutateCredential = useCallback(
    async (
      mutation: 'save' | 'delete',
      secret?: string,
    ): Promise<boolean> => {
      const credentialBridge =
        preferencesDesktopBridge.getCredentialBridge();

      if (!capabilities.credentials || credentialBridge === undefined) {
        setStatus('error');
        setErrorMessage(
          'Credential storage is unavailable in this window.',
        );
        return false;
      }

      const revision = updateRevisionRef.current + 1;
      updateRevisionRef.current = revision;
      setStatus('saving');
      setErrorMessage(null);

      try {
        const nextStatus =
          mutation === 'save'
            ? await credentialBridge.saveCredential(
                'aiApiKey',
                secret ?? '',
              )
            : await credentialBridge.deleteCredential('aiApiKey');

        if (
          mountedRef.current &&
          revision === updateRevisionRef.current
        ) {
          setCredentialStatus(nextStatus);
          setSettings((currentSettings) =>
            applyCredentialStatus(currentSettings, nextStatus),
          );
          setStatus('saved');
        }

        return true;
      } catch {
        if (!mountedRef.current) {
          return false;
        }

        setStatus('error');
        setErrorMessage('Your credential could not be saved. Try again.');

        try {
          const authoritativeStatus =
            await credentialBridge.getCredentialStatus('aiApiKey');

          if (
            mountedRef.current &&
            revision === updateRevisionRef.current
          ) {
            setCredentialStatus(authoritativeStatus);
            setSettings((currentSettings) =>
              applyCredentialStatus(
                currentSettings,
                authoritativeStatus,
              ),
            );
          }
        } catch {
          // The actionable mutation error remains visible.
        }

        return false;
      }
    },
    [capabilities.credentials],
  );

  const saveCredential = useCallback(
    (secret: string) => mutateCredential('save', secret),
    [mutateCredential],
  );

  const deleteCredential = useCallback(
    () => mutateCredential('delete'),
    [mutateCredential],
  );

  return {
    settings,
    capabilities,
    status,
    errorMessage,
    credentialStatus,
    update,
    updateAiConfiguration,
    saveCredential,
    deleteCredential,
  };
}
