/**
 * Generates every icon the app bundle needs from the vector mark in
 * `icon-art.ts`: PNG sizes, `icon.icns`, `icon.ico` and the menu bar
 * template. Run with `node scripts/generate-icons.ts`.
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { renderAppIcon, renderTrayIcon } from "./icon-art.ts";
import { encodePng } from "./png.ts";

/** Output directory. */
const ICON_DIR = join(import.meta.dirname, "..", "app", "src-tauri", "icons");

/** PNG files Tauri's bundler reads, with their pixel sizes. */
const PNG_FILES: ReadonlyArray<readonly [string, number]> = [
  ["32x32.png", 32],
  ["128x128.png", 128],
  ["128x128@2x.png", 256],
  ["icon.png", 512],
];

/** ICNS entries (PNG payloads) and their pixel sizes. */
const ICNS_ENTRIES: ReadonlyArray<readonly [string, number]> = [
  ["icp4", 16],
  ["icp5", 32],
  ["ic07", 128],
  ["ic08", 256],
  ["ic09", 512],
  ["ic10", 1024],
  ["ic11", 32],
  ["ic12", 64],
  ["ic13", 256],
  ["ic14", 512],
];

/** ICO sizes (PNG payloads). */
const ICO_SIZES: readonly number[] = [16, 24, 32, 48, 64, 128, 256];

/** Menu bar icon size in pixels; macOS scales it to the menu bar height. */
const TRAY_SIZE = 44;

/** Rendered PNGs by size, so each size renders once. */
const cache = new Map<number, Buffer>();

/** The app icon at `size` pixels, as PNG. */
function appPng(size: number): Buffer {
  const cached = cache.get(size);
  if (cached !== undefined) {
    return cached;
  }
  const png = encodePng(size, size, renderAppIcon(size));
  cache.set(size, png);
  return png;
}

/** Apple icon file: `icns` header, then typed PNG entries. */
function icns(): Buffer {
  const entries = ICNS_ENTRIES.map(([type, size]) => {
    const data = appPng(size);
    const header = Buffer.alloc(8);
    header.write(type, 0, "ascii");
    header.writeUInt32BE(data.length + 8, 4);
    return Buffer.concat([header, data]);
  });
  const body = Buffer.concat(entries);
  const header = Buffer.alloc(8);
  header.write("icns", 0, "ascii");
  header.writeUInt32BE(body.length + 8, 4);
  return Buffer.concat([header, body]);
}

/** Windows icon file: directory, entries, then PNG payloads. */
function ico(): Buffer {
  const images = ICO_SIZES.map((size) => appPng(size));
  const directory = Buffer.alloc(6 + ICO_SIZES.length * 16);
  directory.writeUInt16LE(0, 0);
  directory.writeUInt16LE(1, 2);
  directory.writeUInt16LE(ICO_SIZES.length, 4);
  let offset = directory.length;
  ICO_SIZES.forEach((size, index) => {
    const entry = 6 + index * 16;
    const length = images[index]?.length ?? 0;
    directory.writeUInt8(size >= 256 ? 0 : size, entry);
    directory.writeUInt8(size >= 256 ? 0 : size, entry + 1);
    directory.writeUInt16LE(1, entry + 4);
    directory.writeUInt16LE(32, entry + 6);
    directory.writeUInt32LE(length, entry + 8);
    directory.writeUInt32LE(offset, entry + 12);
    offset += length;
  });
  return Buffer.concat([directory, ...images]);
}

/** Writes every icon file. */
function main(): void {
  mkdirSync(ICON_DIR, { recursive: true });
  for (const [name, size] of PNG_FILES) {
    writeFileSync(join(ICON_DIR, name), appPng(size));
  }
  writeFileSync(join(ICON_DIR, "icon.icns"), icns());
  writeFileSync(join(ICON_DIR, "icon.ico"), ico());
  writeFileSync(join(ICON_DIR, "tray-template.png"), encodePng(TRAY_SIZE, TRAY_SIZE, renderTrayIcon(TRAY_SIZE)));
  process.stdout.write(`icons written to ${ICON_DIR}\n`);
}

main();
