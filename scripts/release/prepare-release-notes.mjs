import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import { prepareReleaseNotes } from "./platform-signing.mjs";

const inputPath = resolve(process.argv[2] ?? "");
const outputPath = resolve(process.argv[3] ?? "");

if (!process.argv[2] || !process.argv[3]) {
  console.error(
    "Usage: node scripts/release/prepare-release-notes.mjs <input> <output>",
  );
  process.exit(2);
}

try {
  const notes = prepareReleaseNotes(readFileSync(inputPath, "utf8"), {
    appleSigningStatus: process.env.APPLE_SIGNING_STATUS,
    windowsSigningStatus: process.env.WINDOWS_SIGNING_STATUS,
  });
  writeFileSync(outputPath, notes);
  console.log("Prepared release notes for the resolved platform signing state.");
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
