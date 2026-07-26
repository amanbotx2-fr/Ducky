import { NextResponse } from "next/server";
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
    const { releaseTag, asset } =
      await resolveLatestReleaseAsset(platform);
    const metadata = getDownloadRequestMetadata(request);

    await recordDownload({
      platform,
      releaseTag,
      ...metadata,
      assetName: asset.name,
      occurredAt: new Date().toISOString(),
    });

    return NextResponse.redirect(asset.downloadUrl, {
      status: 302,
      headers: {
        "Cache-Control": "no-store",
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
