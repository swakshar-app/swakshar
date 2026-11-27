/** Builds an offline, self-contained pyHanko runtime on a native release runner. */
import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { installRuntimePackages, type RuntimePackage } from "./install-pdf-dependencies.ts";

/** Exact versions and accepted wheel hashes, reviewed through the age gate. */
interface RuntimeLock {
  readonly python: string;
  readonly minimumAgeDays: number;
  readonly packages: readonly RuntimePackage[];
}

/** All build intermediates are ignored and never shipped as repository source. */
const ROOT = join(import.meta.dirname, "..");
/** Packaging dependency versions and hashes. */
const LOCK = JSON.parse(readFileSync(join(import.meta.dirname, "pdf-runtime.lock.json"), "utf8")) as RuntimeLock;
/** Uses a builder-installed interpreter; end users receive this interpreter bundled. */
const PYTHON = process.env["SWAKSHAR_BUILD_PYTHON"] ?? (process.platform === "win32" ? "python" : "python3");
/** Architecture folders let a universal Mac app carry both native runtimes. */
const ARCH = process.arch;
/** Private build environment, isolated from globally installed packages. */
const WORK = join(ROOT, ".pdf-runtime-build", `${process.platform}-${ARCH}`);
/** Every native bundle lands under Tauri's resource directory. */
const OUTPUT = join(ROOT, "app", "src-tauri", "resources", "pdf-engine", ARCH);

/** Runs an argument-vector process; no shell interpolation or package code at lock time. */
function run(executable: string, args: readonly string[], input?: string): void {
  execFileSync(executable, args, { cwd: ROOT, stdio: input === undefined ? "inherit" : ["pipe", "inherit", "inherit"], ...(input === undefined ? {} : { input }) });
}

/** Installs hash-locked wheels and packages the upstream entrypoint unchanged. */
function main(): void {
  const version = execFileSync(PYTHON, ["--version"], { encoding: "utf8" }).trim();
  if (version !== `Python ${LOCK.python}`) { throw new Error(`Build with Python ${LOCK.python}; received ${version}.`); }
  const packages = LOCK.packages.filter((entry) => entry.platforms.includes(process.platform));
  const cutoff = Date.now() - LOCK.minimumAgeDays * 86_400_000;
  if (packages.some((entry) => Date.parse(entry.published) > cutoff || entry.hashes.length === 0)) {
    throw new Error("A locked PDF runtime dependency is too new or lacks hashes.");
  }
  mkdirSync(WORK, { recursive: true });
  const environment = join(WORK, "venv");
  run(PYTHON, ["-m", "venv", environment]);
  const python = join(environment, process.platform === "win32" ? "Scripts/python.exe" : "bin/python");
  const nativeCrypto = installRuntimePackages(python, packages, WORK);
  run(python, ["-m", "pip", "check"]);
  const location = JSON.parse(execFileSync(python, ["-c", "import json,sysconfig; print(json.dumps({'site':sysconfig.get_path('purelib')}))"], { encoding: "utf8" })) as { readonly site: string };
  const collect = ["pyhanko", "pyhanko.cli", "pyhanko_certvalidator", "pkcs11", "asn1crypto", "oscrypto", "tzlocal", "tzdata", "certifi"];
  const identity = process.env["APPLE_SIGNING_IDENTITY"];
  if (process.platform === "darwin" && process.env["CI"] === "true" && !identity) {
    throw new Error("macOS release runtimes require the existing Developer ID signing identity.");
  }
  const signing = process.platform === "darwin" && identity ? ["--codesign-identity", identity,
    "--osx-entitlements-file", join(ROOT, "app", "src-tauri", "Entitlements.plist")] : [];
  run(python, ["-m", "PyInstaller", "--noconfirm", "--onedir", "--name", "swakshar-pdf",
    "--distpath", join(WORK, "dist"), "--workpath", join(WORK, "work"), "--specpath", WORK,
    ...collect.flatMap((name) => ["--collect-all", name]),
    ...packages.flatMap((entry) => ["--copy-metadata", entry.name]),
    ...signing,
    join(location.site, "pyhanko", "__main__.py")]);
  mkdirSync(OUTPUT, { recursive: true });
  cpSync(join(WORK, "dist", "swakshar-pdf"), OUTPUT, { recursive: true, verbatimSymlinks: true });
  const pythonLicense = join(ROOT, "docs", "third-party", "PYTHON.md");
  if (!existsSync(pythonLicense)) { throw new Error("The Python distribution license must accompany the bundled runtime."); }
  cpSync(pythonLicense, join(OUTPUT, "PYTHON-LICENSE.md"));
  if (nativeCrypto !== null) { cpSync(nativeCrypto.license, join(OUTPUT, "OPENSSL-LICENSE.md")); }
  writeFileSync(join(OUTPUT, "runtime.json"), `${JSON.stringify({ python: LOCK.python, arch: ARCH, ...(nativeCrypto === null ? {} : { openssl: { version: nativeCrypto.version, sha256: nativeCrypto.sha256 } }), packages: packages.map(({ name, version: pinned }) => ({ name, version: pinned })) }, null, 2)}\n`);
  const executable = join(OUTPUT, process.platform === "win32" ? "swakshar-pdf.exe" : "swakshar-pdf");
  run(executable, ["--no-plugins", "--help"]);
  run(executable, ["--no-plugins", "sign", "addsig", "pkcs11", "--help"]);
  const archive = join(ROOT, ".pdf-runtime-build", `pdf-runtime-${process.platform}-${ARCH}.tar`);
  run("tar", ["-cf", archive, "-C", join(OUTPUT, ".."), ARCH]);
}

main();
