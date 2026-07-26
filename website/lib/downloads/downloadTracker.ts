import "server-only";

import { getServerSupabaseClient } from "../supabase/server";

export type DownloadEvent = {
  platform: "mac" | "windows" | "linux";
  releaseTag: string;
  browser: string | null;
  operatingSystem: string | null;
  referrer: string | null;
  country: string | null;
  assetName: string;
  occurredAt: string;
};

export interface DownloadTracker {
  record(event: DownloadEvent): Promise<void>;
}

class SupabaseDownloadTracker implements DownloadTracker {
  async record(event: DownloadEvent): Promise<void> {
    const { error } = await getServerSupabaseClient().from("downloads").insert({
      platform: event.platform,
      version: event.releaseTag,
      browser: event.browser,
      operating_system: event.operatingSystem,
      referrer: event.referrer,
      country: event.country,
      asset_name: event.assetName,
    });

    if (error) {
      throw new Error(`Supabase download insert failed: ${error.message}`);
    }
  }
}

const tracker: DownloadTracker = new SupabaseDownloadTracker();

export async function recordDownload(event: DownloadEvent): Promise<void> {
  try {
    await tracker.record(event);
  } catch (error) {
    console.error("[ducky-download] Tracking failed", error);
  }
}
