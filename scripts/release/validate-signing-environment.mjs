import { loadCommittedUpdaterPublicKey } from "./release-contract.mjs";

const platform = process.argv[2];
const supported = new Set(["macos", "windows", "linux"]);

if (!supported.has(platform)) {
  console.error(
    "Usage: node scripts/release/validate-signing-environment.mjs <macos|windows|linux>",
  );
  process.exit(2);
}

try {
  loadCommittedUpdaterPublicKey(process.env.TAURI_UPDATER_PUBLIC_KEY_PATH);

  const required = [
    "TAURI_SIGNING_PRIVATE_KEY",
    "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
  ];
  if (platform === "macos") {
    required.push(
      "APPLE_CERTIFICATE",
      "APPLE_CERTIFICATE_PASSWORD",
      "APPLE_API_ISSUER",
      "APPLE_API_KEY",
      "APPLE_API_PRIVATE_KEY",
    );
  }
  if (platform === "windows") {
    required.push(
      "WINDOWS_CERTIFICATE",
      "WINDOWS_CERTIFICATE_PASSWORD",
      "WINDOWS_TIMESTAMP_URL",
    );
  }

  const missing = required.filter((name) => !process.env[name]?.trim());
  if (missing.length > 0) {
    throw new Error(
      `Missing required ${platform} release configuration: ${missing.join(", ")}.`,
    );
  }
  if (
    platform === "windows" &&
    new URL(process.env.WINDOWS_TIMESTAMP_URL).protocol !== "https:"
  ) {
    throw new Error("WINDOWS_TIMESTAMP_URL must use HTTPS.");
  }

  console.log(
    `Validated presence of ${platform} signing configuration without reading or logging secret values.`,
  );
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
