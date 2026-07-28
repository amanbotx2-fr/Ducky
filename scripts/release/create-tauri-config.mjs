import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import {
  updaterEndpoint,
  validateReleaseVersions,
  validateUpdaterPublicKey,
} from "./release-contract.mjs";

const outputPath = resolve(
  process.argv[2] ?? "release-config/tauri.release.conf.json",
);
const platform = process.argv[3];

if (!new Set(["macos", "windows", "linux"]).has(platform)) {
  console.error(
    "Usage: node scripts/release/create-tauri-config.mjs <output> <macos|windows|linux>",
  );
  process.exit(2);
}

try {
  validateReleaseVersions({ tag: process.env.RELEASE_TAG || undefined });
  const publicKey = validateUpdaterPublicKey(
    process.env.TAURI_UPDATER_PUBLIC_KEY,
  );

  const config = {
    bundle: {
      createUpdaterArtifacts: true,
    },
    plugins: {
      updater: {
        pubkey: publicKey,
        endpoints: [updaterEndpoint],
      },
    },
  };

  if (platform === "windows") {
    const certificateThumbprint =
      process.env.WINDOWS_CERTIFICATE_THUMBPRINT?.trim();
    const timestampUrl = process.env.WINDOWS_TIMESTAMP_URL?.trim();

    if ((certificateThumbprint && !timestampUrl) || (!certificateThumbprint && timestampUrl)) {
      throw new Error(
        "Windows signing requires both WINDOWS_CERTIFICATE_THUMBPRINT and WINDOWS_TIMESTAMP_URL.",
      );
    }
    if (certificateThumbprint && timestampUrl) {
      const timestamp = new URL(timestampUrl);
      if (timestamp.protocol !== "https:") {
        throw new Error("WINDOWS_TIMESTAMP_URL must use HTTPS.");
      }
      config.bundle.windows = {
        certificateThumbprint,
        digestAlgorithm: "sha256",
        timestampUrl: timestamp.href,
      };
    }
  }

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(config, null, 2)}\n`, {
    mode: 0o600,
  });
  console.log(`Created release-only Tauri configuration for ${platform}.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
