/**
 * Release notes arrive as the changelog's Markdown: paragraphs and "- "
 * bullets, possibly wrapped at 80 columns. The update card shows them as
 * paragraphs and list items instead of raw text.
 */

/** A paragraph, or a run of bullets. */
export type NoteBlock =
  | { readonly kind: "paragraph"; readonly text: string }
  | { readonly kind: "list"; readonly items: readonly string[] };

/** Marker of a list item. */
const BULLET = "- ";

/** Splits notes into blocks, joining each bullet's and paragraph's wrapped lines. */
export function noteBlocks(notes: string): NoteBlock[] {
  const blocks: NoteBlock[] = [];
  let open = false;
  for (const raw of notes.split("\n")) {
    const line = raw.trim();
    const last = blocks.at(-1);
    if (line === "") {
      open = false;
    } else if (line.startsWith(BULLET)) {
      const item = line.slice(BULLET.length);
      if (open && last?.kind === "list") {
        blocks[blocks.length - 1] = { kind: "list", items: [...last.items, item] };
      } else {
        blocks.push({ kind: "list", items: [item] });
      }
      open = true;
    } else if (open && last?.kind === "list") {
      const items = [...last.items];
      items[items.length - 1] = `${items.at(-1) ?? ""} ${line}`;
      blocks[blocks.length - 1] = { kind: "list", items };
    } else if (open && last?.kind === "paragraph") {
      blocks[blocks.length - 1] = { kind: "paragraph", text: `${last.text} ${line}` };
    } else {
      blocks.push({ kind: "paragraph", text: line });
      open = true;
    }
  }
  return blocks;
}
