/**
 * Sets the release version everywhere it is declared:
 * `node scripts/bump-version.ts 0.2.0`.
 */
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

/** Repository root. */
const ROOT = join(import.meta.dirname, "..");
/** Plain semantic version, no pre-release suffix. */
const SEMVER = /^\d+\.\d+\.\d+$/u;

/** Replaces the first `version` value matched by `pattern` in `file`. */
async function replaceVersion(file: string, pattern: RegExp, version: string): Promise<void> {
  const path = join(ROOT, file);
  const text = await readFile(path, "utf8");
  if (!pattern.test(text)) {
    throw new Error(`no version found in ${file}`);
  }
  await writeFile(path, text.replace(pattern, (_match: string, prefix: string) => `${prefix}${version}"`));
}

/** Validates the argument and updates every file. */
async function main(): Promise<void> {
  const version = process.argv[2] ?? "";
  if (!SEMVER.test(version)) {
    process.stderr.write("usage: node scripts/bump-version.ts <major.minor.patch>\n");
    process.exitCode = 2;
    return;
  }
  await replaceVersion("Cargo.toml", /(\[workspace\.package\]\nversion = ")[^"]*"/u, version);
  await replaceVersion("app/package.json", /("version": ")[^"]*"/u, version);
  await replaceVersion("app/src-tauri/tauri.conf.json", /("version": ")[^"]*"/u, version);
  process.stdout.write(`version set to ${version}\n`);
}

await main();
