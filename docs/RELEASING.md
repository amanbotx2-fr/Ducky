# Releasing Ducky

Ducky uses one tag-triggered GitHub Actions workflow to build, verify, and
publish Tauri packages for macOS, Windows, and Linux. Native updater artifacts
are always signed. Apple and Windows platform signing is enabled automatically
when the complete platform credential set is configured and skipped otherwise.

A release contains:

- deterministic platform installers;
- signed native updater artifacts;
- `latest.json`;
- detached `.sig` files; and
- `SHA256SUMS.txt`.

Do not create a GitHub Release or upload assets manually. The workflow uses an
atomic draft → verify → publish sequence and refuses to replace assets on an
already published release.

## GitHub configuration

Configure these values under **Repository Settings → Secrets and variables →
Actions**. Never commit or log their values.

### Mandatory updater signing

| Kind | Name |
| --- | --- |
| Secret | `TAURI_SIGNING_PRIVATE_KEY` |
| Secret | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |

Commit only the matching public key at `src-tauri/updater.pubkey`. Installed
applications trust this key, so rotating it requires a separately reviewed
trust migration.

The committed public key and both updater-signing secrets are mandatory.
Validation fails before platform builds begin if any of them are missing.
This requirement is identical for signed and unsigned platform releases.

### Optional Apple signing and notarization

| Kind | Name |
| --- | --- |
| Secret | `APPLE_CERTIFICATE` |
| Secret | `APPLE_CERTIFICATE_PASSWORD` |
| Secret | `APPLE_API_ISSUER` |
| Secret | `APPLE_API_KEY` |
| Secret | `APPLE_API_PRIVATE_KEY` |

When all five values exist, the macOS runner logs `Apple signing: enabled`,
imports the Developer ID certificate into an ephemeral keychain, signs and
notarizes the universal application, and verifies Gatekeeper assessment and
stapled tickets before staging artifacts.

If any value is absent, the runner logs
`Apple signing: skipped (credentials not configured)`, skips certificate
import, signing, notarization, and notarization verification, and continues
with an unsigned `.app` and DMG. Updater archives and their `.sig` files remain
mandatory and signed.

### Optional Windows signing

| Kind | Name |
| --- | --- |
| Secret | `WINDOWS_CERTIFICATE` |
| Secret | `WINDOWS_CERTIFICATE_PASSWORD` |
| Repository variable | `WINDOWS_TIMESTAMP_URL` |

When all three values exist, the Windows runner logs
`Windows signing: enabled`, imports the certificate into its ephemeral user
store, signs and timestamps the NSIS and MSI packages, and requires valid
Authenticode signatures before staging them.

If any value is absent, the runner logs
`Windows signing: skipped (credentials not configured)`, skips certificate
import and signature verification, and continues with unsigned NSIS and MSI
packages. The updater signature for the NSIS package remains mandatory.

Partially configured platform credentials are treated as not configured and
platform signing is skipped. If a complete Windows configuration uses a
non-HTTPS timestamp URL, validation fails.

The built-in `GITHUB_TOKEN` is the only GitHub token. Only the publish job has
`contents: write`.

## Release preparation

1. Start from an up-to-date `main`.
2. Set the same `X.Y.Z` version in:

   - `package.json`;
   - the root package in `package-lock.json`;
   - `src-tauri/Cargo.toml`; and
   - `src-tauri/tauri.conf.json`.

3. Run:

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
5. Tag that exact commit:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

The tag must be exactly `vMAJOR.MINOR.PATCH`, match every version source, and
point to a commit contained in `main`.

## Pipeline architecture

1. **Validate** checks tag/version consistency, membership in `main`, the
   committed updater public key, mandatory updater-signing secrets, locked
   dependency installation, and tests.
2. **Build** runs a three-platform matrix:

   - macOS universal DMG and updater archive;
   - Windows x64 NSIS and MSI packages; and
   - Linux x64 AppImage and DEB packages.

3. **Platform signing** is resolved independently for Apple and Windows.
   Complete credentials enable signing and verification; absent or partial
   credentials select the unsigned build path. Linux is unchanged.
4. **Stage** gives every artifact a deterministic `Ducky-Tauri-X.Y.Z-...`
   name and retains updater signatures.
5. **Aggregate** generates `latest.json` and checksums, verifies updater
   signatures, and validates the complete local inventory.
6. **Draft verification** uploads the complete bundle, verifies the exact
   hosted inventory, redownloads updater artifacts, and verifies their
   signatures again.
7. **Publish** occurs only after every preceding job succeeds.

The matrix uses `fail-fast: false` for complete diagnostics, while the publish
job depends on every platform. A failed run may reset an unpublished draft.
Published assets are immutable.

## Asset contract

| Platform | Installer assets | Updater asset |
| --- | --- | --- |
| macOS universal | `Ducky-Tauri-X.Y.Z-macos-universal.dmg` | `Ducky-Tauri-X.Y.Z-darwin-universal.app.tar.gz` + `.sig` |
| Windows x64 | `Ducky-Tauri-X.Y.Z-windows-x86_64-setup.exe`, `.msi` | NSIS installer + `.sig` |
| Linux x64 | `Ducky-Tauri-X.Y.Z-linux-x86_64.AppImage`, `.deb` | AppImage + `.sig` |

`latest.json` maps both supported macOS architectures to the universal archive
and maps Windows/Linux x64 to their signed native updater artifacts. Every URL
is pinned to the exact release tag.

Platform code signing and updater signing are separate trust mechanisms.
Unsigned Apple or Windows installers still contain updater artifacts protected
by the mandatory Tauri updater signature. When either platform is unsigned,
the generated GitHub Release notes include:

> macOS and/or Windows artifacts are unsigned because platform signing credentials are not yet configured.

The production update endpoint is:

```text
https://github.com/amanbotx2-fr/Ducky/releases/latest/download/latest.json
```

The website download routes select only namespaced Tauri installers from the
latest GitHub release.

## Failure and rollback

If draft verification fails, leave the release unpublished and fix the source
before rerunning the tag workflow. If a published release is defective, do
not replace its assets; publish a newer patch version.

Missing or invalid updater signing configuration always fails the release.
Missing Apple or Windows credentials do not fail it; they select the documented
unsigned path. A complete but invalid platform signing configuration fails its
platform job rather than silently falling back to unsigned output.

Production credentials, hardware installation tests, staged updater tests, and
go-live approval remain release-operator responsibilities outside repository
migration work.
