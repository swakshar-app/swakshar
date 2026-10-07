import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AttachedView, InventoryView } from "../api/types";
import { TokenGuide } from "./TokenGuide";

const mocks = vi.hoisted(() => ({ openDriverPage: vi.fn(() => Promise.resolve()) }));
vi.mock("../api/commands", () => ({
  mainApi: { openDriverPage: mocks.openDriverPage, addDriver: () => Promise.resolve(null) },
  errorText: (reason: unknown) => String(reason),
}));

afterEach(() => {
  cleanup();
  mocks.openDriverPage.mockClear();
});

/** One plugged-in token that no driver reads yet. */
function missing(driverPage: string | null): InventoryView {
  const token: AttachedView = { maker: "Hypersecu", product: "USB TOKEN", family: "ePass2003 / HYP2003", state: "missing", detail: null, vendorId: 0x2ccf, driverPage };
  return { modules: [], tokens: [], attached: [token] };
}

describe("TokenGuide", () => {
  it("opens the maker's download page for its vendor id", () => {
    render(<TokenGuide tokens={missing("https://example.invalid/drivers")} onChange={() => undefined} />);
    fireEvent.click(screen.getByText("Get the driver"));
    expect(mocks.openDriverPage).toHaveBeenCalledWith(0x2ccf);
  });

  it("leaves the button out when no page is known", () => {
    render(<TokenGuide tokens={missing(null)} onChange={() => undefined} />);
    expect(screen.queryByText("Get the driver")).toBeNull();
    expect(screen.getByText("Check again")).toBeTruthy();
  });
});
