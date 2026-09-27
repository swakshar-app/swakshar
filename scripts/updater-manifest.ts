/**
 * Prints a draft release's updater manifest with release download links:
 * `node scripts/updater-manifest.ts latest.json assets.json owner/repo v0.1.2`,
 * where `assets.json` is `gh release view <tag> --json assets`.
 */
import { readFile } from "node:fs/promises";

import { downloadLinks } from "./updater-links.ts";

/** Reads both files, rewrites the links and prints the manifest. */
async function main(): Promise<void> {
  const [manifestPath, assetsPath, repository, tag] = process.argv.slice(2);
  if (manifestPath === undefined || assetsPath === undefined || repository === undefined || tag === undefined) {
    process.stderr.write("usage: node scripts/updater-manifest.ts <latest.json> <assets.json> <owner/repo> <tag>\n");
    process.exitCode = 2;
    return;
  }
  const manifest: unknown = JSON.parse(await readFile(manifestPath, "utf8"));
  const assets: unknown = JSON.parse(await readFile(assetsPath, "utf8"));
  process.stdout.write(`${JSON.stringify(downloadLinks(manifest, assets, repository, tag), null, 2)}\n`);
}

await main();
