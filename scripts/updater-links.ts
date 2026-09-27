/**
 * Rewrites the download links in an updater manifest (`latest.json`).
 * tauri-action fills a draft's manifest with GitHub API asset links, which
 * count against GitHub's hourly limit on anonymous API calls, so people
 * behind one office network could be refused an update. Release download
 * links carry no such limit.
 */

/** True for a plain object. */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Asset names keyed by API link, from `gh release view --json assets`. */
function assetNames(assets: unknown): Map<string, string> {
  const list = isRecord(assets) ? assets["assets"] : undefined;
  if (!Array.isArray(list)) {
    throw new Error("the asset list has no assets array");
  }
  const names = new Map<string, string>();
  for (const asset of list) {
    if (isRecord(asset) && typeof asset["apiUrl"] === "string" && typeof asset["name"] === "string") {
      names.set(asset["apiUrl"], asset["name"]);
    }
  }
  return names;
}

/**
 * The manifest with every platform's link pointing at the release download
 * of the same asset. Throws when a link names an asset the release does not
 * have, so a broken manifest never ships.
 */
export function downloadLinks(manifest: unknown, assets: unknown, repository: string, tag: string): unknown {
  const platforms = isRecord(manifest) ? manifest["platforms"] : undefined;
  if (!isRecord(manifest) || !isRecord(platforms)) {
    throw new Error("the manifest has no platforms");
  }
  const names = assetNames(assets);
  const base = `https://github.com/${repository}/releases/download/${tag}/`;
  const rewritten = Object.fromEntries(
    Object.entries(platforms).map(([key, entry]) => {
      const url = isRecord(entry) ? entry["url"] : undefined;
      if (!isRecord(entry) || typeof url !== "string") {
        throw new Error(`platform ${key} has no link`);
      }
      if (url.startsWith(base)) {
        return [key, entry];
      }
      const name = names.get(url);
      if (name === undefined) {
        throw new Error(`platform ${key} links to ${url}, which is not in the release`);
      }
      return [key, { ...entry, url: `${base}${encodeURIComponent(name)}` }];
    }),
  );
  return { ...manifest, platforms: rewritten };
}
