/** Installs locked runtime packages and compiles the missing Intel Mac crypto wheel. */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** Package artifacts accepted by the runtime's publication-age and hash locks. */
export interface RuntimePackage {
  readonly name: string;
  readonly version: string;
  readonly published: string;
  readonly platforms: readonly string[];
  readonly hashes: readonly string[];
  readonly sourceHashes: readonly string[];
}

/** Static library provenance and license to carry into the frozen runtime. */
export interface NativeCrypto {
  readonly version: string;
  readonly sha256: string;
  readonly license: string;
}

/** Exact upstream archive, independent of the runner's Homebrew packages. */
interface OpenSslLock { readonly version: string; readonly published: string; readonly url: string; readonly sha256: string; }
/** Every other platform receives prebuilt, hash-locked wheels. */
const SOURCE_CRYPTO = process.platform === "darwin" && process.arch === "x64";
/** Common installer arguments prohibit unpinned dependency resolution. */
const PIP_INSTALL = ["-m", "pip", "install", "--disable-pip-version-check", "--no-deps", "--require-hashes"];

/** Writes only exact versions and explicitly accepted hashes into a pip input. */
function requirement(entry: RuntimePackage, hashes: readonly string[]): string {
  return `${entry.name}==${entry.version} ${hashes.map((hash) => `--hash=sha256:${hash}`).join(" ")}`;
}

/** Carries the repository's Rust pin into pip's temporary source-build directory. */
function rustToolchain(): string {
  const file = readFileSync(join(import.meta.dirname, "..", "rust-toolchain.toml"), "utf8");
  const version = /^channel = "([0-9.]+)"$/mu.exec(file)?.[1];
  if (version === undefined) { throw new Error("The crypto builder needs the repository Rust pin."); }
  return version;
}

/** Builds static OpenSSL so the Intel helper never needs a user's Homebrew library. */
function buildOpenSsl(work: string): NativeCrypto & { readonly prefix: string } {
  const lock = JSON.parse(readFileSync(join(import.meta.dirname, "openssl.lock.json"), "utf8")) as OpenSslLock;
  if (Date.parse(lock.published) > Date.now() - 7 * 86_400_000) { throw new Error("OpenSSL must be at least seven days old."); }
  const archive = join(work, "openssl.tar.gz");
  execFileSync("/usr/bin/curl", ["--fail", "--location", "--proto", "=https", "--max-time", "180", "--output", archive, lock.url], { stdio: "inherit" });
  if (createHash("sha256").update(readFileSync(archive)).digest("hex") !== lock.sha256) { throw new Error("OpenSSL source checksum mismatch."); }
  const source = join(work, "openssl-source");
  const prefix = join(work, "openssl-static");
  mkdirSync(source, { recursive: true });
  execFileSync("tar", ["-xzf", archive, "-C", source, "--strip-components", "1"], { stdio: "inherit" });
  const options = { cwd: source, stdio: "inherit" as const, env: { ...process.env, MACOSX_DEPLOYMENT_TARGET: "12.0" } };
  execFileSync("perl", ["Configure", "darwin64-x86_64-cc", "no-shared", "no-tests", "no-module", `--prefix=${prefix}`], options);
  execFileSync("make", ["-j3"], options);
  execFileSync("make", ["install_sw"], options);
  return { version: lock.version, sha256: lock.sha256, license: join(source, "LICENSE.txt"), prefix };
}

/** Installs wheels first, then the original locked crypto source only on Intel macOS. */
export function installRuntimePackages(python: string, packages: readonly RuntimePackage[], work: string): NativeCrypto | null {
  const wheels = packages.filter((entry) => !SOURCE_CRYPTO || entry.name !== "cryptography");
  const path = join(work, "requirements.md");
  writeFileSync(path, `${wheels.map((entry) => requirement(entry, entry.hashes)).join("\n")}\n`);
  execFileSync(python, [...PIP_INSTALL, "--only-binary=:all:", "-r", path], { stdio: "inherit" });
  if (!SOURCE_CRYPTO) { return null; }
  const crypto = packages.find((entry) => entry.name === "cryptography");
  if (crypto === undefined || crypto.sourceHashes.length === 0) { throw new Error("The Intel crypto source hash must be locked."); }
  const native = buildOpenSsl(work);
  const sourcePath = join(work, "crypto-source.md");
  writeFileSync(sourcePath, `${requirement(crypto, crypto.sourceHashes)}\n`);
  execFileSync(python, [...PIP_INSTALL, "--no-binary=:all:", "--no-build-isolation", "-r", sourcePath], {
    stdio: "inherit", env: { ...process.env, RUSTUP_TOOLCHAIN: rustToolchain(), OPENSSL_DIR: native.prefix, OPENSSL_STATIC: "1", MACOSX_DEPLOYMENT_TARGET: "12.0" },
  });
  return native;
}
