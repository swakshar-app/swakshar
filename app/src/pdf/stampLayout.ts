/** Shared Helvetica metrics and geometry from the approved DSC CLI stamp. */
import style from "./stamp-layout.json";

/** Measures Helvetica text using the same advances as the native PDF writer. */
function textWidth(text: string, size: number): number {
  return [...text].reduce((sum, character) => sum + (style.widths[character.charCodeAt(0) - 32] ?? 556), 0) * size / 1000;
}

/** Wraps text by measured width, splitting an oversized token if necessary. */
function wrap(text: string, size: number, width: number): string[] {
  const lines: string[] = [];
  let current = "";
  for (const character of text.replace(/\s+/gu, " ").trim()) {
    if (current !== "" && textWidth(current + character, size) > width) {
      const space = current.lastIndexOf(" ");
      if (space > 0) { lines.push(current.slice(0, space)); current = current.slice(space + 1); }
      else { lines.push(current); current = ""; }
    }
    current += character;
  }
  if (current.trim() !== "") { lines.push(current.trim()); }
  return lines;
}

/** Matches native UTC timestamp formatting while the owner positions the stamp. */
export function stampDate(date: Date): string {
  return `${date.toISOString().slice(0, 19).replaceAll("-", ".").replace("T", " ")} UTC`;
}

/** Fits the approved typography into a small placement without clipping labels. */
export function stampLayout(width: number, height: number, signer: string, date: string): {
  readonly scale: number; readonly lines: readonly string[]; readonly top: number;
} {
  let scale = Math.min(1, width / 100, height / 60);
  let lines: string[] = [];
  for (let pass = 0; pass < 8; pass += 1) {
    lines = [...wrap(`Digitally signed by ${signer}`, style.bodySize * scale, width - 2 * style.padding * scale),
      ...wrap(`Date: ${date}`, style.bodySize * scale, width - 2 * style.padding * scale)];
    const needed = (2 * style.padding + style.headSize * 1.15 + lines.length * style.leading) * scale;
    if (needed <= height) { break; }
    scale *= height / needed;
  }
  return { scale, lines, top: (style.padding + style.headSize * 0.8) * scale };
}

export { style as stampStyle };
