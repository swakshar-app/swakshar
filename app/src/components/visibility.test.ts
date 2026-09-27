import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { useWindowVisible } from "./visibility";

describe("useWindowVisible", () => {
  it("imports and runs outside the app shell, following the page", () => {
    const { result } = renderHook(() => useWindowVisible());
    expect(result.current).toBe(document.visibilityState === "visible");
  });
});
