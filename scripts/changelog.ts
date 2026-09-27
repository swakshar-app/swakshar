/**
 * Reads release notes out of CHANGELOG.md, where each release is a
 * `## x.y.z` section.
 */

/** Heading prefix of a release section. */
const SECTION = "## ";

/**
 * The notes under `## <version>`, without the heading, or `null` when the
 * version has no section or an empty one. A leading `v` is ignored, and a
 * pre-release such as `0.1.0-rc.1` uses the notes of `0.1.0`.
 */
export function sectionFor(changelog: string, version: string): string | null {
  const plain = version.replace(/^v/u, "");
  const lines = changelog.split("\n");
  for (const wanted of [plain, plain.split("-")[0] ?? plain]) {
    const start = lines.findIndex((line) => line.trim() === `${SECTION}${wanted}`);
    if (start >= 0) {
      const rest = lines.slice(start + 1);
      const end = rest.findIndex((line) => line.startsWith(SECTION));
      const body = (end < 0 ? rest : rest.slice(0, end)).join("\n").trim();
      return body === "" ? null : body;
    }
  }
  return null;
}
