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
      /getCompanionSettingsBridge\(\)/,
    );
    assert.doesNotMatch(runtimeSettingsHook, /getCompanionBridge\(\)/);
    assert.match(
      preferencesSettingsHook,
      /getPreferencesSettingsBridge\(\)/,
    );
    assert.match(
      preferencesSettingsHook,
      /updateAiConfiguration[\s\S]*getPreferencesBridge\(\)/,
    );
  });
});
