import { loadCommittedUpdaterPublicKey } from "./release-contract.mjs";

export const updaterSigningInputs = Object.freeze([
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
]);

export const platformSigningInputs = Object.freeze({
  macos: Object.freeze([
    "APPLE_CERTIFICATE",
    "APPLE_CERTIFICATE_PASSWORD",
    "APPLE_API_ISSUER",
    "APPLE_API_KEY",
    "APPLE_API_PRIVATE_KEY",
  ]),
  windows: Object.freeze([
    "WINDOWS_CERTIFICATE",
    "WINDOWS_CERTIFICATE_PASSWORD",
    "WINDOWS_TIMESTAMP_URL",
  ]),
  linux: Object.freeze([]),
});

const platformLabels = Object.freeze({
  macos: "Apple",
  windows: "Windows",
  linux: "Linux platform",
});

const releaseNoteStart = "<!-- ducky-platform-signing-note:start -->";
const releaseNoteEnd = "<!-- ducky-platform-signing-note:end -->";

export const unsignedPlatformReleaseNote =
  "macOS and/or Windows artifacts are unsigned because platform signing credentials are not yet configured.";

function missingInputs(names, environment) {
  return names.filter((name) => !environment[name]?.trim());
}

export function validateMandatoryUpdaterSigning(
  environment = process.env,
  publicKeyPath = environment.TAURI_UPDATER_PUBLIC_KEY_PATH,
) {
  const publicKey = loadCommittedUpdaterPublicKey(publicKeyPath);
  const missing = missingInputs(updaterSigningInputs, environment);

  if (missing.length > 0) {
    throw new Error(
      `Missing mandatory updater signing configuration: ${missing.join(", ")}.`,
    );
  }

  return publicKey;
}

export function resolvePlatformSigning(platform, environment = process.env) {
  const required = platformSigningInputs[platform];
  if (required === undefined) {
    throw new Error(`Unsupported release platform: ${platform}.`);
  }

  if (platform === "linux") {
    return Object.freeze({
      platform,
      label: platformLabels[platform],
      status: "not_applicable",
      missing: Object.freeze([]),
    });
  }

  const missing = missingInputs(required, environment);
  if (missing.length > 0) {
    return Object.freeze({
      platform,
      label: platformLabels[platform],
      status: "skipped",
      missing: Object.freeze(missing),
    });
  }

  if (
    platform === "windows" &&
    new URL(environment.WINDOWS_TIMESTAMP_URL).protocol !== "https:"
  ) {
    throw new Error("WINDOWS_TIMESTAMP_URL must use HTTPS.");
  }

  return Object.freeze({
    platform,
    label: platformLabels[platform],
    status: "enabled",
    missing: Object.freeze([]),
  });
}

export function formatPlatformSigningStatus(state) {
  if (state.status === "enabled") {
    return `${state.label} signing: enabled`;
  }
  if (state.status === "skipped") {
    return `${state.label} signing: skipped (credentials not configured)`;
  }
  return `${state.label} signing: not applicable`;
}

export function prepareReleaseNotes(
  body,
  { appleSigningStatus, windowsSigningStatus },
) {
  const supportedStatuses = new Set(["enabled", "skipped"]);
  if (
    !supportedStatuses.has(appleSigningStatus) ||
    !supportedStatuses.has(windowsSigningStatus)
  ) {
    throw new Error(
      "Release notes require enabled or skipped Apple and Windows signing statuses.",
    );
  }

  const markerPattern = new RegExp(
    `\\n*${releaseNoteStart}[\\s\\S]*?${releaseNoteEnd}\\n*`,
    "g",
  );
  const generatedNotes = body.replace(markerPattern, "\n").trimEnd();

  if (
    appleSigningStatus === "enabled" &&
    windowsSigningStatus === "enabled"
  ) {
    return generatedNotes.length > 0 ? `${generatedNotes}\n` : "";
  }

  const prefix = generatedNotes.length > 0 ? `${generatedNotes}\n\n` : "";
  return `${prefix}${releaseNoteStart}\n${unsignedPlatformReleaseNote}\n${releaseNoteEnd}\n`;
}
