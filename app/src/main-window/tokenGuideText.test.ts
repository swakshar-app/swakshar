import { describe, expect, it } from "vitest";

import type { AttachedView } from "../api/types";
import { advice, tokenName } from "./tokenGuideText";

/** A plugged-in token with the given fields. */
function attached(fields: Partial<AttachedView>): AttachedView {
  return { maker: "Hypersecu", product: "USB TOKEN", family: "ePass2003 / HYP2003", state: "missing", detail: null, vendorId: 0x2ccf, driverPage: "https://example.invalid/drivers", ...fields };
}

describe("tokenName", () => {
  it("joins maker and product", () => {
    expect(tokenName(attached({}))).toBe("Hypersecu USB TOKEN");
  });

  it("does not repeat a maker the product already names", () => {
    expect(tokenName(attached({ maker: "Feitian", product: "FEITIAN ePass2003" }))).toBe("FEITIAN ePass2003");
  });

  it("falls back to a generic name", () => {
    expect(tokenName(attached({ maker: null, product: null }))).toBe("A DSC token");
  });
});

describe("advice", () => {
  it("names the driver family to install", () => {
    expect(advice(attached({}))).toContain("Install the ePass2003 / HYP2003 driver for macOS");
  });

  it("points at Get the driver when the maker's page is known", () => {
    expect(advice(attached({}))).toContain("Get the driver");
  });

  it("falls back to the Certifying Authority when no page is known", () => {
    expect(advice(attached({ driverPage: null }))).toContain("Certifying Authority");
  });

  it("asks for the token's own driver when the family is unknown", () => {
    expect(advice(attached({ family: null }))).toContain("the driver that came with it");
  });

  it("passes on the driver's own error", () => {
    expect(advice(attached({ state: "failed", detail: "CKR_GENERAL_ERROR" }))).toContain("(CKR_GENERAL_ERROR)");
  });

  it("points at the other processor", () => {
    expect(advice(attached({ state: "other-architecture" }))).toContain("different processor");
  });

  it("says nothing for a ready token", () => {
    expect(advice(attached({ state: "ready" }))).toBe("");
  });
});
