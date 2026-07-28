import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";

const releaseDirectory = resolve(process.argv[2] ?? "release-artifacts");
const releaseJsonPath = resolve(process.argv[3] ?? "release-draft.json");
const tag = process.env.RELEASE_TAG;

try {
  if (!tag) {
    throw new Error("RELEASE_TAG is required.");
  }
  if (!existsSync(releaseJsonPath)) {
    throw new Error(`Draft release response not found: ${releaseJsonPath}.`);
  }

  const release = JSON.parse(readFileSync(releaseJsonPath, "utf8"));
  if (release.tag_name !== tag || release.draft !== true) {
    throw new Error(`GitHub release ${tag} is not the expected draft.`);
  }

  const local = readdirSync(releaseDirectory)
    .filter((file) => statSync(resolve(releaseDirectory, file)).isFile())
    .sort();
  const remoteAssets = Array.isArray(release.assets) ? release.assets : [];
  const remote = remoteAssets.map((asset) => asset.name).sort();
  if (JSON.stringify(local) !== JSON.stringify(remote)) {
    throw new Error(
      `Draft asset inventory mismatch.\nLocal: ${local.join(", ")}\nRemote: ${remote.join(", ")}`,
    );
  }

  for (const asset of remoteAssets) {
    const localSize = statSync(resolve(releaseDirectory, asset.name)).size;
    if (asset.state !== "uploaded" || asset.size !== localSize) {
      throw new Error(
        `Draft asset ${asset.name} is not fully uploaded (${asset.state}, ${asset.size}/${localSize} bytes).`,
      );
    }
  }

  console.log(`Verified ${remote.length} assets on draft release ${tag}.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
