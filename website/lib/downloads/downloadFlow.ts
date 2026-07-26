import type { DownloadEvent } from "./downloadTracker";
import type {
  DownloadPlatform,
  ResolvedReleaseAsset,
} from "./githubRelease";
import type { DownloadRequestMetadata } from "./requestMetadata";

export const downloadTrackingTimeoutMs = 400;

type DownloadFlowDependencies<ResponseType> = {
  resolveLatestReleaseAsset(
    platform: DownloadPlatform,
  ): Promise<ResolvedReleaseAsset>;
  getDownloadRequestMetadata(request: Request): DownloadRequestMetadata;
  recordDownload(event: DownloadEvent): Promise<void>;
  scheduleAfterResponse(task: () => Promise<void>): void;
  createRedirect(downloadUrl: string): ResponseType;
  logTrackingFailure(
    message: string,
    context: {
      error: unknown;
      route: string;
      platform: DownloadPlatform;
      version: string;
    },
  ): void;
  trackingTimeoutMs?: number;
};

class DownloadTrackingTimeoutError extends Error {
  constructor(timeoutMs: number) {
    super(`Download tracking exceeded ${timeoutMs}ms`);
    this.name = "DownloadTrackingTimeoutError";
  }
}

async function runTrackingTask<ResponseType>(
  event: DownloadEvent,
  dependencies: DownloadFlowDependencies<ResponseType>,
): Promise<void> {
  const timeoutMs =
    dependencies.trackingTimeoutMs ?? downloadTrackingTimeoutMs;
  let timeout: ReturnType<typeof setTimeout> | undefined;

  try {
    await Promise.race([
      dependencies.recordDownload(event),
      new Promise<never>((_, reject) => {
        timeout = setTimeout(
          () => reject(new DownloadTrackingTimeoutError(timeoutMs)),
          timeoutMs,
        );
      }),
    ]);
  } catch (error) {
    dependencies.logTrackingFailure("[ducky-download] Tracking failed", {
      error,
      route: `/download/${event.platform}`,
      platform: event.platform,
      version: event.releaseTag,
    });
  } finally {
    if (timeout) {
      clearTimeout(timeout);
    }
  }
}

export async function createDownloadRedirect<ResponseType>(
  request: Request,
  platform: DownloadPlatform,
  dependencies: DownloadFlowDependencies<ResponseType>,
): Promise<ResponseType> {
  const { releaseTag, asset } =
    await dependencies.resolveLatestReleaseAsset(platform);
  const metadata = dependencies.getDownloadRequestMetadata(request);
  const event: DownloadEvent = {
    platform,
    releaseTag,
    ...metadata,
    assetName: asset.name,
    occurredAt: new Date().toISOString(),
  };

  try {
    dependencies.scheduleAfterResponse(() =>
      runTrackingTask(event, dependencies),
    );
  } catch (error) {
    dependencies.logTrackingFailure(
      "[ducky-download] Tracking could not be scheduled",
      {
        error,
        route: `/download/${platform}`,
        platform,
        version: releaseTag,
      },
    );
  }

  return dependencies.createRedirect(asset.downloadUrl);
}
