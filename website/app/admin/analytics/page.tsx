import type { Metadata } from "next";
import { AnalyticsDashboard } from "../../../components/Analytics/AnalyticsDashboard";
import { getDownloadAnalyticsOverview } from "../../../lib/analytics/downloadAnalytics";
import type { DownloadAnalyticsOverview } from "../../../lib/analytics/types";

export const dynamic = "force-dynamic";

export const metadata: Metadata = {
  title: "Download Analytics — Ducky",
  description: "Internal overview of Ducky download activity.",
  robots: {
    follow: false,
    index: false,
  },
};

const unavailableOverview: DownloadAnalyticsOverview = {
  totalDownloads: 0,
  downloadsToday: 0,
  downloadsThisWeek: 0,
  downloadsThisMonth: 0,
  platforms: {
    mac: 0,
    windows: 0,
    linux: 0,
  },
  releases: [],
};

export default async function AnalyticsPage() {
  let data = unavailableOverview;
  let isAvailable = false;

  try {
    data = await getDownloadAnalyticsOverview();
    isAvailable = true;
  } catch (error) {
    console.error("[ducky-analytics] Overview query failed", {
      message: error instanceof Error ? error.message : "Unknown error",
    });
  }

  return <AnalyticsDashboard data={data} isAvailable={isAvailable} />;
}
