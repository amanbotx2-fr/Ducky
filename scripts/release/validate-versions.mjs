import { validateReleaseVersions } from "./release-contract.mjs";

try {
  const version = validateReleaseVersions({
    tag: process.env.RELEASE_TAG || undefined,
  });
  console.log(`Validated release version ${version} across Node and Tauri.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
