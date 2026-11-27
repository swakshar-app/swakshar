import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { usePdf } from "./usePdf";

const mocks = vi.hoisted(() => ({ bytes: vi.fn(), parse: vi.fn() }));
vi.mock("../api/signing", () => ({ signingApi: { pdfBytes: mocks.bytes } }));
vi.mock("pdfjs-dist/legacy/build/pdf.mjs", () => ({ getDocument: mocks.parse, GlobalWorkerOptions: {} }));

afterEach(() => { cleanup(); vi.resetAllMocks(); });

describe("binary PDF loading lifecycle", () => {
  it("passes binary bytes directly to the worker and releases it on unmount", async () => {
    const bytes = new Uint8Array([37, 80, 68, 70]).buffer;
    const document = { numPages: 1 };
    const destroy = vi.fn(() => Promise.resolve());
    mocks.bytes.mockResolvedValue(bytes);
    mocks.parse.mockReturnValue({ promise: Promise.resolve(document), destroy });
    const { result, unmount } = renderHook(() => usePdf({ id: 1, name: "example.pdf", size: 4 }));
    await waitFor(() => { expect(result.current.pdf).toBe(document); });
    const options = mocks.parse.mock.calls[0]?.[0] as { data: Uint8Array; enableXfa: boolean };
    expect(options.data.buffer).toBe(bytes);
    expect(options.enableXfa).toBe(false);
    unmount();
    expect(destroy).toHaveBeenCalledTimes(1);
  });

  it("never starts an obsolete parser after the editor closes during transfer", async () => {
    let finish: ((bytes: ArrayBuffer) => void) | undefined;
    mocks.bytes.mockReturnValue(new Promise<ArrayBuffer>((resolve) => { finish = resolve; }));
    const { unmount } = renderHook(() => usePdf({ id: 1, name: "example.pdf", size: 4 }));
    unmount();
    await act(async () => { finish?.(new ArrayBuffer(4)); });
    expect(mocks.parse).not.toHaveBeenCalled();
  });

  it("reports transfer errors instead of leaving a permanent loading state", async () => {
    mocks.bytes.mockRejectedValue(new Error("Selection expired"));
    const { result } = renderHook(() => usePdf({ id: 1, name: "example.pdf", size: 4 }));
    await waitFor(() => { expect(result.current.error).toContain("Selection expired"); });
    expect(mocks.parse).not.toHaveBeenCalled();
  });
});
