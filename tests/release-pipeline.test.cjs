const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { mkdtemp, readFile, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const { test } = require("node:test");

async function releaseContract() {
  return import(
    new URL("../scripts/release/release-contract.mjs", `file://${__filename}`)
  );
}

async function platformSigning() {
  return import(
    new URL("../scripts/release/platform-signing.mjs", `file://${__filename}`)
  );
}

test("release versions stay consistent across Node, Cargo, and Tauri", async () => {
  const { validateReleaseVersions } = await releaseContract();
  assert.equal(validateReleaseVersions({ tag: "v2.0.0" }), "2.0.0");
});

test("release version validation rejects a mismatched Tauri version", async () => {
  const { validateReleaseVersions } = await releaseContract();
  const directory = await mkdtemp(join(tmpdir(), "ducky-release-version-"));
  const packagePath = join(directory, "package.json");
  const lockPath = join(directory, "package-lock.json");
  const cargoPath = join(directory, "Cargo.toml");
  const tauriPath = join(directory, "tauri.conf.json");

  await Promise.all([
    writeFile(packagePath, '{"version":"2.0.0"}'),
    writeFile(lockPath, '{"packages":{"":{"version":"2.0.0"}}}'),
    writeFile(cargoPath, '[package]\nname = "ducky"\nversion = "2.0.0"\n'),
    writeFile(tauriPath, '{"version":"2.1.0"}'),
  ]);

  assert.throws(
    () =>
      validateReleaseVersions({
        tag: "v2.0.0",
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
  const {
    loadCommittedUpdaterPublicKey,
    validateUpdaterPublicKey,
  } = await releaseContract();
  assert.throws(() => validateUpdaterPublicKey(), /not configured/);
  assert.throws(
    () => validateUpdaterPublicKey("replace-me-with-a-public-key"),
    /placeholder/,
  );
  assert.throws(
    () =>
      loadCommittedUpdaterPublicKey(
        join(tmpdir(), "ducky-missing-updater-public-key"),
      ),
    /Committed updater public key is missing/,
  );
});

test("unsigned releases preserve mandatory updater signing", async () => {
  const {
    formatPlatformSigningStatus,
    prepareReleaseNotes,
    resolvePlatformSigning,
    unsignedPlatformReleaseNote,
    validateMandatoryUpdaterSigning,
  } = await platformSigning();
  const updaterEnvironment = {
    TAURI_SIGNING_PRIVATE_KEY: "private-key",
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "private-key-password",
  };

  assert.ok(
    validateMandatoryUpdaterSigning(
      updaterEnvironment,
      resolve("src-tauri/updater.pubkey"),
    ),
  );
  const apple = resolvePlatformSigning("macos", updaterEnvironment);
  const windows = resolvePlatformSigning("windows", updaterEnvironment);

  assert.equal(apple.status, "skipped");
  assert.equal(windows.status, "skipped");
  assert.equal(
    formatPlatformSigningStatus(apple),
    "Apple signing: skipped (credentials not configured)",
  );
  assert.equal(
    formatPlatformSigningStatus(windows),
    "Windows signing: skipped (credentials not configured)",
  );
  assert.ok(
    prepareReleaseNotes("Generated notes\n", {
      appleSigningStatus: apple.status,
      windowsSigningStatus: windows.status,
    }).includes(unsignedPlatformReleaseNote),
  );
});

test("fully signed releases enable Apple and Windows signing", async () => {
  const {
    prepareReleaseNotes,
    resolvePlatformSigning,
    unsignedPlatformReleaseNote,
  } = await platformSigning();
  const environment = {
    APPLE_CERTIFICATE: "certificate",
    APPLE_CERTIFICATE_PASSWORD: "certificate-password",
    APPLE_API_ISSUER: "issuer",
    APPLE_API_KEY: "key-id",
    APPLE_API_PRIVATE_KEY: "private-key",
    WINDOWS_CERTIFICATE: "certificate",
    WINDOWS_CERTIFICATE_PASSWORD: "certificate-password",
    WINDOWS_TIMESTAMP_URL: "https://timestamp.example.test",
  };

  assert.equal(resolvePlatformSigning("macos", environment).status, "enabled");
  assert.equal(
    resolvePlatformSigning("windows", environment).status,
    "enabled",
  );
  assert.ok(
    !prepareReleaseNotes("Generated notes\n", {
      appleSigningStatus: "enabled",
      windowsSigningStatus: "enabled",
    }).includes(unsignedPlatformReleaseNote),
  );
});

test("mixed releases sign only the fully configured platform", async () => {
  const {
    prepareReleaseNotes,
    resolvePlatformSigning,
    unsignedPlatformReleaseNote,
  } = await platformSigning();
  const environment = {
    APPLE_CERTIFICATE: "certificate",
    APPLE_CERTIFICATE_PASSWORD: "certificate-password",
    APPLE_API_ISSUER: "issuer",
    APPLE_API_KEY: "key-id",
    APPLE_API_PRIVATE_KEY: "private-key",
    WINDOWS_TIMESTAMP_URL: "https://timestamp.example.test",
  };

  assert.equal(resolvePlatformSigning("macos", environment).status, "enabled");
  assert.equal(
    resolvePlatformSigning("windows", environment).status,
    "skipped",
  );
  assert.ok(
    prepareReleaseNotes("Generated notes\n", {
      appleSigningStatus: "enabled",
      windowsSigningStatus: "skipped",
    }).includes(unsignedPlatformReleaseNote),
  );
});

test("release-note signing disclosure is idempotent and removed for fully signed releases", async () => {
  const { prepareReleaseNotes, unsignedPlatformReleaseNote } =
    await platformSigning();
  const unsignedNotes = prepareReleaseNotes("Generated notes\n", {
    appleSigningStatus: "skipped",
    windowsSigningStatus: "enabled",
  });
  const repeatedUnsignedNotes = prepareReleaseNotes(unsignedNotes, {
    appleSigningStatus: "skipped",
    windowsSigningStatus: "enabled",
  });
  const signedNotes = prepareReleaseNotes(repeatedUnsignedNotes, {
    appleSigningStatus: "enabled",
    windowsSigningStatus: "enabled",
  });

  assert.equal(
    repeatedUnsignedNotes.split(unsignedPlatformReleaseNote).length - 1,
    1,
  );
  assert.ok(!signedNotes.includes(unsignedPlatformReleaseNote));
});

test("release configuration omits Windows signing unless it is enabled", async () => {
  const directory = await mkdtemp(join(tmpdir(), "ducky-signing-config-"));
  const unsignedPath = join(directory, "unsigned.json");
  const signedPath = join(directory, "signed.json");
  const script = resolve("scripts/release/create-tauri-config.mjs");
  const baseEnvironment = {
    ...process.env,
    RELEASE_TAG: "v2.0.0",
    WINDOWS_TIMESTAMP_URL: "https://timestamp.example.test",
  };

  execFileSync(process.execPath, [script, unsignedPath, "windows"], {
    env: { ...baseEnvironment, PLATFORM_SIGNING: "skipped" },
    stdio: "pipe",
  });
  execFileSync(process.execPath, [script, signedPath, "windows"], {
    env: {
      ...baseEnvironment,
      PLATFORM_SIGNING: "enabled",
      WINDOWS_CERTIFICATE_THUMBPRINT: "ABCDEF123456",
    },
    stdio: "pipe",
  });

  const unsigned = JSON.parse(await readFile(unsignedPath, "utf8"));
  const signed = JSON.parse(await readFile(signedPath, "utf8"));
  assert.equal(unsigned.bundle.windows, undefined);
  assert.deepEqual(signed.bundle.windows, {
    certificateThumbprint: "ABCDEF123456",
    digestAlgorithm: "sha256",
    timestampUrl: "https://timestamp.example.test/",
  });
  assert.equal(unsigned.bundle.createUpdaterArtifacts, true);
  assert.equal(signed.bundle.createUpdaterArtifacts, true);
});

test("missing updater public key fails mandatory signing validation", async () => {
  const { validateMandatoryUpdaterSigning } = await platformSigning();

  assert.throws(
    () =>
      validateMandatoryUpdaterSigning(
        {
          TAURI_SIGNING_PRIVATE_KEY: "private-key",
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "private-key-password",
        },
        join(tmpdir(), "ducky-missing-mandatory-updater.pubkey"),
      ),
    /Committed updater public key is missing/,
  );
});

test("missing updater private key fails mandatory signing validation", async () => {
  const { validateMandatoryUpdaterSigning } = await platformSigning();

  assert.throws(
    () =>
      validateMandatoryUpdaterSigning(
        {
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "private-key-password",
        },
        resolve("src-tauri/updater.pubkey"),
      ),
    /Missing mandatory updater signing configuration: TAURI_SIGNING_PRIVATE_KEY/,
  );
});

test("updater signature verification is isolated from Tauri desktop dependencies", async () => {
  const [script, manifest, lockfile, helper] = await Promise.all([
    readFile(
      resolve("scripts/release/verify-updater-signatures.mjs"),
      "utf8",
    ),
    readFile(
      resolve(
        "scripts/release/updater-signature-verifier/Cargo.toml",
      ),
      "utf8",
    ),
    readFile(
      resolve(
        "scripts/release/updater-signature-verifier/Cargo.lock",
      ),
      "utf8",
    ),
    readFile(
      resolve(
        "scripts/release/updater-signature-verifier/src/main.rs",
      ),
      "utf8",
    ),
  ]);

  assert.match(
    script,
    /scripts\/release\/updater-signature-verifier\/Cargo\.toml/,
  );
  assert.match(script, /"--locked"/);
  assert.doesNotMatch(script, /src-tauri\/Cargo\.toml|--example/);
  assert.match(manifest, /base64 = "0\.22"/);
  assert.match(manifest, /minisign-verify = "0\.2\.5"/);
  assert.doesNotMatch(manifest, /(?:^|\s)(?:tauri|glib|gtk|webkit)/im);
  assert.doesNotMatch(lockfile, /(?:glib|gtk|tauri|webkit)/i);
  assert.match(helper, /PublicKey::decode/);
  assert.match(helper, /Signature::decode/);
  assert.match(helper, /\.verify\(&artifact, &signature, true\)/);
});

test("website cutover selects namespaced Tauri installers", async () => {
  const source = await readFile(
    resolve("website/lib/downloads/githubRelease.ts"),
    "utf8",
  );
  assert.match(source, /const tauriAssetPrefix = "ducky-tauri-"/);
  assert.match(source, /tauriCandidates\[0\]/);
  assert.match(source, /no Tauri/);
  assert.doesNotMatch(source, /releaseMajor|candidates\[0\]/);
});

test("release workflow publishes only the atomic Tauri bundle", async () => {
  const [workflow, releaseConfig] = await Promise.all([
    readFile(resolve(".github/workflows/release.yml"), "utf8"),
    readFile(resolve("scripts/release/create-tauri-config.mjs"), "utf8"),
  ]);

  assert.match(workflow, /build-tauri:/);
  assert.doesNotMatch(workflow, /build-electron:|electron-builder|latest-mac\.yml|latest-linux\.yml/);
  assert.match(workflow, /platform: macos[\s\S]*platform: windows[\s\S]*platform: linux/);
  assert.match(releaseConfig, /createUpdaterArtifacts: true/);
  assert.match(workflow, /TAURI_SIGNING_PRIVATE_KEY: \$\{\{ secrets\./);
  assert.match(workflow, /Validate mandatory updater signing/);
  assert.match(workflow, /validate-signing-environment\.mjs updater/);
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
  assert.match(workflow, /--json databaseId/);
  assert.match(
    workflow,
    /DRAFT_RELEASE_ID=\$draft_release_id[\s\S]*repos\/\$GITHUB_REPOSITORY\/releases\/\$DRAFT_RELEASE_ID/,
  );
  assert.doesNotMatch(
    workflow,
    /repos\/\$GITHUB_REPOSITORY\/releases\/tags\/\$RELEASE_TAG/,
  );
  assert.match(workflow, /release:verify-signatures/);
  assert.match(workflow, /Verify complete Tauri bundle/);
  assert.match(
    workflow,
    /Apple signing: skipped \(credentials not configured\)/,
  );
  assert.match(workflow, /Apple signing: enabled/);
  assert.match(
    workflow,
    /Windows signing: skipped \(credentials not configured\)/,
  );
  assert.match(workflow, /Windows signing: enabled/);
  assert.match(workflow, /TimeStamperCertificate/);
  assert.match(
    workflow,
    /steps\.signing\.outputs\.platform_signing == 'enabled'/,
  );
  assert.match(
    workflow,
    /platform-signing-status-\$\{\{ matrix\.platform \}\}\.txt/,
  );
  assert.match(workflow, /prepare-release-notes\.mjs/);
  assert.match(releaseConfig, /platformSigning === "enabled"/);
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
  const [source, signingConfiguration] = await Promise.all([
    readFile(
      resolve("scripts/release/validate-signing-environment.mjs"),
      "utf8",
    ),
    readFile(resolve("scripts/release/platform-signing.mjs"), "utf8"),
  ]);
  assert.match(signingConfiguration, /TAURI_SIGNING_PRIVATE_KEY/);
  assert.match(signingConfiguration, /APPLE_API_PRIVATE_KEY/);
  assert.match(signingConfiguration, /WINDOWS_CERTIFICATE/);
  assert.match(signingConfiguration, /missing\.join/);
  assert.doesNotMatch(source, /console\.(?:log|error)\([^)]*process\.env/);
  assert.doesNotMatch(
    signingConfiguration,
    /console\.(?:log|error)\([^)]*environment/,
  );
});
