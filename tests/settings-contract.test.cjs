const assert = require('node:assert/strict');
const { mkdtemp, readFile, rm, writeFile } = require('node:fs/promises');
const { tmpdir } = require('node:os');
const { join } = require('node:path');
const { describe, test } = require('node:test');

const { SettingsService } = require('../dist/main/SettingsService.js');

const credentialManager = {
  decrypt: () => {
    throw new Error('The shared fixture has no credential.');
  },
  encrypt: () => {
    throw new Error('The shared fixture does not update credentials.');
  },
  isEncryptionAvailable: () => false,
};

describe('cross-runtime settings contract', () => {
  test('loads the shared Electron settings fixture without migration loss', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'ducky-settings-contract-'));
    const filePath = join(directory, 'settings.json');
    const fixture = await readFile(
      join(
        __dirname,
        'fixtures',
        'settings',
        'electron-current.json',
      ),
      'utf8',
    );

    try {
      await writeFile(filePath, fixture, 'utf8');
      const service = new SettingsService(filePath, credentialManager);
      const settings = await service.load();

      assert.equal(settings.userName, 'Aman');
      assert.equal(settings.stickyMessage, 'Ship carefully');
      assert.deepEqual(settings.general, {
        alwaysOnTop: false,
        launchAtStartup: true,
        eyeTracking: true,
      });
      assert.deepEqual(settings.notificationSounds, {
        enabled: true,
        sound: 'zen-chime',
        volume: 42,
      });
      assert.equal(settings.ai.apiKeyConfigured, false);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });
});
