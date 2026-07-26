import "server-only";

import { getServerSupabaseClient } from "../supabase/server";
import type {
  DownloadAnalyticsOverview,
  ReleaseDownloadCount,
} from "./types";

const overviewFunction = "get_download_analytics_overview";

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object";
}

function readCount(value: unknown, field: string): number {
  const count =
    typeof value === "number"
      ? value
      : typeof value === "string"
        ? Number(value)
        : Number.NaN;

  if (!Number.isFinite(count) || count < 0) {
    throw new Error(`Invalid analytics count for ${field}`);
  }

  return Math.trunc(count);
}

function readReleaseCounts(value: unknown): ReleaseDownloadCount[] {
  if (!Array.isArray(value)) {
    throw new Error("Invalid release analytics response");
  }

  return value.map((release, index) => {
    if (!isRecord(release) || typeof release.version !== "string") {
      throw new Error(`Invalid release analytics item at index ${index}`);
    }

    return {
      version: release.version,
      downloads: readCount(release.downloads, `releases[${index}]`),
    };
  });
}

function parseOverview(value: unknown): DownloadAnalyticsOverview {
  if (!isRecord(value) || !isRecord(value.platforms)) {
    throw new Error("Invalid download analytics response");
  }

  const platforms = value.platforms;

  return {
    totalDownloads: readCount(value.totalDownloads, "totalDownloads"),
    downloadsToday: readCount(value.downloadsToday, "downloadsToday"),
    downloadsThisWeek: readCount(
      value.downloadsThisWeek,
      "downloadsThisWeek",
    ),
    downloadsThisMonth: readCount(
      value.downloadsThisMonth,
      "downloadsThisMonth",
    ),
    platforms: {
      mac: readCount(platforms.mac, "platforms.mac"),
      windows: readCount(platforms.windows, "platforms.windows"),
      linux: readCount(platforms.linux, "platforms.linux"),
    },
    releases: readReleaseCounts(value.releases),
  };
}

export async function getDownloadAnalyticsOverview(): Promise<DownloadAnalyticsOverview> {
  const { data, error } =
    await getServerSupabaseClient().rpc(overviewFunction);

  if (error) {
    throw new Error(`Supabase analytics query failed: ${error.message}`);
  }

  return parseOverview(data);
}
