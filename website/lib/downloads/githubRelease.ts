export type DownloadPlatform = "mac" | "windows" | "linux";

type GitHubReleaseAsset = {
  id: number;
  name: string;
  state: string;
  browser_download_url: string;
};

type GitHubRelease = {
  id: number;
  tag_name: string;
  assets: GitHubReleaseAsset[];
};

export type ResolvedReleaseAsset = {
  releaseTag: string;
  asset: {
    id: number;
    name: string;
    downloadUrl: string;
  };
};

const latestReleaseUrl =
  "https://api.github.com/repos/amanbotx2-fr/Ducky/releases/latest";
const releaseDownloadUrlPrefix =
  "https://github.com/amanbotx2-fr/Ducky/releases/download/";
const releaseCacheTtlMs = 5 * 60 * 1000;
const tauriAssetPrefix = "ducky-tauri-";

let cachedRelease:
  | {
      value: GitHubRelease;
      expiresAt: number;
    }
  | undefined;
let pendingRelease: Promise<GitHubRelease> | undefined;

function isGitHubRelease(value: unknown): value is GitHubRelease {
  if (!value || typeof value !== "object") {
    return false;
  }

  const release = value as Partial<GitHubRelease>;
  return (
    typeof release.id === "number" &&
    typeof release.tag_name === "string" &&
    Array.isArray(release.assets)
  );
}

async function fetchLatestRelease(): Promise<GitHubRelease> {
  const response = await fetch(latestReleaseUrl, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "Ducky-Website",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    next: {
      revalidate: releaseCacheTtlMs / 1000,
    },
  });

  if (!response.ok) {
    throw new Error(
      `GitHub latest release request failed with status ${response.status}`,
    );
  }

  const release: unknown = await response.json();
  if (!isGitHubRelease(release)) {
    throw new Error("GitHub returned an invalid latest release response");
  }

  return release;
}

async function getLatestRelease(): Promise<GitHubRelease> {
  const now = Date.now();
  if (cachedRelease && cachedRelease.expiresAt > now) {
    return cachedRelease.value;
  }

  pendingRelease ??= fetchLatestRelease();

  try {
    const release = await pendingRelease;
    cachedRelease = {
      value: release,
      expiresAt: Date.now() + releaseCacheTtlMs,
    };
    return release;
  } finally {
    pendingRelease = undefined;
  }
}

function getAssetScore(platform: DownloadPlatform, name: string): number {
  const normalizedName = name.toLowerCase();

  if (platform === "mac") {
    return normalizedName.includes("universal") ? 20 : 0;
  }

  if (platform === "windows") {
    return (
      (normalizedName.includes("setup") ? 20 : 0) +
      (normalizedName.includes("x64") ? 5 : 0)
    );
  }

  return /x86[_-]64|amd64/.test(normalizedName) ? 10 : 0;
}

function getPreferredExtension(platform: DownloadPlatform): string {
  if (platform === "mac") {
    return ".dmg";
  }

  if (platform === "windows") {
    return ".exe";
  }

  return ".appimage";
}

function selectAsset(
  release: GitHubRelease,
  platform: DownloadPlatform,
): GitHubReleaseAsset {
  const extension = getPreferredExtension(platform);
  const candidates = release.assets
    .filter(
      (asset) =>
        asset.state === "uploaded" &&
        asset.name.toLowerCase().endsWith(extension) &&
        asset.browser_download_url.startsWith(releaseDownloadUrlPrefix),
    )
    .sort(
      (left, right) =>
        getAssetScore(platform, right.name) -
          getAssetScore(platform, left.name) ||
        left.name.localeCompare(right.name),
    );

  const tauriCandidates = candidates.filter((asset) =>
    asset.name.toLowerCase().startsWith(tauriAssetPrefix),
  );
  const asset = tauriCandidates[0];
  if (!asset) {
    throw new Error(
      `The latest GitHub release has no Tauri ${extension} asset for ${platform}`,
    );
  }

  return asset;
}

export async function resolveLatestReleaseAsset(
  platform: DownloadPlatform,
): Promise<ResolvedReleaseAsset> {
  const release = await getLatestRelease();
  const asset = selectAsset(release, platform);

  return {
    releaseTag: release.tag_name,
    asset: {
      id: asset.id,
      name: asset.name,
      downloadUrl: asset.browser_download_url,
    },
  };
}
