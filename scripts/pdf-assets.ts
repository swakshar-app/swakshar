/** Copies PDF.js font and character-map assets into the bundled webview. */
import { cpSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

/** Resolve through the UI package because pnpm isolates its dependencies. */
const requireFromApp = createRequire(join(import.meta.dirname, "..", "app", "package.json"));
/** Package assets are distributed unchanged under PDF.js's Apache-2.0 license. */
const source = dirname(requireFromApp.resolve("pdfjs-dist/package.json"));
/** Vite copies public files into the final frontend bundle. */
const target = join(import.meta.dirname, "..", "app", "public", "pdfjs");

mkdirSync(target, { recursive: true });
for (const name of ["cmaps", "standard_fonts", "iccs", "wasm"]) {
  cpSync(join(source, name), join(target, name), { recursive: true });
}
cpSync(join(source, "LICENSE"), join(target, "LICENSE"));
