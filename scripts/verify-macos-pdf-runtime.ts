/** Checks signed runtime structure on native macOS CI builders before app packaging. */
import { execFileSync } from "node:child_process";
import { lstatSync } from "node:fs";
import { join } from "node:path";

/** Framework links must survive every copy and artifact transfer. */
const FRAMEWORK_LINKS = ["Python", "Resources", "Versions/Current"] as const;
/** A strict signature check rejects changed sealed files and malformed frameworks. */
const VERIFY_ARGUMENTS = ["--verify", "--deep", "--strict", "--verbose=2"] as const;

/** Verifies the assembled framework and helper without changing their signatures. */
export function verifyMacPdfRuntime(root: string): void {
  if (process.platform !== "darwin") { return; }
  const framework = join(root, "_internal", "Python.framework");
  for (const link of FRAMEWORK_LINKS) {
    if (!lstatSync(join(framework, link)).isSymbolicLink()) {
      throw new Error(`The bundled Python framework lost its ${link} symlink.`);
    }
  }
  for (const target of [framework, join(root, "swakshar-pdf")]) {
    execFileSync("/usr/bin/codesign", [...VERIFY_ARGUMENTS, target], { stdio: "inherit" });
  }
}
