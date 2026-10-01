import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { overview } from "../testing/fixtures";
import { ActivityView } from "./ActivityView";
import { HelpView } from "./HelpView";
import { UpdatesCard } from "./UpdatesCard";

/** A call that never answers, like a view whose data has not arrived. */
function never<T>(): Promise<T> {
  return new Promise<T>(() => undefined);
}

const api = vi.hoisted(() => ({
  activity: vi.fn(),
  doctor: vi.fn(),
  overview: vi.fn(),
}));

vi.mock("../api/commands", () => ({ mainApi: api, errorText: (reason: unknown) => String(reason) }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("views before their data arrives", () => {
  it("Activity says it is loading rather than that nothing was signed", () => {
    api.activity.mockReturnValue(never());
    render(<ActivityView />);
    expect(screen.getByText("Loading.")).toBeTruthy();
    expect(screen.queryByText("No requests yet.")).toBeNull();
  });

  it("Activity says nothing was signed once an empty history arrives", async () => {
    api.activity.mockResolvedValue([]);
    render(<ActivityView />);
    expect(await screen.findByText("No requests yet.")).toBeTruthy();
  });

  it("Updates says it is loading instead of an empty version", () => {
    api.overview.mockReturnValue(never());
    render(<UpdatesCard />);
    expect(screen.getByText("Loading.")).toBeTruthy();
    expect(screen.queryByText(/^Version/u)).toBeNull();
  });

  it("Updates shows the version once it arrives", async () => {
    api.overview.mockResolvedValue(overview({ appVersion: "0.1.5" }));
    render(<UpdatesCard />);
    expect(await screen.findByText(/^Version 0\.1\.5\./u)).toBeTruthy();
  });

  it("the Help checklist says it is loading instead of showing nothing", () => {
    api.doctor.mockReturnValue(never());
    api.overview.mockReturnValue(never());
    render(<HelpView />);
    expect(screen.getAllByText("Loading.").length).toBeGreaterThanOrEqual(2);
  });
});
