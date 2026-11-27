/** Fails packaging instead of producing an installer with missing runtime tools. */
import { existsSync } from "node:fs";
import { join } from "node:path";

/** Tauri's resource source directory. */
const ROOT = join(import.meta.dirname, "..", "app", "src-tauri", "resources");
/** Release builds pass macos to require both slices of the universal app. */
const PLATFORM = process.env["SWAKSHAR_BUNDLE_PLATFORM"] ?? process.platform;
/** Each packaged app must carry its own native runtime, not a user's PATH tools. */
const ARCHITECTURES = PLATFORM === "macos" ? ["arm64", "x64"] : [process.arch];
/** Windows is the only executable-suffix variant. */
const EXECUTABLE = process.platform === "win32" ? "swakshar-pdf.exe" : "swakshar-pdf";

for (const arch of ARCHITECTURES) {
  for (const name of [EXECUTABLE, "runtime.json", "PYTHON-LICENSE.md"]) {
    if (!existsSync(join(ROOT, "pdf-engine", arch, name))) { throw new Error(`Missing bundled PDF runtime ${arch}/${name}. Run pnpm bundle:pdf on the native builder.`); }
  }
}
if ((PLATFORM === "macos" || process.platform === "darwin") && !existsSync(join(ROOT, "embridge", "embridge_3.2.7.0.dmg"))) {
  throw new Error("Missing bundled emBridge installer. Run pnpm bundle:embridge before packaging.");
}
