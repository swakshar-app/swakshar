/**
 * Fails when any authored file is longer than 300 lines. Lockfiles and
 * generated output are skipped. Run with `node scripts/check-line-limit.ts`.
 */
import { readdir, readFile } from "node:fs/promises";
import { extname, join, relative } from "node:path";

/** Repository root. */
const ROOT = join(import.meta.dirname, "..");
/** The limit. Not 301. */
const MAX_LINES = 300;
/** Extensions of authored files. */
const EXTENSIONS = new Set([".rs", ".ts", ".tsx", ".css", ".html", ".toml", ".yml", ".yaml", ".md", ".json", ".plist"]);
/** Directories that hold dependencies or build output. */
const SKIP_DIRS = new Set([".git", "node_modules", "target", "dist", "gen"]);
/** Generated files. */
const SKIP_FILES = new Set(["pnpm-lock.yaml", "Cargo.lock"]);

/** A file and its line count. */
interface Count {
  readonly file: string;
  readonly lines: number;
}

/** Every authored file under `dir`. */
async function walk(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map(async (entry): Promise<string[]> => {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) {
        return SKIP_DIRS.has(entry.name) ? [] : walk(path);
      }
      return entry.isFile() && EXTENSIONS.has(extname(entry.name)) && !SKIP_FILES.has(entry.name) ? [path] : [];
    }),
  );
  return nested.flat();
}

/** Lines in a file, not counting a final newline. */
async function countLines(file: string): Promise<Count> {
  const text = await readFile(file, "utf8");
  const parts = text.split("\n");
  return { file: relative(ROOT, file), lines: text.endsWith("\n") ? parts.length - 1 : parts.length };
}

/** Reports offenders and sets a failing exit code. */
async function main(): Promise<void> {
  const counts = await Promise.all((await walk(ROOT)).map(countLines));
  const offenders = counts.filter((count) => count.lines > MAX_LINES).toSorted((a, b) => b.lines - a.lines);
  for (const offender of offenders) {
    process.stderr.write(`${offender.file}: ${String(offender.lines)} lines (limit ${String(MAX_LINES)})\n`);
  }
  if (offenders.length > 0) {
    process.exitCode = 1;
    return;
  }
  process.stdout.write(`${String(counts.length)} files checked, all within ${String(MAX_LINES)} lines.\n`);
}

await main();
