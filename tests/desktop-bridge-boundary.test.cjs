const assert = require('node:assert/strict');
const { readFile, readdir } = require('node:fs/promises');
const path = require('node:path');
const { describe, it } = require('node:test');

const projectRoot = path.resolve(__dirname, '..');
const sourceRoot = path.join(projectRoot, 'src');
const rendererRoots = [
  'renderer',
  'engine',
  'personality',
  'shared',
].map((directory) => path.join(sourceRoot, directory));

const collectTypeScriptFiles = async (directory) => {
  const entries = await readdir(directory, { withFileTypes: true });
  const nestedFiles = await Promise.all(
    entries.map(async (entry) => {
      const target = path.join(directory, entry.name);

      if (entry.isDirectory()) {
        return collectTypeScriptFiles(target);
      }

      return /\.(?:ts|tsx)$/.test(entry.name) ? [target] : [];
    }),
  );

  return nestedFiles.flat();
};

const readRendererSources = async () => {
  const files = (
    await Promise.all(rendererRoots.map(collectTypeScriptFiles))
  ).flat();

  return Promise.all(
    files.map(async (file) => ({
      file: path.relative(projectRoot, file),
      source: await readFile(file, 'utf8'),
    })),
  );
};

describe('DesktopBridge renderer boundary', () => {
  it('keeps Electron and Tauri APIs outside renderer-owned code', async () => {
    const sources = await readRendererSources();

    for (const { file, source } of sources) {
      assert.doesNotMatch(
        source,
        /(?:from|import\()\s*['"]electron(?:\/[^'"]*)?['"]/,
        `${file} imports Electron directly`,
      );
      assert.doesNotMatch(
        source,
        /(?:from|import\()\s*['"]@tauri-apps\//,
        `${file} imports Tauri directly`,
      );
      assert.doesNotMatch(
        source,
        /\bwindow\.psyduck(?:Preferences)?\b/,
        `${file} accesses an Electron preload global directly`,
      );
    }
  });

  it('exposes only role-scoped bridge views to renderers', async () => {
    const sources = await readRendererSources();
    const bridgeConsumers = sources.filter(({ source }) =>
      source.includes('/desktop/DesktopBridge'),
    );
    const companionConsumers = new Set([
      'src/renderer/App.tsx',
      'src/renderer/components/PsyDuck.tsx',
      'src/renderer/hooks/usePomodoroState.ts',
      'src/renderer/hooks/useReminderNotifications.ts',
      'src/renderer/hooks/useRuntimeSettings.ts',
    ]);
    const preferencesConsumers = new Set([
      'src/renderer/PreferencesApp.tsx',
      'src/renderer/hooks/usePreferencesSettings.ts',
      'src/renderer/hooks/useUpdateStatus.ts',
    ]);

    assert.equal(bridgeConsumers.length, 8);

    for (const { file, source } of bridgeConsumers) {
      assert.doesNotMatch(
        source,
        /\bdesktopBridge\b/,
        `${file} imports the complete runtime adapter`,
      );

      if (companionConsumers.has(file)) {
        assert.match(source, /\bcompanionDesktopBridge\b/);
        assert.doesNotMatch(source, /\bpreferencesDesktopBridge\b/);
        continue;
      }

      assert.equal(preferencesConsumers.has(file), true, file);
      assert.match(source, /\bpreferencesDesktopBridge\b/);
      assert.doesNotMatch(source, /\bcompanionDesktopBridge\b/);
    }
  });

  it('keeps the complete runtime adapter private', async () => {
    const boundary = await readFile(
      path.join(sourceRoot, 'desktop', 'DesktopBridge.ts'),
      'utf8',
    );

    assert.match(
      boundary,
      /export const companionDesktopBridge: CompanionDesktopBridge/,
    );
    assert.match(
      boundary,
      /export const preferencesDesktopBridge: PreferencesDesktopBridge/,
    );
    assert.doesNotMatch(
      boundary,
      /export const (?:runtime)?desktopBridge\b/,
    );
  });

  it('dispatches companion context menus through the narrow window bridge', async () => {
    const source = await readFile(
      path.join(sourceRoot, 'renderer', 'components', 'PsyDuck.tsx'),
      'utf8',
    );

    assert.match(
      source,
      /getCompanionWindowBridge\(\)[\s\S]*showCompanionContextMenu\(\)/,
    );
    assert.doesNotMatch(
      source,
      /getCompanionBridge\(\)[\s\S]{0,100}showCompanionContextMenu\(\)/,
    );
  });

  it('routes Pomodoro UI through the narrow runtime-neutral bridge', async () => {
    const app = await readFile(
      path.join(sourceRoot, 'renderer', 'App.tsx'),
      'utf8',
    );
    const hook = await readFile(
      path.join(
        sourceRoot,
        'renderer',
        'hooks',
        'usePomodoroState.ts',
      ),
      'utf8',
    );

    assert.match(app, /getPomodoroBridge\(\)/);
    assert.match(hook, /getPomodoroBridge\(\)/);
    assert.doesNotMatch(
      hook,
      /getCompanionBridge\(\)[\s\S]*getPomodoroState\(\)/,
    );
  });

  it('registers all Pomodoro event routes before activating recovery', async () => {
    const adapter = await readFile(
      path.join(sourceRoot, 'desktop', 'tauriPomodoroBridge.ts'),
      'utf8',
    );

    for (const event of [
      'pomodoroStateChanged',
      'pomodoroCompleted',
      'customPomodoroDurationRequested',
    ]) {
      assert.match(adapter, new RegExp(`'${event}'`));
    }
    assert.match(adapter, /registeredListenerCount !== 3/);
    assert.match(
      adapter,
      /TAURI_COMMANDS\.activatePomodoroEvents/,
    );
    assert.doesNotMatch(adapter, /setInterval|setTimeout/);
  });

  it('registers all Personal Assistant event routes before activating recovery', async () => {
    const [adapter, transport] = await Promise.all(
      [
        ['desktop', 'tauriBridge.ts'],
        ['desktop', 'tauriPersonalAssistantBridge.ts'],
      ].map((segments) =>
        readFile(path.join(sourceRoot, ...segments), 'utf8'),
      ),
    );

    for (const event of [
      'userNamePanelRequested',
      'stickyMessagePanelRequested',
      'dailyPlannerPanelRequested',
    ]) {
      assert.match(transport, new RegExp(`'${event}'`));
    }
    assert.match(transport, /registeredListenerCount !== 3/);
    assert.match(
      transport,
      /TAURI_COMMANDS\.activatePersonalAssistantEvents/,
    );
    assert.match(
      adapter,
      /const companionBridge: CompanionBridge[\s\S]*getDailyPlanner[\s\S]*getCompanionBridge: \(\) => companionBridge/,
    );
    assert.doesNotMatch(transport, /setInterval|setTimeout/);
  });

  it('routes settings hooks through settings-only bridge views', async () => {
    const runtimeSettingsHook = await readFile(
      path.join(
        sourceRoot,
        'renderer',
        'hooks',
        'useRuntimeSettings.ts',
      ),
      'utf8',
    );
    const preferencesSettingsHook = await readFile(
      path.join(
        sourceRoot,
        'renderer',
        'hooks',
        'usePreferencesSettings.ts',
      ),
      'utf8',
    );

    assert.match(
      runtimeSettingsHook,
      /getRuntimeSettingsBridge\(\)/,
    );
    assert.doesNotMatch(runtimeSettingsHook, /getCompanionBridge\(\)/);
    assert.match(
      preferencesSettingsHook,
      /getPreferencesSettingsBridge\(\)/,
    );
    assert.match(
      preferencesSettingsHook,
      /updateAiConfiguration[\s\S]*getPreferencesAiBridge\(\)/,
    );

    const contracts = await readFile(
      path.join(sourceRoot, 'desktop', 'contracts.ts'),
      'utf8',
    );
    const sharedTypes = await readFile(
      path.join(sourceRoot, 'shared', 'types.ts'),
      'utf8',
    );
    assert.match(contracts, /getCredentialBridge/);
    assert.match(
      sharedTypes,
      /interface CredentialBridge[\s\S]*getCredentialStatus[\s\S]*saveCredential[\s\S]*deleteCredential/,
    );
    assert.doesNotMatch(
      sharedTypes,
      /interface CredentialBridge[\s\S]*loadCredential/,
    );
    assert.match(
      preferencesSettingsHook,
      /getCredentialBridge\(\)[\s\S]*saveCredential\([\s\S]*deleteCredential\(/,
    );
  });

  it('routes whole AI responses through the narrow runtime-neutral bridge', async () => {
    const [application, contracts, adapter, commands] =
      await Promise.all(
        [
          ['renderer', 'App.tsx'],
          ['desktop', 'contracts.ts'],
          ['desktop', 'tauriBridge.ts'],
          ['desktop', 'tauriCommands.ts'],
        ].map((segments) =>
          readFile(path.join(sourceRoot, ...segments), 'utf8'),
        ),
      );

    assert.match(application, /getCompanionAiBridge\(\)\?\.askAI/);
    assert.doesNotMatch(
      application,
      /getCompanionBridge\(\)\?\.askAI/,
    );
    assert.match(contracts, /getCompanionAiBridge/);
    assert.match(adapter, /TAURI_COMMANDS\.askAI/);
    assert.match(commands, /askAI:\s*'ask_ai'/);
    for (const source of [application, contracts, adapter, commands]) {
      assert.doesNotMatch(source, /streamAI|aiToken|cancelAI/);
    }
  });

  it('routes updater status and checks through the exact Preferences bridge', async () => {
    const [hook, contracts, adapter, commands] = await Promise.all(
      [
        ['renderer', 'hooks', 'useUpdateStatus.ts'],
        ['desktop', 'contracts.ts'],
        ['desktop', 'tauriBridge.ts'],
        ['desktop', 'tauriCommands.ts'],
      ].map((segments) =>
        readFile(path.join(sourceRoot, ...segments), 'utf8'),
      ),
    );

    assert.match(hook, /getPreferencesUpdateBridge\(\)/);
    assert.doesNotMatch(hook, /getPreferencesBridge\(\)/);
    assert.match(contracts, /getPreferencesUpdateBridge/);
    assert.match(
      adapter,
      /const preferencesUpdateBridge[\s\S]*TAURI_COMMANDS\.getUpdateStatus[\s\S]*TAURI_COMMANDS\.checkForUpdates[\s\S]*'updateStatusChanged'/,
    );
    assert.match(
      adapter,
      /'updateStatusChanged',[\s\S]*listener,[\s\S]*TAURI_COMMANDS\.getUpdateStatus[\s\S]*\.then\(listener\)/,
    );
    assert.match(commands, /getUpdateStatus:\s*'get_update_status'/);
    assert.match(commands, /checkForUpdates:\s*'check_for_updates'/);
    for (const source of [hook, contracts, adapter, commands]) {
      assert.doesNotMatch(
        source,
        /downloadUpdate|installUpdate|restartToUpdate/,
      );
    }
  });

  it('routes reminder UI through the exact reminder bridge', async () => {
    const [application, notifications, contracts, adapter] =
      await Promise.all(
        [
          ['renderer', 'App.tsx'],
          ['renderer', 'hooks', 'useReminderNotifications.ts'],
          ['desktop', 'contracts.ts'],
          ['desktop', 'tauriBridge.ts'],
        ].map((segments) =>
          readFile(path.join(sourceRoot, ...segments), 'utf8'),
        ),
      );

    for (const source of [application, notifications]) {
      assert.match(source, /getReminderBridge\(\)/);
    }
    assert.doesNotMatch(
      notifications,
      /getCompanionBridge\(\)/,
    );
    assert.match(contracts, /getReminderBridge/);
    assert.match(
      adapter,
      /const reminderBridge[\s\S]*createReminder[\s\S]*updateReminder[\s\S]*deleteReminder[\s\S]*getReminder[\s\S]*listReminders[\s\S]*markReminderCompleted/,
    );
  });

  it('keeps deferred Preferences domains runtime-capability gated', async () => {
    const [contracts, electronAdapter, tauriAdapter, preferencesUi] =
      await Promise.all(
        [
          ['desktop', 'contracts.ts'],
          ['desktop', 'electronBridge.ts'],
          ['desktop', 'tauriBridge.ts'],
          ['renderer', 'PreferencesApp.tsx'],
        ].map((segments) =>
          readFile(path.join(sourceRoot, ...segments), 'utf8'),
        ),
      );

    assert.match(contracts, /getPreferencesSettingsCapabilities/);
    assert.match(contracts, /getPreferencesAiBridge/);
    assert.match(
      electronAdapter,
      /water: true,[\s\S]*updates: true,[\s\S]*ai: true,[\s\S]*aiModelExplorer: true,[\s\S]*credentials: true/,
    );
    assert.match(
      tauriAdapter,
      /water: true,[\s\S]*updates: true,[\s\S]*ai: true,[\s\S]*aiModelExplorer: true,[\s\S]*credentials: true/,
    );
    assert.match(
      tauriAdapter,
      /getCredentialBridge: \(\) => credentialBridge/,
    );
    assert.match(
      tauriAdapter,
      /getPreferencesAiBridge: \(\) => preferencesAiBridge/,
    );
    assert.match(
      preferencesUi,
      /waterSettingsUnavailable[\s\S]*updateSettingsUnavailable[\s\S]*aiSettingsUnavailable/,
    );
    assert.match(
      preferencesUi,
      /ref=\{apiKeyInputRef\}[\s\S]*defaultValue=""[\s\S]*credentialSettingsUnavailable/,
    );
    assert.match(
      preferencesUi,
      /capabilities\.water && capabilities\.updates[\s\S]*General, sound, hydration, and update changes save automatically\./,
    );
    assert.doesNotMatch(
      preferencesUi,
      /useState<[^>]*string[^>]*>\([^)]*apiKey/i,
    );
  });
});
