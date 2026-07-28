import { readFileSync } from "node:fs";
import { basename, resolve } from "node:path";

export const releaseRepository = "amanbotx2-fr/Ducky";
export const releaseAssetPrefix = "Ducky-Tauri-";
export const updaterEndpoint =
  "https://github.com/amanbotx2-fr/Ducky/releases/latest/download/latest.json";

export const supportedReleasePlatforms = new Set([
  "macos",
  "windows",
  "linux",
]);

export function readJson(path) {
  return JSON.parse(readFileSync(resolve(path), "utf8"));
}

export function readCargoPackageVersion(path = "src-tauri/Cargo.toml") {
  const cargoToml = readFileSync(resolve(path), "utf8");
  const packageSection = cargoToml.match(
    /^\[package\]\s*$([\s\S]*?)(?=^\[|(?![\s\S]))/m,
  )?.[1];
  const version = packageSection?.match(
    /^version\s*=\s*"([^"]+)"\s*$/m,
  )?.[1];

  if (!version) {
    throw new Error(`Unable to read [package].version from ${path}.`);
  }

  return version;
}

export function validateStableVersion(value, label) {
  if (!/^[0-9]+\.[0-9]+\.[0-9]+$/.test(value)) {
    throw new Error(`${label} must be a stable MAJOR.MINOR.PATCH version.`);
  }
  return value;
}

export function validateReleaseVersions({
  tag,
  packagePath = "package.json",
  lockPath = "package-lock.json",
  cargoPath = "src-tauri/Cargo.toml",
  tauriPath = "src-tauri/tauri.conf.json",
} = {}) {
  const packageVersion = validateStableVersion(
    readJson(packagePath).version,
    "package.json version",
  );
  const lockVersion = validateStableVersion(
    readJson(lockPath).packages?.[""]?.version,
    "package-lock.json root version",
  );
  const cargoVersion = validateStableVersion(
    readCargoPackageVersion(cargoPath),
    "Cargo package version",
  );
  const tauriVersion = validateStableVersion(
    readJson(tauriPath).version,
    "Tauri configuration version",
  );

  const versions = {
    "package.json": packageVersion,
    "package-lock.json": lockVersion,
    "src-tauri/Cargo.toml": cargoVersion,
    "src-tauri/tauri.conf.json": tauriVersion,
  };
  const mismatches = Object.entries(versions).filter(
    ([, version]) => version !== packageVersion,
  );

  if (mismatches.length > 0) {
    throw new Error(
      `Release version mismatch: ${Object.entries(versions)
        .map(([file, version]) => `${file}=${version}`)
        .join(", ")}.`,
    );
  }

  if (tag !== undefined) {
    if (!/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(tag)) {
      throw new Error(`Release tag ${tag} is not in vMAJOR.MINOR.PATCH form.`);
    }
    if (tag !== `v${packageVersion}`) {
      throw new Error(
        `Release tag ${tag} does not match version ${packageVersion}.`,
      );
    }
  }

  return packageVersion;
}

export function validateUpdaterPublicKey(value) {
  const publicKey = value?.trim();
  if (!publicKey) {
    throw new Error("TAURI_UPDATER_PUBLIC_KEY is not configured.");
  }
  if (/placeholder|replace[-_ ]?me|your[_ -]?/i.test(publicKey)) {
    throw new Error("TAURI_UPDATER_PUBLIC_KEY contains placeholder material.");
  }

  const keyLine = publicKey
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => /^[A-Za-z0-9+/]{40,}={0,2}$/.test(line));
  if (!keyLine) {
    throw new Error(
      "TAURI_UPDATER_PUBLIC_KEY is not a valid Minisign public-key document.",
    );
  }

  return publicKey;
}

export function tauriArtifactNames(version) {
  return {
    macos: {
      installer: `${releaseAssetPrefix}${version}-macos-universal.dmg`,
      updater: `${releaseAssetPrefix}${version}-darwin-universal.app.tar.gz`,
    },
    windows: {
      installer: `${releaseAssetPrefix}${version}-windows-x86_64-setup.exe`,
      msi: `${releaseAssetPrefix}${version}-windows-x86_64.msi`,
    },
    linux: {
      installer: `${releaseAssetPrefix}${version}-linux-x86_64.AppImage`,
      deb: `${releaseAssetPrefix}${version}-linux-x86_64.deb`,
    },
  };
}

export function releaseDownloadUrl(tag, assetName) {
  if (!/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(tag)) {
    throw new Error(`Unsafe release tag: ${tag}.`);
  }
  if (basename(assetName) !== assetName || !assetName.startsWith("Ducky-")) {
    throw new Error(`Unsafe release asset name: ${assetName}.`);
  }

  return `https://github.com/${releaseRepository}/releases/download/${tag}/${encodeURIComponent(assetName)}`;
}

export function requireExactlyOne(files, predicate, label) {
  const matches = files.filter(predicate);
  if (matches.length !== 1) {
    throw new Error(
      `Expected exactly one ${label}; found ${matches.length}: ${matches.join(", ") || "(none)"}.`,
    );
  }
  return matches[0];
}

export function requireSignature(files, artifact) {
  const signature = `${artifact}.sig`;
  if (!files.includes(signature)) {
    throw new Error(`Missing updater signature for ${artifact}.`);
  }
  return signature;
}
