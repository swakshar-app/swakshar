import { describe, expect, it } from "vitest";

import { messageFor } from "./messages";

describe("messageFor", () => {
  it("warns before the last PIN try", () => {
    const message = messageFor({ status: "pin-incorrect", countLow: true, finalTry: true, locked: false });
    expect(message?.tone).toBe("danger");
    expect(message?.text).toContain("One more wrong PIN will lock this token");
  });

  it("says when a wrong PIN locked the token", () => {
    expect(messageFor({ status: "pin-incorrect", countLow: true, finalTry: false, locked: true })?.text).toContain("now locked");
  });

  it("asks for care after an ordinary wrong PIN", () => {
    expect(messageFor({ status: "pin-incorrect", countLow: false, finalTry: false, locked: false })?.text).toContain("check it carefully");
  });

  it("asks for a PIN when none was entered", () => {
    expect(messageFor({ status: "pin-required" })).toEqual({ tone: "warn", text: "Enter the token PIN." });
  });

  it("passes a failure message through", () => {
    expect(messageFor({ status: "failed", message: "The token was removed." })).toEqual({ tone: "danger", text: "The token was removed." });
  });

  it("has nothing to say after signing", () => {
    expect(messageFor({ status: "signed", holder: "TEST HOLDER" })).toBeNull();
  });
});
