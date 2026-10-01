import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { afterEach, describe, expect, it, vi } from "vitest";

import { listenHere } from "./events";

/** The shape Tauri's listen takes, so the registered handler can be called back. */
type Listen = (event: string, handler: (received: { payload: unknown }) => void, options?: { target: string }) => Promise<() => void>;

const tauri = vi.hoisted(() => ({
  listen: vi.fn<Listen>(() => Promise.resolve(() => undefined)),
}));

vi.mock("@tauri-apps/api/event", () => ({ listen: tauri.listen }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "approve" }) }));

afterEach(() => {
  vi.clearAllMocks();
});

/** Every source file under `dir`, tests left out. */
function sources(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      return sources(path);
    }
    return /\.tsx?$/u.test(entry.name) && !/\.test\.tsx?$/u.test(entry.name) ? [path] : [];
  });
}

describe("listenHere", () => {
  it("subscribes to events sent to this window only", async () => {
    const handler = vi.fn();
    await listenHere<number>("some-event", handler);
    expect(tauri.listen).toHaveBeenCalledWith("some-event", expect.any(Function), { target: "approve" });
  });

  it("hands the handler the payload", async () => {
    const handler = vi.fn();
    await listenHere<number>("some-event", handler);
    tauri.listen.mock.calls[0]?.[1]({ payload: 7 });
    expect(handler).toHaveBeenCalledWith(7);
  });

  it("is the only place the app listens for events", () => {
    const direct = sources(join(import.meta.dirname, ".."))
      .filter((path) => !path.endsWith("/api/events.ts"))
      .filter((path) => readFileSync(path, "utf8").includes("@tauri-apps/api/event"));
    expect(direct).toEqual([]);
  });
});
