/** Restores native runtime permissions and relative symlinks from CI artifacts. */
import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync } from "node:fs";
import { join } from "node:path";

/** Archives come only from native build jobs in the same workflow run. */
const SOURCE = join(import.meta.dirname, "..", ".pdf-runtime-build", "artifacts");
/** Tauri copies the fully restored directory into each platform's installer. */
const TARGET = join(import.meta.dirname, "..", "app", "src-tauri", "resources", "pdf-engine");
/** Git Bash's tar does not understand native Windows drive-letter paths. */
const TAR = process.platform === "win32" ? join(process.env["SystemRoot"] ?? "C:\\Windows", "System32", "tar.exe") : "tar";

mkdirSync(TARGET, { recursive: true });
for (const name of readdirSync(SOURCE)) {
  if (/^pdf-runtime-(darwin|linux|win32)-(arm64|x64)\.tar$/.test(name)) {
    execFileSync(TAR, ["-xf", name, "-C", TARGET], { cwd: SOURCE, stdio: "inherit" });
  }
}
