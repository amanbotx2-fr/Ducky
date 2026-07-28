import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";

import {
  releaseDownloadUrl,
  tauriArtifactNames,
  validateReleaseVersions,
} from "./release-contract.mjs";

const releaseDirectory = resolve(process.argv[2] ?? "release-artifacts");
const tag = process.env.RELEASE_TAG;
const pubDate = process.env.RELEASE_PUB_DATE;

function readSignature(artifactName) {
  const signaturePath = resolve(releaseDirectory, `${artifactName}.sig`);
  if (!existsSync(signaturePath) || statSync(signaturePath).size === 0) {
    throw new Error(`Missing updater signature ${artifactName}.sig.`);
  }

  const signature = readFileSync(signaturePath, "utf8").trim();
  if (
    signature.length < 80 ||
    !/^[A-Za-z0-9+/]+={0,2}$/.test(signature) ||
    Buffer.from(signature, "base64").length === 0
  ) {
    throw new Error(`${artifactName}.sig is not valid base64 signature data.`);
  }
  return signature;
}

try {
  if (!tag) {
    throw new Error("RELEASE_TAG is required.");
  }
  if (!pubDate || Number.isNaN(Date.parse(pubDate))) {
    throw new Error("RELEASE_PUB_DATE must be a valid ISO-8601 date.");
  }
  if (
    !existsSync(releaseDirectory) ||
    !statSync(releaseDirectory).isDirectory()
  ) {
    throw new Error(`Release directory does not exist: ${releaseDirectory}.`);
  }

  const version = validateReleaseVersions({ tag });
  const names = tauriArtifactNames(version);
  const files = readdirSync(releaseDirectory);
  const updaterArtifacts = {
    macos: names.macos.updater,
    windows: names.windows.installer,
    linux: names.linux.installer,
  };

  for (const artifact of Object.values(updaterArtifacts)) {
    if (!files.includes(artifact)) {
      throw new Error(`Missing staged updater artifact ${artifact}.`);
    }
  }

  const platform = (artifact) => ({
    url: releaseDownloadUrl(tag, artifact),
    signature: readSignature(artifact),
  });
  const manifest = {
    version,
    notes: `Ducky ${tag}`,
    pub_date: new Date(pubDate).toISOString(),
    platforms: {
      "darwin-aarch64": platform(updaterArtifacts.macos),
      "darwin-x86_64": platform(updaterArtifacts.macos),
      "windows-x86_64": platform(updaterArtifacts.windows),
      "linux-x86_64": platform(updaterArtifacts.linux),
    },
  };

  writeFileSync(
    resolve(releaseDirectory, "latest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  console.log(`Generated signed updater manifest for ${tag}.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
