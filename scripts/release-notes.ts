/**
 * Prints a version's release notes from CHANGELOG.md, one line per bullet,
 * and fails when there are none, so a release cannot ship without notes:
 * `node scripts/release-notes.ts 0.1.0`.
 */
import { readFile } from "node:fs/promises";
import { join } from "node:path";

import { sectionFor, unwrap } from "./changelog.ts";

/** The changelog at the repository root. */
const CHANGELOG = join(import.meta.dirname, "..", "CHANGELOG.md");

/** Prints the notes, or explains what is missing and exits non-zero. */
async function main(): Promise<void> {
  const version = process.argv[2] ?? "";
  const notes = sectionFor(await readFile(CHANGELOG, "utf8"), version);
  if (version === "" || notes === null) {
    process.stderr.write(`CHANGELOG.md has no notes for "${version}". Add a "## ${version.replace(/^v/u, "")}" section.\n`);
    process.exitCode = 1;
    return;
  }
  process.stdout.write(`${unwrap(notes)}\n`);
}

await main();
