/**
 * Regenerates the bundle icons from the SVG masters in
 * `app/src-tauri/icons/source` with Tauri's own `tauri icon` command, then
 * keeps only the files the bundler and the tray use. Run with
 * `pnpm run icons`.
 */
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Icon directory next to `tauri.conf.json`. */
const ICON_DIR = join(import.meta.dirname, "..", "app", "src-tauri", "icons");
/** App icon master: 1024 canvas, macOS grid, transparent margin. */
const APP_SOURCE = join(ICON_DIR, "source", "app-icon.svg");
/** Menu bar master: black on transparent, used as a template image. */
const TRAY_SOURCE = join(ICON_DIR, "source", "tray-template.svg");
/** Files from `tauri icon` that `tauri.conf.json` and the UI reference. */
const APP_FILES: readonly string[] = ["32x32.png", "128x128.png", "128x128@2x.png", "icon.png", "icon.icns", "icon.ico"];
/** Menu bar icon size in pixels (22 points at 2x). */
const TRAY_SIZE = 44;

/** Runs `tauri icon` on `source` into a fresh temporary directory. */
function tauriIcon(source: string, extra: readonly string[]): string {
  const output = mkdtempSync(join(tmpdir(), "swakshar-icons-"));
  execFileSync("pnpm", ["--filter", "swakshar-ui", "exec", "tauri", "icon", source, "--output", output, ...extra], {
    stdio: ["ignore", "ignore", "inherit"],
  });
  return output;
}

/** Writes the app icons and the menu bar template. */
function main(): void {
  const app = tauriIcon(APP_SOURCE, []);
  for (const name of APP_FILES) {
    copyFileSync(join(app, name), join(ICON_DIR, name));
  }
  rmSync(app, { recursive: true, force: true });
  const tray = tauriIcon(TRAY_SOURCE, ["--png", String(TRAY_SIZE)]);
  copyFileSync(join(tray, `${String(TRAY_SIZE)}x${String(TRAY_SIZE)}.png`), join(ICON_DIR, "tray-template.png"));
  rmSync(tray, { recursive: true, force: true });
  process.stdout.write(`icons written to ${ICON_DIR}\n`);
}

main();
