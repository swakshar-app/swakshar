import { describe, expect, it } from "vitest";

import { heading } from "./heading";

describe("heading", () => {
  it("names the GST portal for gst.gov.in and its subdomains", () => {
    expect(heading("https://gst.gov.in")).toBe("The GST portal wants your signature");
    expect(heading("https://services.gst.gov.in")).toBe("The GST portal wants your signature");
  });

  it("does not call a look-alike domain the GST portal", () => {
    expect(heading("https://gst.gov.in.example.com")).toBe("gst.gov.in.example.com wants your signature");
    expect(heading("https://fakegst.gov.in")).toBe("fakegst.gov.in wants your signature");
  });

  it("names any other allowed site by its host", () => {
    expect(heading("https://example.gov.in")).toBe("example.gov.in wants your signature");
  });
});
