/**
 * The Swakshar mark as signed-distance shapes: a seal ring holding a
 * handwritten stroke. Coordinates are fractions of the canvas.
 */

/** A 2D point. */
interface Point {
  readonly x: number;
  readonly y: number;
}

/** A cubic Bezier segment. */
type Cubic = readonly [Point, Point, Point, Point];

/** The signature stroke, as three joined cubic segments. */
const STROKE: readonly Cubic[] = [
  [{ x: 0.33, y: 0.57 }, { x: 0.36, y: 0.38 }, { x: 0.46, y: 0.37 }, { x: 0.47, y: 0.5 }],
  [{ x: 0.47, y: 0.5 }, { x: 0.48, y: 0.62 }, { x: 0.56, y: 0.64 }, { x: 0.585, y: 0.52 }],
  [{ x: 0.585, y: 0.52 }, { x: 0.6, y: 0.44 }, { x: 0.65, y: 0.42 }, { x: 0.69, y: 0.46 }],
];

/** Points sampled per segment when flattening the stroke. */
const SAMPLES_PER_SEGMENT = 32;

/** Flattens the stroke into a polyline. */
function flatten(curves: readonly Cubic[]): Point[] {
  const points: Point[] = [];
  for (const [p0, p1, p2, p3] of curves) {
    for (let step = 0; step <= SAMPLES_PER_SEGMENT; step += 1) {
      const t = step / SAMPLES_PER_SEGMENT;
      const u = 1 - t;
      const a = u * u * u;
      const b = 3 * u * u * t;
      const c = 3 * u * t * t;
      const d = t * t * t;
      points.push({ x: a * p0.x + b * p1.x + c * p2.x + d * p3.x, y: a * p0.y + b * p1.y + c * p2.y + d * p3.y });
    }
  }
  return points;
}

/** The stroke as a polyline, computed once. */
const POLYLINE: Point[] = flatten(STROKE);

/** Distance from `p` to the segment `a`-`b`. */
function segmentDistance(p: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lengthSquared = dx * dx + dy * dy;
  const t = lengthSquared === 0 ? 0 : Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / lengthSquared));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

/** Distance from `p` to the stroke's centre line. */
function strokeDistance(p: Point): number {
  let best = Number.POSITIVE_INFINITY;
  for (let index = 1; index < POLYLINE.length; index += 1) {
    const a = POLYLINE[index - 1];
    const b = POLYLINE[index];
    if (a !== undefined && b !== undefined) {
      best = Math.min(best, segmentDistance(p, a, b));
    }
  }
  return best;
}

/** Signed distance to a rounded square centred on the canvas. */
function roundedSquare(p: Point, half: number, radius: number): number {
  const qx = Math.abs(p.x - 0.5) - half + radius;
  const qy = Math.abs(p.y - 0.5) - half + radius;
  return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - radius;
}

/** Coverage from a signed distance in pixels (negative is inside). */
function coverage(distancePixels: number): number {
  return Math.max(0, Math.min(1, 0.5 - distancePixels));
}

/** Coverage of the ring and stroke at `p`, strokes widened by `weight`. */
function markCoverage(p: Point, size: number, ringRadius: number, weight: number): number {
  const ring = Math.abs(Math.hypot(p.x - 0.5, p.y - 0.5) - ringRadius) - weight * 0.4;
  const stroke = strokeDistance({ x: 0.5 + (p.x - 0.5) * (0.245 / ringRadius), y: 0.5 + (p.y - 0.5) * (0.245 / ringRadius) }) * (ringRadius / 0.245) - weight * 0.5;
  return Math.max(coverage(ring * size), coverage(stroke * size));
}

/** Mixes two channel values. */
function mix(from: number, to: number, amount: number): number {
  return Math.round(from + (to - from) * amount);
}

/** The app icon: teal rounded square, white mark. RGBA bytes. */
export function renderAppIcon(size: number): Uint8Array {
  const pixels = new Uint8Array(size * size * 4);
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const p = { x: (x + 0.5) / size, y: (y + 0.5) / size };
      const shape = coverage(roundedSquare(p, 0.4023, 0.181) * size);
      const mark = markCoverage(p, size, 0.245, Math.max(0.034, 2.2 / size));
      const offset = (y * size + x) * 4;
      pixels[offset] = mix(mix(0x1f, 0x0b, p.y), 255, mark);
      pixels[offset + 1] = mix(mix(0x72, 0x3c, p.y), 255, mark);
      pixels[offset + 2] = mix(mix(0x66, 0x37, p.y), 255, mark);
      pixels[offset + 3] = Math.round(shape * 255);
    }
  }
  return pixels;
}

/** The menu bar template icon: black mark on transparent. RGBA bytes. */
export function renderTrayIcon(size: number): Uint8Array {
  const pixels = new Uint8Array(size * size * 4);
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const p = { x: (x + 0.5) / size, y: (y + 0.5) / size };
      const offset = (y * size + x) * 4;
      pixels[offset + 3] = Math.round(markCoverage(p, size, 0.4, 0.09) * 255);
    }
  }
  return pixels;
}
