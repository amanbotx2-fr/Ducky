const assert = require("node:assert/strict");
const { mkdtemp, readFile, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const { test } = require("node:test");

async function releaseContract() {
  return import(
    new URL("../scripts/release/release-contract.mjs", `file://${__filename}`)
  );
}

test("release versions stay consistent across Node, Cargo, and Tauri", async () => {
  const { validateReleaseVersions } = await releaseContract();
  assert.equal(validateReleaseVersions({ tag: "v1.1.0" }), "1.1.0");
});

test("release version validation rejects a mismatched Tauri version", async () => {
  const { validateReleaseVersions } = await releaseContract();
  const directory = await mkdtemp(join(tmpdir(), "ducky-release-version-"));
  const packagePath = join(directory, "package.json");
  const lockPath = join(directory, "package-lock.json");
  const cargoPath = join(directory, "Cargo.toml");
  const tauriPath = join(directory, "tauri.conf.json");

  await Promise.all([
    writeFile(packagePath, '{"version":"1.1.0"}'),
    writeFile(lockPath, '{"packages":{"":{"version":"1.1.0"}}}'),
    writeFile(cargoPath, '[package]\nname = "ducky"\nversion = "1.1.0"\n'),
    writeFile(tauriPath, '{"version":"1.2.0"}'),
  ]);

  assert.throws(
    () =>
      validateReleaseVersions({
        tag: "v1.1.0",
        packagePath,
        lockPath,
        cargoPath,
        tauriPath,
      }),
    /Release version mismatch/,
  );
});

test("Tauri updater asset names are deterministic and collision-free", async () => {
  const { tauriArtifactNames } = await releaseContract();
  const names = tauriArtifactNames("2.0.0");
  const assets = [
    names.macos.installer,
    names.macos.updater,
    names.windows.installer,
    names.windows.msi,
    names.linux.installer,
    names.linux.deb,
  ];

  assert.equal(new Set(assets).size, assets.length);
  assert.ok(assets.every((asset) => asset.startsWith("Ducky-Tauri-2.0.0-")));
  assert.match(names.macos.installer, /macos-universal\.dmg$/);
  assert.match(names.windows.installer, /windows-x86_64-setup\.exe$/);
  assert.match(names.linux.installer, /linux-x86_64\.AppImage$/);
});

test("updater release URLs are pinned to the exact tag and repository", async () => {
  const { releaseDownloadUrl } = await releaseContract();
  assert.equal(
    releaseDownloadUrl(
      "v2.0.0",
      "Ducky-Tauri-2.0.0-linux-x86_64.AppImage",
    ),
    "https://github.com/amanbotx2-fr/Ducky/releases/download/v2.0.0/Ducky-Tauri-2.0.0-linux-x86_64.AppImage",
  );
  assert.throws(
    () => releaseDownloadUrl("latest", "Ducky-Tauri-2.0.0.AppImage"),
    /Unsafe release tag/,
  );
  assert.throws(
    () => releaseDownloadUrl("v2.0.0", "../Ducky-Tauri-2.0.0.AppImage"),
    /Unsafe release asset name/,
  );
});

test("release-only configuration rejects missing and placeholder updater keys", async () => {
  const { validateUpdaterPublicKey } = await releaseContract();
  assert.throws(() => validateUpdaterPublicKey(), /not configured/);
  assert.throws(
    () => validateUpdaterPublicKey("replace-me-with-a-public-key"),
    /placeholder/,
  );
});

test("website cutover selects namespaced Tauri installers", async () => {
  const source = await readFile(
    resolve("website/lib/downloads/githubRelease.ts"),
    "utf8",
  );
  assert.match(source, /const tauriAssetPrefix = "ducky-tauri-"/);
  assert.match(source, /tauriCandidates\[0\]/);
  assert.match(source, /releaseMajor < 2/);
  assert.match(source, /no Tauri/);
});
