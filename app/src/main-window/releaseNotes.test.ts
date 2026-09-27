import { describe, expect, it } from "vitest";

import { noteBlocks } from "./releaseNotes";

describe("noteBlocks", () => {
  it("joins a bullet's wrapped lines", () => {
    expect(noteBlocks("- Restart opens Swakshar.\n  Before, it stayed\n  closed.\n- Second.")).toEqual([
      { kind: "list", items: ["Restart opens Swakshar. Before, it stayed closed.", "Second."] },
    ]);
  });

  it("keeps paragraphs and lists apart", () => {
    expect(noteBlocks("The first\nrelease.\n\n- One.\n- Two.")).toEqual([
      { kind: "paragraph", text: "The first release." },
      { kind: "list", items: ["One.", "Two."] },
    ]);
  });

  it("treats plain text as one paragraph and nothing as nothing", () => {
    expect(noteBlocks("Faster start.")).toEqual([{ kind: "paragraph", text: "Faster start." }]);
    expect(noteBlocks("  \n")).toEqual([]);
  });
});
