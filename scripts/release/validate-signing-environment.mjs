import { appendFileSync } from "node:fs";

import {
  formatPlatformSigningStatus,
  resolvePlatformSigning,
  validateMandatoryUpdaterSigning,
} from "./platform-signing.mjs";

const target = process.argv[2];
const supported = new Set(["updater", "macos", "windows", "linux"]);

if (!supported.has(target)) {
  console.error(
    "Usage: node scripts/release/validate-signing-environment.mjs <updater|macos|windows|linux>",
  );
  process.exit(2);
}

try {
  validateMandatoryUpdaterSigning();
  console.log("Updater signing: enabled (mandatory)");

  if (target !== "updater") {
    const platformSigning = resolvePlatformSigning(target);
    console.log(formatPlatformSigningStatus(platformSigning));

    if (process.env.GITHUB_OUTPUT) {
      appendFileSync(
        process.env.GITHUB_OUTPUT,
        `platform_signing=${platformSigning.status}\n`,
      );
    }
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
