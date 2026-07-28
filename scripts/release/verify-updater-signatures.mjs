import { existsSync, readFileSync, statSync } from "node:fs";
import { basename, resolve } from "node:path";
import { spawnSync } from "node:child_process";

import { validateUpdaterPublicKey } from "./release-contract.mjs";

const releaseDirectory = resolve(process.argv[2] ?? "release-artifacts");

try {
  const publicKey = validateUpdaterPublicKey(
    process.env.TAURI_UPDATER_PUBLIC_KEY,
  );
  const manifestPath = resolve(releaseDirectory, "latest.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  const checked = new Set();

  for (const [target, release] of Object.entries(manifest.platforms ?? {})) {
    const artifactName = basename(new URL(release.url).pathname);
    if (checked.has(artifactName)) {
      continue;
    }

    const artifactPath = resolve(releaseDirectory, artifactName);
    const signaturePath = `${artifactPath}.sig`;
    if (
      !existsSync(artifactPath) ||
      !existsSync(signaturePath) ||
      statSync(artifactPath).size === 0 ||
      statSync(signaturePath).size === 0
    ) {
      throw new Error(`Missing signed updater pair for ${target}.`);
    }

    const verification = spawnSync(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "src-tauri/Cargo.toml",
        "--example",
        "verify_updater_signature",
        "--",
        artifactPath,
        signaturePath,
      ],
      {
        cwd: resolve("."),
        env: {
          ...process.env,
          TAURI_UPDATER_PUBLIC_KEY: publicKey,
        },
        encoding: "utf8",
      },
    );
    if (verification.status !== 0) {
      throw new Error(
        `Signature verification failed for ${artifactName}: ${
          verification.stderr.trim() || "unknown verifier error"
        }`,
      );
    }
    checked.add(artifactName);
  }

  if (checked.size !== 3) {
    throw new Error(
      `Expected three unique platform updater artifacts; verified ${checked.size}.`,
    );
  }
  console.log(`Cryptographically verified ${checked.size} updater artifacts.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
