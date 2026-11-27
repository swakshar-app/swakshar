/** Coordinate helpers shared by pointer placement and its regression tests. */
import type { PdfBox } from "../api/signing";

/** Smallest visible signature box, matching backend validation. */
const MIN_BOX_POINTS = 12;

/** Normalizes two PDF-space points, including reverse drags and negative origins. */
export function boxFromPoints(start: readonly number[], end: readonly number[]): PdfBox {
  const [x1 = 0, y1 = 0] = start;
  const [x2 = 0, y2 = 0] = end;
  return [Math.round(Math.min(x1, x2)), Math.round(Math.min(y1, y2)), Math.round(Math.max(x1, x2)), Math.round(Math.max(y1, y2))];
}

/** Prevents empty, inverted and unusably small stamp rectangles. */
export function isUsableBox(box: PdfBox): boolean {
  return box.every(Number.isFinite) && box[2] - box[0] >= MIN_BOX_POINTS && box[3] - box[1] >= MIN_BOX_POINTS;
}
