import { describe, expect, it } from "vitest";

import { onMac } from "./platform";

describe("onMac", () => {
  it("recognises the macOS web view", () => {
    expect(onMac("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)")).toBe(true);
  });

  it("rejects Windows and Linux web views", () => {
    expect(onMac("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0")).toBe(false);
    expect(onMac("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)")).toBe(false);
  });
});
