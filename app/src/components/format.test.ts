import { describe, expect, it } from "vitest";

import { formatCountdown, hostOf } from "./format";

describe("formatCountdown", () => {
  it("shows minutes and zero-padded seconds", () => {
    expect(formatCountdown(65)).toBe("1:05");
    expect(formatCountdown(300)).toBe("5:00");
  });

  it("never shows a negative time", () => {
    expect(formatCountdown(-3)).toBe("0:00");
  });

  it("drops fractions of a second", () => {
    expect(formatCountdown(59.9)).toBe("0:59");
  });
});

describe("hostOf", () => {
  it("returns the host of an origin", () => {
    expect(hostOf("https://services.gst.gov.in")).toBe("services.gst.gov.in");
  });

  it("keeps text that is not a URL", () => {
    expect(hostOf("not a url")).toBe("not a url");
  });
});
