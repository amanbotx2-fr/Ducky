import type { UpdateStatus } from '../shared/updates';

export interface MigrationDialogOptions {
  readonly type: 'info';
  readonly title: string;
  readonly message: string;
  readonly detail: string;
  readonly buttons: readonly [string, string];
  readonly defaultId: number;
  readonly cancelId: number;
  readonly noLink: boolean;
}

export interface TauriMigrationFlowOptions {
  readonly currentVersion: string;
  readonly releasePageUrl: string;
  readonly showDialog: (
    options: MigrationDialogOptions,
  ) => Promise<{ readonly response: number }>;
  readonly openExternal: (url: string) => Promise<void>;
  readonly logFailure?: (
    operation: string,
    error: unknown,
  ) => void;
}

export const TAURI_MIGRATION_DIALOG: MigrationDialogOptions = {
  type: 'info',
  title: 'PsyDuck 2.0',
  message: 'PsyDuck has been rebuilt on a faster native engine.',
  detail:
    'This is a one-time manual upgrade.\n\nFuture updates become automatic after installing PsyDuck 2.0.',
  buttons: ['Download PsyDuck 2.0', 'Remind Me Later'],
  defaultId: 0,
  cancelId: 1,
  noLink: true,
};

const getMajorVersion = (value: string): number | null => {
  const match = /^v?(\d+)(?:\.|$)/u.exec(value.trim());

  if (match === null) {
    return null;
  }

  const major = Number(match[1]);
  return Number.isSafeInteger(major) ? major : null;
};

export class TauriMigrationFlow {
  private readonly currentMajor: number | null;
  private readonly releasePageUrl: string;
  private readonly showDialog: TauriMigrationFlowOptions['showDialog'];
  private readonly openExternal: TauriMigrationFlowOptions['openExternal'];
  private readonly logFailure: NonNullable<
    TauriMigrationFlowOptions['logFailure']
  >;
  private readonly promptedVersions = new Set<string>();

  public constructor(options: TauriMigrationFlowOptions) {
    this.currentMajor = getMajorVersion(options.currentVersion);
    this.releasePageUrl = options.releasePageUrl;
    this.showDialog = options.showDialog;
    this.openExternal = options.openExternal;
    this.logFailure =
      options.logFailure ??
      ((operation) => {
        console.warn(`[updates] ${operation}`);
      });
  }

  public async handleStatus(status: UpdateStatus): Promise<void> {
    if (
      status.phase !== 'available' ||
      this.currentMajor === null ||
      this.currentMajor >= 2 ||
      getMajorVersion(status.availableVersion) !== 2
    ) {
      return;
    }

    const availableVersion = status.availableVersion.trim();

    if (this.promptedVersions.has(availableVersion)) {
      return;
    }

    this.promptedVersions.add(availableVersion);

    let response: number;

    try {
      response = (
        await this.showDialog(TAURI_MIGRATION_DIALOG)
      ).response;
    } catch (error) {
      this.logFailure('migration_dialog_failed', error);
      return;
    }

    if (response !== 0) {
      return;
    }

    try {
      await this.openExternal(this.releasePageUrl);
    } catch (error) {
      this.logFailure('migration_release_open_failed', error);
    }
  }
}
