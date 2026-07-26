export type ReleaseDownloadCount = {
  version: string;
  downloads: number;
};

export type DownloadAnalyticsOverview = {
  totalDownloads: number;
  downloadsToday: number;
  downloadsThisWeek: number;
  downloadsThisMonth: number;
  platforms: {
    mac: number;
    windows: number;
    linux: number;
  };
  releases: ReleaseDownloadCount[];
};
