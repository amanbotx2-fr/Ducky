const repositoryUrl = "https://github.com/amanbotx2-fr/Ducky";
const releaseVersion = "1.1.0";
const releaseTag = `v${releaseVersion}`;
const releaseAssetBase = `${repositoryUrl}/releases/download/${releaseTag}`;

export const releaseAssets = {
  version: releaseVersion,
  macos: `${releaseAssetBase}/Ducky-${releaseVersion}-universal.dmg`,
  windows: `${releaseAssetBase}/Ducky-Setup-${releaseVersion}-x64.exe`,
  linux: `${releaseAssetBase}/Ducky-${releaseVersion}-x86_64.AppImage`,
} as const;

export const supportLinks = {
  documentation: `${repositoryUrl}/tree/main/docs`,
  community: `${repositoryUrl}/issues`,
  issue: `${repositoryUrl}/issues/new`,
  repository: repositoryUrl,
} as const;
