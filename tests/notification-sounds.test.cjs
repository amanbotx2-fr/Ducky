const assert = require('node:assert/strict');
const {
  readFile,
  stat,
} = require('node:fs/promises');
const { join } = require('node:path');
const { describe, test } = require('node:test');

const {
  createDefaultSettings,
  parsePreferencesSettingsPatch,
  parseSettingsPatch,
  toPreferencesSettings,
  toRuntimeSettings,
} = require('../dist/shared/settings.js');
const {
  DEFAULT_NOTIFICATION_SOUND_SETTINGS,
  NOTIFICATION_SOUND_OPTIONS,
  parseNotificationSoundSettingsPatch,
} = require('../dist/shared/notificationSounds.js');

describe('notification sound settings', () => {
  test('uses safe defaults in runtime and Preferences settings', () => {
    const settings = createDefaultSettings();

    assert.deepEqual(
      settings.notificationSounds,
      DEFAULT_NOTIFICATION_SOUND_SETTINGS,
    );
    assert.deepEqual(
      toRuntimeSettings(settings).notificationSounds,
      DEFAULT_NOTIFICATION_SOUND_SETTINGS,
    );
    assert.deepEqual(
      toPreferencesSettings(settings).notificationSounds,
      DEFAULT_NOTIFICATION_SOUND_SETTINGS,
    );
  });

  test('accepts supported values and rejects malformed patches', () => {
    const validPatch = {
      enabled: false,
      sound: 'zen-chime',
      volume: 28,
    };

    assert.deepEqual(
      parseNotificationSoundSettingsPatch(validPatch),
      validPatch,
    );
    assert.deepEqual(parseSettingsPatch({
      notificationSounds: validPatch,
    }), {
      notificationSounds: validPatch,
    });
    assert.deepEqual(parsePreferencesSettingsPatch({
      notificationSounds: validPatch,
    }), {
      notificationSounds: validPatch,
    });

    for (const invalidPatch of [
      { sound: 'air-horn' },
      { volume: -1 },
      { volume: 101 },
      { volume: 27.5 },
      { enabled: 'yes' },
      { loop: true },
    ]) {
      assert.equal(
        parseNotificationSoundSettingsPatch(invalidPatch),
        null,
      );
    }
  });

});

describe('built-in notification sound pack', () => {
  test('contains one compact PCM WAV asset for every selectable sound', async () => {
    for (const sound of NOTIFICATION_SOUND_OPTIONS) {
      const filePath = join(
        __dirname,
        '..',
        'assets',
        'sounds',
        `${sound.id}.wav`,
      );
      const [contents, metadata] = await Promise.all([
        readFile(filePath),
        stat(filePath),
      ]);

      assert.equal(contents.subarray(0, 4).toString('ascii'), 'RIFF');
      assert.equal(contents.subarray(8, 12).toString('ascii'), 'WAVE');
      assert.ok(metadata.size > 44);
      assert.ok(metadata.size < 64 * 1024);
    }
  });
});
