import { describe, expect, it } from "vitest";

import { overview } from "../testing/fixtures";
import { statusText } from "./statusText";

describe("statusText", () => {
  it("is ready when signing is on and the certificate is trusted", () => {
    expect(statusText(overview())).toMatchObject({ title: "Ready for the GST portal", tone: "success" });
  });

  it("names the port it listens on", () => {
    expect(statusText(overview()).detail).toContain("127.0.0.1:1585");
  });

  it("asks to finish setup while the certificate is untrusted", () => {
    const text = statusText(overview({ trust: { status: "not-trusted", validUntil: null, command: null } }));
    expect(text).toMatchObject({ title: "Finish setup", tone: "warn" });
  });

  it("says signing is off when paused", () => {
    expect(statusText(overview({ server: { state: "paused", port: null, error: null } })).title).toBe("Signing is off");
  });

  it("shows the start-up error", () => {
    const text = statusText(overview({ server: { state: "failed", port: null, error: "No free port." } }));
    expect(text).toEqual({ title: "Signing could not start", detail: "No free port.", tone: "danger" });
  });

  it("puts a waiting request first", () => {
    expect(statusText(overview({ waiting: true })).title).toBe("Waiting for your approval");
  });
});
