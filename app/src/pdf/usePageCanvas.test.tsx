import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type { PDFDocumentProxy } from "pdfjs-dist";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { usePageCanvas } from "./usePageCanvas";

beforeEach(() => {
  vi.stubGlobal("ResizeObserver", class {
    /** No resize is emitted unless the test requests one. */
    observe(): void { /* Initial width is sufficient for these lifecycle tests. */ }
    /** Releases the mock subscription. */
    disconnect(): void { /* No native observer was created. */ }
  });
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe("PDF canvas rendering", () => {
  it("cancels an obsolete render and ignores its eventual rejection", async () => {
    let reject: ((reason: Error) => void) | undefined;
    const cancel = vi.fn();
    const render = vi.fn(() => ({ promise: new Promise<void>((_resolve, fail) => { reject = fail; }), cancel }));
    const page = { getViewport: () => ({ width: 650, height: 900 }), render };
    const pdf = { getPage: vi.fn(() => Promise.resolve(page)) } as unknown as PDFDocumentProxy;
    const canvas = { current: document.createElement("canvas") };
    const container = { current: document.createElement("div") };
    const { result, unmount } = renderHook(() => usePageCanvas(pdf, 1, canvas, container));
    await waitFor(() => { expect(render).toHaveBeenCalledOnce(); });
    unmount();
    expect(cancel).toHaveBeenCalledOnce();
    await act(async () => { reject?.(new Error("Rendering cancelled")); });
    expect(result.current.error).toBeNull();
  });

  it("bounds canvas pixels even for unusually tall pages", async () => {
    const view = { width: 650, height: 20_000 };
    const page = { getViewport: () => view, render: () => ({ promise: Promise.resolve(), cancel: vi.fn() }) };
    const pdf = { getPage: () => Promise.resolve(page) } as unknown as PDFDocumentProxy;
    const canvas = { current: document.createElement("canvas") };
    const container = { current: document.createElement("div") };
    const { result } = renderHook(() => usePageCanvas(pdf, 1, canvas, container));
    await waitFor(() => { expect(result.current.rendering).toBe(false); });
    expect(canvas.current.width * canvas.current.height).toBeLessThan(2_020_000);
  });
});
