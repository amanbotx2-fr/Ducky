import { existsSync, readFileSync, statSync } from "node:fs";
import { basename, resolve } from "node:path";
import { spawnSync } from "node:child_process";

import { loadCommittedUpdaterPublicKey } from "./release-contract.mjs";

const releaseDirectory = resolve(process.argv[2] ?? "release-artifacts");
const verifierManifestPath = resolve(
  "scripts/release/updater-signature-verifier/Cargo.toml",
);
const verifierTargetDirectory =
  process.env.CARGO_TARGET_DIR ??
  resolve("src-tauri/target/updater-signature-verifier");

try {
  const publicKey = loadCommittedUpdaterPublicKey(
    process.env.TAURI_UPDATER_PUBLIC_KEY_PATH,
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
        "--locked",
        "--manifest-path",
        verifierManifestPath,
        "--",
        artifactPath,
        signaturePath,
      ],
      {
        cwd: resolve("."),
        env: {
          ...process.env,
          CARGO_TARGET_DIR: verifierTargetDirectory,
          TAURI_UPDATER_PUBLIC_KEY: publicKey,
        },
        encoding: "utf8",
      },
    );
    if (verification.error) {
      throw new Error(
        `Unable to run updater signature verifier for ${artifactName}: ${verification.error.message}`,
      );
    }
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
