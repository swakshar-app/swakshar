/** Converts a pip JSON resolution into exact, age-gated PDF runtime pins and hashes. */
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** Minimum age for every wheel accepted into the runtime lock. */
const MIN_RELEASE_MS = 7 * 24 * 60 * 60 * 1000;
/** Conditional packages required on platforms other than the resolution host. */
const EXTRA_PACKAGES = ["pefile", "pywin32-ctypes", "tzdata", "typing_extensions", "colorama"];
/** Only these dependencies are platform-specific with Python 3.12. */
const PLATFORMS: Readonly<Record<string, readonly string[]>> = {
  macholib: ["darwin"], pefile: ["win32"], "pywin32-ctypes": ["win32"], colorama: ["win32"],
};

/** PyPI file metadata needed for version age and pip hash checking. */
interface ReleaseFile { readonly digests: { readonly sha256: string }; readonly filename: string; readonly upload_time_iso_8601: string; }
/** PyPI's stable JSON API, reduced to the fields used here. */
interface PackageIndex { readonly releases: Readonly<Record<string, readonly ReleaseFile[]>>; }
/** pip's installation report contains the resolver's chosen name and version. */
interface Resolution { readonly install: readonly { readonly metadata: { readonly name: string; readonly version: string } }[]; }

/** Compares stable numeric release versions without using lexicographic ordering. */
function compare(left: string, right: string): number {
  return left.localeCompare(right, "en", { numeric: true });
}

/** Fetches official wheel metadata; no package code is installed or executed. */
async function pin(name: string, maximum: string | null): Promise<unknown> {
  const response = await fetch(`https://pypi.org/pypi/${encodeURIComponent(name)}/json`);
  if (!response.ok) { throw new Error(`PyPI metadata unavailable for ${name}`); }
  const index = await response.json() as PackageIndex;
  const cutoff = Date.now() - MIN_RELEASE_MS;
  const versions = Object.entries(index.releases).filter(([version, files]) =>
    /^\d+(?:\.\d+)*$/.test(version) && (maximum === null || compare(version, maximum) <= 0)
    && files.some((file) => file.filename.endsWith(".whl") && Date.parse(file.upload_time_iso_8601) <= cutoff),
  ).toSorted(([left], [right]) => compare(right, left));
  const selected = versions[0];
  if (selected === undefined) { throw new Error(`No aged wheel release for ${name}`); }
  const [version, files] = selected;
  const wheels = files.filter((file) => file.filename.endsWith(".whl") && Date.parse(file.upload_time_iso_8601) <= cutoff);
  return { name, version, platforms: PLATFORMS[name.toLowerCase()] ?? ["darwin", "linux", "win32"],
    published: wheels.map((file) => file.upload_time_iso_8601).toSorted()[0], hashes: [...new Set(wheels.map((file) => file.digests.sha256))] };
}

/** Writes JSON only; pip requirement lines are generated in memory when building. */
async function main(): Promise<void> {
  const reportPath = process.argv[2];
  if (reportPath === undefined) { throw new Error("Pass the pip --report JSON path."); }
  const report = JSON.parse(readFileSync(reportPath, "utf8")) as Resolution;
  const packages = new Map(report.install.map(({ metadata }) => [metadata.name.toLowerCase(), metadata.version]));
  for (const name of EXTRA_PACKAGES) { if (!packages.has(name)) { packages.set(name, ""); } }
  const pins = await Promise.all([...packages].toSorted(([a], [b]) => a.localeCompare(b)).map(([name, version]) => pin(name, version || null)));
  const output = { python: "3.12.16", minimumAgeDays: 7, packages: pins };
  const lines = ["{", `  "python": ${JSON.stringify(output.python)},`, "  \"minimumAgeDays\": 7,", "  \"packages\": [",
    ...pins.map((entry, index) => `    ${JSON.stringify(entry)}${index === pins.length - 1 ? "" : ","}`), "  ]", "}", ""];
  writeFileSync(join(import.meta.dirname, "pdf-runtime.lock.json"), lines.join("\n"));
}

await main();
