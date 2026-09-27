import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { sectionFor, unwrap } from "./changelog.ts";

/** A changelog with two releases. */
const CHANGELOG = `# Changelog

Intro text.

## 0.2.0

- Second.

## 0.1.0

- First.
- Also first.
`;

describe("sectionFor", () => {
  it("returns a version's section without its heading", () => {
    assert.equal(sectionFor(CHANGELOG, "0.1.0"), "- First.\n- Also first.");
  });

  it("stops at the next release", () => {
    assert.equal(sectionFor(CHANGELOG, "0.2.0"), "- Second.");
  });

  it("accepts a leading v", () => {
    assert.equal(sectionFor(CHANGELOG, "v0.2.0"), "- Second.");
  });

  it("uses the final version's notes for a release candidate", () => {
    assert.equal(sectionFor(CHANGELOG, "0.1.0-rc.2"), "- First.\n- Also first.");
  });

  it("returns null for a version without notes", () => {
    assert.equal(sectionFor(CHANGELOG, "0.3.0"), null);
    assert.equal(sectionFor("## 0.4.0\n\n", "0.4.0"), null);
  });
});

describe("unwrap", () => {
  it("joins a bullet's wrapped lines into one", () => {
    assert.equal(unwrap("- Restart opens Swakshar.\n  Before, it stayed\n  closed.\n- Second."), "- Restart opens Swakshar. Before, it stayed closed.\n- Second.");
  });

  it("joins wrapped paragraphs and keeps the blank lines between them", () => {
    assert.equal(unwrap("The first\nrelease.\n\n- One."), "The first release.\n\n- One.");
  });

  it("leaves one-line bullets alone", () => {
    assert.equal(unwrap("- One.\n- Two."), "- One.\n- Two.");
  });
});
