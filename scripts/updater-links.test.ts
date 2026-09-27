import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { downloadLinks } from "./updater-links.ts";

/** Repository and tag of the release under test. */
const REPOSITORY = "swakshar-app/swakshar";
const TAG = "v0.1.2";
const API = `https://api.github.com/repos/${REPOSITORY}/releases/assets`;
const DOWNLOAD = `https://github.com/${REPOSITORY}/releases/download/${TAG}`;

/** Assets as `gh release view --json assets` lists them. */
const ASSETS = {
  assets: [
    { apiUrl: `${API}/11`, name: "Swakshar_0.1.2_universal.app.tar.gz" },
    { apiUrl: `${API}/22`, name: "Swakshar_0.1.2_x64-setup.exe" },
  ],
};

/** A manifest as tauri-action writes it for a draft. */
function manifest(urls: Record<string, string>): unknown {
  const platforms = Object.fromEntries(Object.entries(urls).map(([key, url]) => [key, { signature: `sig-${key}`, url }]));
  return { version: "0.1.2", notes: "Notes.", pub_date: "2026-09-27T12:00:00.000Z", platforms };
}

describe("downloadLinks", () => {
  it("swaps API asset links for release download links, keeping everything else", () => {
    const result = downloadLinks(manifest({ "darwin-universal": `${API}/11`, "windows-x86_64": `${API}/22` }), ASSETS, REPOSITORY, TAG);
    assert.deepEqual(result, manifest({
      "darwin-universal": `${DOWNLOAD}/Swakshar_0.1.2_universal.app.tar.gz`,
      "windows-x86_64": `${DOWNLOAD}/Swakshar_0.1.2_x64-setup.exe`,
    }));
  });

  it("keeps links that already point at this release's downloads", () => {
    const link = `${DOWNLOAD}/Swakshar_0.1.2_universal.app.tar.gz`;
    const result = downloadLinks(manifest({ "darwin-universal": link }), ASSETS, REPOSITORY, TAG);
    assert.deepEqual(result, manifest({ "darwin-universal": link }));
  });

  it("encodes asset names for use in a link", () => {
    const assets = { assets: [{ apiUrl: `${API}/33`, name: "Swakshar 0.1.2+1.tar.gz" }] };
    const result = downloadLinks(manifest({ "linux-x86_64": `${API}/33` }), assets, REPOSITORY, TAG);
    assert.deepEqual(result, manifest({ "linux-x86_64": `${DOWNLOAD}/Swakshar%200.1.2%2B1.tar.gz` }));
  });

  it("fails on a link to an asset the release does not have", () => {
    assert.throws(() => downloadLinks(manifest({ "darwin-universal": `${API}/99` }), ASSETS, REPOSITORY, TAG), /darwin-universal/u);
  });

  it("fails on a manifest without platforms", () => {
    assert.throws(() => downloadLinks({ version: "0.1.2" }, ASSETS, REPOSITORY, TAG), /platforms/u);
  });
});
