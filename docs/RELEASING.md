# Releasing Ducky

Ducky uses one tag-triggered GitHub Actions workflow for the temporary
Electron-to-Tauri transition. A release contains:

- signed Tauri installers and updater artifacts for macOS, Windows, and Linux;
- `latest.json` for Tauri v2 updates;
- the legacy Electron installers and `latest-mac.yml`, `latest.yml`, and
  `latest-linux.yml` required by existing installations;
- `SHA256SUMS.txt` covering every published asset.

Do not create a GitHub Release or upload assets manually. The workflow preserves
an atomic draft → verify → publish sequence and refuses to modify a release that
has already been published.

## Production trust setup

Production credentials are external operational inputs. Never generate them in
CI, commit them, paste them into logs, or reuse test material.

### Tauri updater identity

The release manager must supply one externally generated, stable Tauri updater
key pair:

1. Commit the base64-encoded public-key document at
   `src-tauri/updater.pubkey`.
2. Add the matching private key as the GitHub Actions secret
   `TAURI_SIGNING_PRIVATE_KEY`.
3. Add its password as the GitHub Actions secret
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

The committed public key is embedded only in release builds. Normal local
builds keep the credential-free Phase 10 configuration. The workflow rejects a
missing or placeholder public key, and the native updater rejects any artifact
whose signature does not match it.

Changing this key after a Tauri release is a trust migration: already installed
clients will continue trusting the old key. Do not rotate it as an ordinary
repository variable.

### Apple signing and notarization

Add these GitHub Actions secrets:

| Name | Content |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64 encoding of the Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for that `.p12` |
| `APPLE_API_ISSUER` | App Store Connect API issuer ID |
| `APPLE_API_KEY` | App Store Connect API key ID |
| `APPLE_API_PRIVATE_KEY` | Complete private `.p8` key text |

The macOS runner imports the certificate into an ephemeral keychain, signs the
universal application, submits it for notarization through the App Store
Connect API, and requires valid signatures, Gatekeeper assessment, and stapled
notarization tickets before staging artifacts.

### Windows signing

Add these GitHub Actions values:

| Kind | Name | Content |
| --- | --- | --- |
| Secret | `WINDOWS_CERTIFICATE` | Base64 encoding of the Authenticode `.pfx` |
| Secret | `WINDOWS_CERTIFICATE_PASSWORD` | Password for that `.pfx` |
| Repository variable | `WINDOWS_TIMESTAMP_URL` | Certificate issuer's HTTPS RFC 3161 timestamp URL |

The Windows runner imports the certificate into the ephemeral user certificate
store, derives its thumbprint without logging it, configures SHA-256
timestamped signing, and requires valid Authenticode signatures on both NSIS
and MSI output.

Configure secrets and variables under **Repository Settings → Secrets and
variables → Actions**. The built-in `GITHUB_TOKEN` is the only GitHub token;
only the publish job receives `contents: write`.

## Release checklist

1. Start from an up-to-date `main`.
2. Update the same stable `X.Y.Z` version in:

   - `package.json`;
   - `package-lock.json` root package;
   - `src-tauri/Cargo.toml`;
   - `src-tauri/tauri.conf.json`.

   `npm version X.Y.Z --no-git-tag-version` updates the Node package and
   lockfile; update both Tauri version fields in the same release change.

3. Run the local credential-independent checks:

   ```bash
   npm ci
   npm run release:validate-versions
   npm run typecheck
   npm test
   npm run build
   cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
   cargo test --manifest-path src-tauri/Cargo.toml
   cargo build --manifest-path src-tauri/Cargo.toml
   ```

4. Commit and push the release change to `main`.
5. Tag that exact commit and push the tag:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

The tag must use exactly `vMAJOR.MINOR.PATCH`, match all four version sources,
and point to a commit contained in `main`.

## Pipeline architecture

The release workflow has four gates:

1. **Validate** checks the tag, all Node/Rust/Tauri versions, membership in
   `main`, locked dependency installation, and tests.
2. **Build** runs two native three-platform matrices:
   - Electron builds the legacy universal macOS, Windows x64, and Linux x64
     feeds without changing their existing names or metadata;
   - Tauri builds signed/notarized universal macOS, signed Windows x64, and
     Linux x64 packages with updater signatures.
3. **Aggregate and stage** downloads all six build outputs, verifies the
   Electron metadata, generates `latest.json` and SHA-256 checksums,
   cryptographically verifies each Tauri updater artifact, and uploads the
   complete set to a draft GitHub Release.
4. **Verify and publish** checks the exact draft name/state/size inventory,
   downloads the updater files back through the authenticated GitHub API,
   verifies their signatures again, and only then publishes the draft.

`fail-fast` is disabled within each build matrix for complete diagnostics, but
the publish job cannot run unless every native build succeeds. Re-running a
failed tag can reset its draft assets. A published release is immutable to the
workflow.

## Tauri release assets

Tauri assets use a runtime-qualified namespace, so they cannot collide with or
be mistaken for the legacy Electron feed:

| Platform | Installer assets | Updater asset |
| --- | --- | --- |
| macOS universal | `Ducky-Tauri-X.Y.Z-macos-universal.dmg` | `Ducky-Tauri-X.Y.Z-darwin-universal.app.tar.gz` + `.sig` |
| Windows x64 | `Ducky-Tauri-X.Y.Z-windows-x86_64-setup.exe`, `.msi` | NSIS installer + `.sig` |
| Linux x64 | `Ducky-Tauri-X.Y.Z-linux-x86_64.AppImage`, `.deb` | AppImage + `.sig` |

`latest.json` maps both `darwin-aarch64` and `darwin-x86_64` to the signed
universal archive, and maps `windows-x86_64` and `linux-x86_64` to their signed
native updater artifacts. Every URL is pinned to the exact release tag.

The embedded production endpoint is:

```text
https://github.com/amanbotx2-fr/Ducky/releases/latest/download/latest.json
```

## Electron transition and website cutover

The final Electron 2.x feed metadata remains in the same GitHub Release.
Existing Electron clients discover that version through their existing
Electron Builder channel and show the approved one-time manual migration
dialog. No framework replacement or installer chaining is attempted.

The website prefers `Ducky-Tauri-...` installers whenever they are present.
It retains a compatibility fallback for current Electron-only v1 releases, but
v2 and later downloads fail closed if a namespaced Tauri installer is absent.
This prevents a public v2 download button from silently serving the transition
Electron package.

## Production verification and rollback

Before the first production publish, verify on real hardware:

- clean Tauri install on macOS Apple Silicon and Intel, Windows x64, and a
  supported Linux x64 distribution;
- macOS Gatekeeper/notarization and Windows Authenticode publisher identity;
- update detection from an older signed Tauri build;
- signed download, installation, application restart, and preserved settings;
- the final Electron migration dialog and Download/Remind Me Later paths;
- website downloads select the matching Tauri installer;
- legacy Electron metadata continues resolving all referenced files.

The workflow performs the artifact and hosted-draft checks automatically, but
interactive installation/restart verification is a release-manager gate.

If a draft fails verification, leave it unpublished and rerun the corrected tag
workflow. If a published release is defective, do not replace its assets:
publish a newer patch version. Preserve old signatures, metadata, and
installers for clients already consuming them.

## Current credential gate

The pipeline is implemented to fail closed, but it cannot produce a real signed
release until the stable updater key pair, Apple credentials, and Windows
certificate described above are supplied. A test key or unsigned package must
never be used to bypass this gate.
