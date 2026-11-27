import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { usePolling } from "./hooks";

vi.mock("./visibility", () => ({ useWindowVisible: () => true }));

afterEach(() => { cleanup(); vi.useRealTimers(); });

describe("polling under slow backend responses", () => {
  it("does not queue more calls or refreshes while one read is pending", async () => {
    vi.useFakeTimers();
    let finish: ((value: string) => void) | undefined;
    const load = vi.fn(() => new Promise<string>((resolve) => { finish = resolve; }));
    const { result } = renderHook(() => usePolling(load, 100));
    await act(async () => { await vi.advanceTimersByTimeAsync(1000); });
    act(() => { result.current.refresh(); result.current.refresh(); });
    expect(load).toHaveBeenCalledTimes(1);
    await act(async () => { finish?.("ready"); });
    expect(result.current.data).toBe("ready");
    await act(async () => { await vi.advanceTimersByTimeAsync(100); });
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("ignores responses from an unmounted view", async () => {
    let finish: ((value: string) => void) | undefined;
    const load = (): Promise<string> => new Promise((resolve) => { finish = resolve; });
    const { result, unmount } = renderHook(() => usePolling(load, 100));
    unmount();
    await act(async () => { finish?.("obsolete"); });
    expect(result.current.data).toBeNull();
  });

  it("does not start work when a shared parent already supplies the data", () => {
    const load = vi.fn(() => Promise.resolve("unused"));
    renderHook(() => usePolling(load, 100, false));
    expect(load).not.toHaveBeenCalled();
  });

  it("reuses a pending read when polling resumes before the backend replies", async () => {
    let finish: ((value: string) => void) | undefined;
    const load = vi.fn(() => new Promise<string>((resolve) => { finish = resolve; }));
    const { result, rerender } = renderHook(({ enabled }) => usePolling(load, 100, enabled), { initialProps: { enabled: true } });
    await act(async () => { await Promise.resolve(); });
    rerender({ enabled: false });
    rerender({ enabled: true });
    expect(load).toHaveBeenCalledTimes(1);
    await act(async () => { finish?.("ready"); });
    expect(result.current.data).toBe("ready");
  });
});
