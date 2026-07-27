const assert = require('node:assert/strict');
const { readFile } = require('node:fs/promises');
const path = require('node:path');
const { describe, it } = require('node:test');

const projectRoot = path.resolve(__dirname, '..');

describe('Pomodoro completion parity', () => {
  it('preserves the existing React sound, celebration, and personality flow', async () => {
    const app = await readFile(
      path.join(projectRoot, 'src', 'renderer', 'App.tsx'),
      'utf8',
    );

    assert.match(app, /bridge\.onPomodoroCompleted\(\(\) => \{/);
    assert.match(
      app,
      /notificationSoundService\.play\('pomodoro'\)/,
    );
    assert.match(app, /setCelebrating\(true\)/);
    assert.match(app, /setCelebrating\(false\)/);
    assert.match(
      app,
      /personalityService\.emitPomodoroCompletion\(sourceEventId\)/,
    );
    assert.match(
      app,
      /pendingPomodoroCompletionRef\.current = sourceEventId/,
    );
  });

  it('keeps completion in the renderer and adds no native notification path', async () => {
    const cargo = await readFile(
      path.join(projectRoot, 'src-tauri', 'Cargo.toml'),
      'utf8',
    );
    const companionCapability = await readFile(
      path.join(
        projectRoot,
        'src-tauri',
        'capabilities',
        'companion.json',
      ),
      'utf8',
    );

    assert.doesNotMatch(cargo, /tauri-plugin-notification/);
    assert.doesNotMatch(companionCapability, /notification:/);
  });
});
