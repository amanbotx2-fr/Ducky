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

test("release workflow preserves atomic dual-runtime publication", async () => {
  const [workflow, releaseConfig] = await Promise.all([
    readFile(resolve(".github/workflows/release.yml"), "utf8"),
    readFile(resolve("scripts/release/create-tauri-config.mjs"), "utf8"),
  ]);

  assert.match(workflow, /build-electron:/);
  assert.match(workflow, /build-tauri:/);
  assert.match(workflow, /platform: macos[\s\S]*platform: windows[\s\S]*platform: linux/);
  assert.match(releaseConfig, /createUpdaterArtifacts: true/);
  assert.match(workflow, /TAURI_SIGNING_PRIVATE_KEY: \$\{\{ secrets\./);
  assert.match(
    workflow,
    /APPLE_CERTIFICATE: \$\{\{ matrix\.platform == 'macos' && secrets\./,
  );
  assert.match(
    workflow,
    /WINDOWS_CERTIFICATE: \$\{\{ matrix\.platform == 'windows' && secrets\./,
  );
  assert.match(workflow, /Verify signed updater downloads from draft/);
  assert.match(workflow, /verify-github-draft\.mjs/);
  assert.match(workflow, /release:verify-signatures/);
  assert.match(workflow, /latest-mac\.yml/);
  assert.match(workflow, /latest-linux\.yml/);
  assert.ok(
    workflow.indexOf("Verify remote draft inventory") <
      workflow.indexOf("Publish verified GitHub Release"),
  );
  assert.ok(
    workflow.indexOf("Verify signed updater downloads from draft") <
      workflow.indexOf("Publish verified GitHub Release"),
  );
  assert.doesNotMatch(
    workflow,
    /TAURI_SIGNING_PRIVATE_KEY:\s*(?!\$\{\{ secrets\.)\S+/,
  );
});

test("signing preflight checks names without printing credential values", async () => {
  const source = await readFile(
    resolve("scripts/release/validate-signing-environment.mjs"),
    "utf8",
  );
  assert.match(source, /TAURI_SIGNING_PRIVATE_KEY/);
  assert.match(source, /APPLE_API_PRIVATE_KEY/);
  assert.match(source, /WINDOWS_CERTIFICATE/);
  assert.match(source, /missing\.join/);
  assert.doesNotMatch(source, /console\.(?:log|error)\([^)]*process\.env/);
});
