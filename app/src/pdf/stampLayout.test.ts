import { describe, expect, it } from "vitest";

import { stampDate, stampLayout, stampStyle } from "./stampLayout";

describe("approved DSC stamp layout", () => {
  it("keeps full text inside small placements", () => {
    for (const [width = 0, height = 0] of [[12, 12], [170, 70], [100, 24]]) {
      const { scale, lines } = stampLayout(width, height, "Example Long Certificate Holder Name", "2026.11.27 12:00:00 UTC");
      const needed = (2 * stampStyle.padding + stampStyle.headSize * 1.15 + lines.length * stampStyle.leading) * scale;
      expect(needed).toBeLessThanOrEqual(height + 0.01);
      expect(lines.join(" ")).toContain("Digitally signed by");
    }
  });

  it("shows an explicit UTC date", () => {
    expect(stampDate(new Date("2026-11-27T12:00:00Z"))).toBe("2026.11.27 12:00:00 UTC");
  });
});
