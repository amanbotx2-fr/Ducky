import {
  createHash,
} from "node:crypto";
import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
} from "node:fs";
import { basename, resolve } from "node:path";

import {
  releaseDownloadUrl,
  tauriArtifactNames,
  validateReleaseVersions,
} from "./release-contract.mjs";

const releaseDirectory = resolve(process.argv[2] ?? "release-artifacts");
const tag = process.env.RELEASE_TAG;

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

try {
  if (!tag) {
    throw new Error("RELEASE_TAG is required.");
  }
  if (
    !existsSync(releaseDirectory) ||
    !statSync(releaseDirectory).isDirectory()
  ) {
    throw new Error(`Release directory does not exist: ${releaseDirectory}.`);
  }

  const version = validateReleaseVersions({ tag });
  const names = tauriArtifactNames(version);
  const expectedTauri = [
    names.macos.installer,
    names.macos.updater,
    `${names.macos.updater}.sig`,
    names.windows.installer,
    `${names.windows.installer}.sig`,
    names.windows.msi,
    names.linux.installer,
    `${names.linux.installer}.sig`,
    names.linux.deb,
    "latest.json",
  ];
  const files = readdirSync(releaseDirectory).filter((file) =>
    statSync(resolve(releaseDirectory, file)).isFile(),
  );
  const duplicates = files.filter(
    (file, index) => files.indexOf(file) !== index,
  );
  if (duplicates.length > 0) {
    throw new Error(`Duplicate release asset names: ${duplicates.join(", ")}.`);
  }

  for (const file of expectedTauri) {
    const path = resolve(releaseDirectory, file);
    if (!existsSync(path) || statSync(path).size === 0) {
      throw new Error(`Missing or empty Tauri release asset ${file}.`);
    }
  }

  const manifest = JSON.parse(
    readFileSync(resolve(releaseDirectory, "latest.json"), "utf8"),
  );
  if (manifest.version !== version) {
    throw new Error(
      `latest.json version ${manifest.version} does not match ${version}.`,
    );
  }
  if (!manifest.pub_date || Number.isNaN(Date.parse(manifest.pub_date))) {
    throw new Error("latest.json has no valid pub_date.");
  }

  const targetArtifacts = {
    "darwin-aarch64": names.macos.updater,
    "darwin-x86_64": names.macos.updater,
    "windows-x86_64": names.windows.installer,
    "linux-x86_64": names.linux.installer,
  };
  for (const [target, artifact] of Object.entries(targetArtifacts)) {
    const entry = manifest.platforms?.[target];
    if (!entry) {
      throw new Error(`latest.json is missing target ${target}.`);
    }
    const expectedUrl = releaseDownloadUrl(tag, artifact);
    if (entry.url !== expectedUrl) {
      throw new Error(
        `latest.json ${target} URL does not match ${expectedUrl}.`,
      );
    }
    const signature = readFileSync(
      resolve(releaseDirectory, `${artifact}.sig`),
      "utf8",
    ).trim();
    if (entry.signature !== signature) {
      throw new Error(
        `latest.json ${target} signature does not match ${artifact}.sig.`,
      );
    }
  }

  const checksumPath = resolve(releaseDirectory, "SHA256SUMS.txt");
  if (!existsSync(checksumPath)) {
    throw new Error("Missing SHA256SUMS.txt.");
  }
  const checksumEntries = new Map(
    readFileSync(checksumPath, "utf8")
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .map((line) => {
        const match = line.match(/^([a-f0-9]{64})  (.+)$/);
        if (!match) {
          throw new Error(`Invalid SHA256SUMS.txt entry: ${line}.`);
        }
        return [match[2].replace(/^\.\//, ""), match[1]];
      }),
  );
  const filesToHash = files.filter((file) => file !== "SHA256SUMS.txt");
  if (checksumEntries.size !== filesToHash.length) {
    throw new Error(
      "SHA256SUMS.txt does not contain exactly one entry per release asset.",
    );
  }
  for (const file of filesToHash) {
    const expected = checksumEntries.get(file);
    const actual = sha256(resolve(releaseDirectory, file));
    if (expected !== actual) {
      throw new Error(`SHA-256 verification failed for ${basename(file)}.`);
    }
  }

  console.log(
    `Verified complete Tauri release bundle for ${tag} (${files.length} assets).`,
  );
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
