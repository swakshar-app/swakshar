import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { CertView, InventoryView } from "../api/types";
import { TokensPanel } from "./TokensPanel";

vi.mock("../api/commands", () => ({ mainApi: {}, errorText: (reason: unknown) => String(reason) }));

afterEach(cleanup);

/** A certificate held by `holder`. */
function cert(holder: string, signing: boolean): CertView {
  return { holder, issuer: "Test CA", classLabel: signing ? "Class 3" : "Unclassified", validUntil: "01-01-2028", valid: true, expiresSoon: false, signing, hasPan: signing };
}

/** One token carrying a signing certificate and three authority certificates. */
const INVENTORY: InventoryView = {
  modules: [],
  attached: [],
  tokens: [
    {
      name: "Hypersecu HYP2003",
      serial: "****0001",
      pinPad: false,
      pinLocked: false,
      pinFinalTry: false,
      certificates: [cert("TEST HOLDER", true), cert("Root CA", false), cert("Intermediate CA", false), cert("Issuing CA", false)],
    },
  ],
};

describe("TokensPanel", () => {
  it("puts signing certificates first and folds the authorities away", () => {
    render(<TokensPanel tokens={INVENTORY} error={null} showGuide={false} onChange={() => undefined} />);
    expect(screen.getByText("TEST HOLDER").closest("details")).toBeNull();
    expect(screen.getByText("Root CA").closest("details")).not.toBeNull();
    expect(screen.getByText(/3 more certificates/)).toBeTruthy();
  });

  it("says when a token has no signing certificate", () => {
    const token = INVENTORY.tokens[0];
    if (token === undefined) {
      throw new Error("fixture has a token");
    }
    const onlyAuthorities: InventoryView = { ...INVENTORY, tokens: [{ ...token, certificates: [cert("Root CA", false)] }] };
    render(<TokensPanel tokens={onlyAuthorities} error={null} showGuide={false} onChange={() => undefined} />);
    expect(screen.getByText("No signing certificate on this token.")).toBeTruthy();
  });
});
