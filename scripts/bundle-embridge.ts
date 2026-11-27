/** Packages the owner-authorized, unmodified emBridge installer on macOS. */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

/** Vendor binary provenance is recorded without storing binaries in git. */
interface VendorLock { readonly version: string; readonly published: string; readonly archiveSha256: string; readonly installer: string; readonly installerSha256: string; }
/** Only this installer version may enter the app bundle. */
const LOCK = JSON.parse(readFileSync(join(import.meta.dirname, "embridge.lock.json"), "utf8")) as VendorLock;
/** Tauri copies this directory into the app's read-only resources. */
const OUTPUT = join(import.meta.dirname, "..", "app", "src-tauri", "resources", "embridge");

/** Rejects changes to either the upstream archive or installer payload. */
function requireHash(path: string, expected: string): void {
  if (createHash("sha256").update(readFileSync(path)).digest("hex") !== expected) {
    throw new Error(`emBridge ${LOCK.version} checksum mismatch. The vendor may have replaced its download.`);
  }
}

/** Accepts an archived vendor download or a builder-controlled mirror with the same hash. */
async function main(): Promise<void> {
  if (process.platform !== "darwin") { return; }
  if (Date.parse(LOCK.published) > Date.now() - 7 * 86_400_000) { throw new Error("emBridge must be at least seven days old."); }
  const destination = join(OUTPUT, LOCK.installer);
  if (existsSync(destination)) { requireHash(destination, LOCK.installerSha256); return; }
  const archive = process.env["SWAKSHAR_EMBRIDGE_ARCHIVE"];
  const sourceUrl = process.env["SWAKSHAR_EMBRIDGE_ARCHIVE_URL"];
  const work = join(import.meta.dirname, "..", ".pdf-runtime-build", "embridge");
  mkdirSync(work, { recursive: true });
  let selected = archive;
  if (selected === undefined && sourceUrl !== undefined) {
    const parsed = new URL(sourceUrl);
    if (parsed.protocol !== "https:") { throw new Error("Use HTTPS for the emBridge archive mirror."); }
    selected = join(work, "emBridge.zip");
    execFileSync("/usr/bin/curl", ["--fail", "--location", "--proto", "=https", "--max-time", "120", "--output", selected, sourceUrl], { stdio: "inherit" });
  }
  if (selected === undefined) { throw new Error("Set SWAKSHAR_EMBRIDGE_ARCHIVE to the original emBridge 3.2.7.0 ZIP, or SWAKSHAR_EMBRIDGE_ARCHIVE_URL to its HTTPS mirror. Newer downloads must pass a reviewed lock update."); }
  requireHash(selected, LOCK.archiveSha256);
  const extracted = join(work, "extracted");
  mkdirSync(extracted, { recursive: true });
  execFileSync("/usr/bin/ditto", ["-x", "-k", selected, extracted]);
  const installer = join(extracted, LOCK.installer);
  requireHash(installer, LOCK.installerSha256);
  mkdirSync(OUTPUT, { recursive: true });
  cpSync(installer, destination);
}

await main();
