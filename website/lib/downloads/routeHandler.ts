import { after, NextResponse } from "next/server";
import { createDownloadRedirect } from "./downloadFlow";
import { recordDownload } from "./downloadTracker";
import {
  resolveLatestReleaseAsset,
  type DownloadPlatform,
} from "./githubRelease";
import { getDownloadRequestMetadata } from "./requestMetadata";

export async function handleDownloadRequest(
  request: Request,
  platform: DownloadPlatform,
): Promise<NextResponse> {
  try {
    return await createDownloadRedirect(request, platform, {
      resolveLatestReleaseAsset,
      getDownloadRequestMetadata,
      recordDownload,
      scheduleAfterResponse: after,
      createRedirect(downloadUrl) {
        return NextResponse.redirect(downloadUrl, {
          status: 302,
          headers: {
            "Cache-Control": "no-store",
          },
        });
      },
      logTrackingFailure(message, context) {
        console.error(message, context);
      },
    });
  } catch (error) {
    console.error(`[ducky-download] ${platform} download failed`, error);

    return NextResponse.json(
      { error: "The download is temporarily unavailable. Please try again." },
      {
        status: 502,
        headers: {
          "Cache-Control": "no-store",
        },
      },
    );
  }
}
