const assert = require('node:assert/strict');
const { describe, test } = require('node:test');

const {
  TAURI_MIGRATION_DIALOG,
  TauriMigrationFlow,
} = require('../dist/main/TauriMigrationFlow.js');

const RELEASES_URL =
  'https://github.com/amanbotx2-fr/Ducky/releases';

const available = (version = '2.0.0') => ({
  phase: 'available',
  currentVersion: '1.1.0',
  availableVersion: version,
});

const createFlow = ({
  currentVersion = '1.1.0',
  response = 1,
  showError,
  openError,
} = {}) => {
  const calls = {
    dialogs: [],
    opened: [],
    failures: [],
  };
  const flow = new TauriMigrationFlow({
    currentVersion,
    releasePageUrl: RELEASES_URL,
    showDialog: async (options) => {
      calls.dialogs.push(options);

      if (showError !== undefined) {
        throw showError;
      }

      return { response };
    },
    openExternal: async (url) => {
      calls.opened.push(url);

      if (openError !== undefined) {
        throw openError;
      }
    },
    logFailure: (operation, error) => {
      calls.failures.push({ operation, error });
    },
  });

  return { calls, flow };
};

describe('Electron to Tauri migration flow', () => {
  test('offers the exact manual migration dialog once per available version', async () => {
    const { calls, flow } = createFlow();

    await flow.handleStatus(available());
    await flow.handleStatus(available());

    assert.deepEqual(calls.dialogs, [TAURI_MIGRATION_DIALOG]);
    assert.deepEqual(TAURI_MIGRATION_DIALOG.buttons, [
      'Download PsyDuck 2.0',
      'Remind Me Later',
    ]);
    assert.match(TAURI_MIGRATION_DIALOG.message, /faster native engine/);
    assert.match(TAURI_MIGRATION_DIALOG.detail, /one-time manual upgrade/);
    assert.match(
      TAURI_MIGRATION_DIALOG.detail,
      /Future updates become automatic/,
    );
    assert.deepEqual(calls.opened, []);
  });

  test('opens only the configured official release page after Download', async () => {
    const { calls, flow } = createFlow({ response: 0 });

    await flow.handleStatus(available('v2.0.0'));

    assert.deepEqual(calls.opened, [RELEASES_URL]);
    assert.deepEqual(calls.failures, []);
  });

  test('ignores ordinary Electron updates and clients already on Tauri', async () => {
    const electron = createFlow();
    const tauri = createFlow({ currentVersion: '2.0.0' });

    await electron.flow.handleStatus(available('1.2.0'));
    await electron.flow.handleStatus({
      phase: 'not-available',
      currentVersion: '1.1.0',
    });
    await tauri.flow.handleStatus(available('2.1.0'));

    assert.deepEqual(electron.calls.dialogs, []);
    assert.deepEqual(tauri.calls.dialogs, []);
  });

  test('contains dialog and browser failures without changing update status', async () => {
    const dialogFailure = createFlow({
      response: 0,
      showError: new Error('dialog unavailable'),
    });
    const browserFailure = createFlow({
      response: 0,
      openError: new Error('browser unavailable'),
    });
    const status = available();

    await dialogFailure.flow.handleStatus(status);
    await browserFailure.flow.handleStatus(status);

    assert.equal(
      dialogFailure.calls.failures[0].operation,
      'migration_dialog_failed',
    );
    assert.equal(
      browserFailure.calls.failures[0].operation,
      'migration_release_open_failed',
    );
    assert.deepEqual(status, available());
  });
});
