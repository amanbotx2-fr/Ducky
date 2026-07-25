const repositoryUrl = "https://github.com/amanbotx2-fr/Ducky";

export const downloadLinks = {
  mac: "/download/mac",
  windows: "/download/windows",
  linux: "/download/linux",
} as const;

export const supportLinks = {
  documentation: `${repositoryUrl}/tree/main/docs`,
  community: `${repositoryUrl}/issues`,
  features: `${repositoryUrl}#features`,
  issue: `${repositoryUrl}/issues/new`,
  repository: repositoryUrl,
} as const;
