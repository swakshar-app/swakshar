import { describe, expect, it } from "vitest";

import { boxFromPoints, isUsableBox } from "./placement";

describe("PDF signature placement", () => {
  it("normalizes dragging in either direction", () => {
    expect(boxFromPoints([180, 90], [10, 20])).toEqual([10, 20, 180, 90]);
  });

  it("preserves negative coordinates used by offset crop boxes", () => {
    expect(boxFromPoints([-80.4, 9.6], [90.2, 80.1])).toEqual([-80, 10, 90, 80]);
  });

  it("rejects tiny, inverted and nonfinite rectangles", () => {
    expect(isUsableBox([0, 0, 170, 70])).toBe(true);
    expect(isUsableBox([0, 0, 4, 70])).toBe(false);
    expect(isUsableBox([30, 0, 20, 70])).toBe(false);
    expect(isUsableBox([0, 0, Number.NaN, 70])).toBe(false);
  });
});
