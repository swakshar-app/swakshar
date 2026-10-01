import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { useWindowVisible } from "./visibility";

/** Global the Tauri shell injects. */
const TAURI_INTERNALS = "__TAURI_INTERNALS__";

const tauri = vi.hoisted(() => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
  isVisible: vi.fn(() => Promise.resolve(true)),
}));

vi.mock("@tauri-apps/api/event", () => ({ listen: tauri.listen }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main", isVisible: tauri.isVisible }) }));

afterEach(() => {
  Reflect.deleteProperty(window, TAURI_INTERNALS);
  vi.clearAllMocks();
});

describe("useWindowVisible", () => {
  it("imports and runs outside the app shell, following the page", () => {
    const { result } = renderHook(() => useWindowVisible());
    expect(result.current).toBe(document.visibilityState === "visible");
  });

  it("listens only to visibility events sent to its own window", async () => {
    vi.resetModules();
    Object.defineProperty(window, TAURI_INTERNALS, { value: {}, configurable: true });
    const fresh = await import("./visibility");
    renderHook(() => fresh.useWindowVisible());
    expect(tauri.listen).toHaveBeenCalledWith("window-visibility", expect.any(Function), { target: "main" });
  });
});
