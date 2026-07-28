import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  statSync,
} from "node:fs";
import { basename, join, resolve } from "node:path";

import {
  requireExactlyOne,
  supportedReleasePlatforms,
  tauriArtifactNames,
  validateReleaseVersions,
} from "./release-contract.mjs";

const platform = process.argv[2];
const sourceDirectory = resolve(process.argv[3] ?? "src-tauri/target/release/bundle");
const outputDirectory = resolve(process.argv[4] ?? "release-tauri");

if (!supportedReleasePlatforms.has(platform)) {
  console.error(
    "Usage: node scripts/release/stage-tauri-artifacts.mjs <macos|windows|linux> <bundle-directory> [output-directory]",
  );
  process.exit(2);
}

function collectFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectFiles(path));
    } else if (entry.isFile()) {
      files.push(path);
    }
  }
  return files;
}

function copy(source, targetName) {
  if (!existsSync(source) || statSync(source).size === 0) {
    throw new Error(`Tauri artifact is missing or empty: ${source}.`);
  }
  const target = join(outputDirectory, targetName);
  if (existsSync(target)) {
    throw new Error(`Refusing to overwrite staged release asset ${targetName}.`);
  }
  copyFileSync(source, target);
}

function signatureFor(files, artifact) {
  const signature = `${artifact}.sig`;
  if (!files.includes(signature)) {
    throw new Error(`Missing Tauri updater signature for ${basename(artifact)}.`);
  }
  return signature;
}

try {
  if (!existsSync(sourceDirectory) || !statSync(sourceDirectory).isDirectory()) {
    throw new Error(`Tauri bundle directory does not exist: ${sourceDirectory}.`);
  }

  const version = validateReleaseVersions({
    tag: process.env.RELEASE_TAG || undefined,
  });
  const names = tauriArtifactNames(version)[platform];
  const files = collectFiles(sourceDirectory);
  mkdirSync(outputDirectory, { recursive: true });

  if (platform === "macos") {
    const dmg = requireExactlyOne(
      files,
      (file) => file.endsWith(".dmg"),
      "macOS DMG",
    );
    const updater = requireExactlyOne(
      files,
      (file) => file.endsWith(".app.tar.gz"),
      "macOS updater archive",
    );
    copy(dmg, names.installer);
    copy(updater, names.updater);
    copy(signatureFor(files, updater), `${names.updater}.sig`);
  } else if (platform === "windows") {
    const installer = requireExactlyOne(
      files,
      (file) => file.toLowerCase().endsWith("-setup.exe"),
      "Windows NSIS installer",
    );
    const msi = requireExactlyOne(
      files,
      (file) => file.toLowerCase().endsWith(".msi"),
      "Windows MSI installer",
    );
    copy(installer, names.installer);
    copy(signatureFor(files, installer), `${names.installer}.sig`);
    copy(msi, names.msi);
    const msiSignature = `${msi}.sig`;
    if (files.includes(msiSignature)) {
      copy(msiSignature, `${names.msi}.sig`);
    }
  } else {
    const appImage = requireExactlyOne(
      files,
      (file) => file.endsWith(".AppImage"),
      "Linux AppImage",
    );
    const deb = requireExactlyOne(
      files,
      (file) => file.endsWith(".deb"),
      "Linux DEB package",
    );
    copy(appImage, names.installer);
    copy(signatureFor(files, appImage), `${names.installer}.sig`);
    copy(deb, names.deb);
  }

  console.log(`Staged deterministic ${platform} Tauri release assets:`);
  for (const file of readdirSync(outputDirectory).sort()) {
    console.log(`- ${file}`);
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
