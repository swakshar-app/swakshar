import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { updateView } from "../testing/fixtures";
import { UpdateToast } from "./UpdateToast";

const api = vi.hoisted(() => ({
  downloadUpdate: vi.fn(() => Promise.resolve()),
  restartToUpdate: vi.fn((_whenIdle: boolean) => Promise.resolve()),
}));

vi.mock("../api/commands", () => ({ mainApi: api, errorText: (reason: unknown) => String(reason) }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("UpdateToast", () => {
  it("stays hidden when nothing is on offer", () => {
    for (const state of ["idle", "unavailable", "checking"] as const) {
      const { container } = render(<UpdateToast update={updateView({ state })} onChange={() => undefined} />);
      expect(container.innerHTML).toBe("");
      cleanup();
    }
  });

  it("offers a download, with the changes on request", async () => {
    const onChange = vi.fn();
    render(<UpdateToast update={updateView({ state: "available", version: "0.1.1", notes: "Faster start." })} onChange={onChange} />);
    expect(screen.getByText("Swakshar 0.1.1 is available")).toBeTruthy();
    expect(screen.queryByText("Faster start.")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "See changes" }));
    expect(screen.getByText("Faster start.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Download" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    expect(api.downloadUpdate).toHaveBeenCalledOnce();
  });

  it("shows wrapped changelog bullets as whole list items", () => {
    const notes = "- Restart opens Swakshar again.\n  Before, it stayed closed.\n- Faster updates.";
    render(<UpdateToast update={updateView({ state: "available", version: "0.1.3", notes })} onChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: "See changes" }));
    const items = screen.getAllByRole("listitem").map((item) => item.textContent);
    expect(items).toEqual(["Restart opens Swakshar again. Before, it stayed closed.", "Faster updates."]);
  });

  it("shows download progress", () => {
    render(<UpdateToast update={updateView({ state: "downloading", version: "0.1.1", progress: 40 })} onChange={() => undefined} />);
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("40");
  });

  it("restarts now or when idle once downloaded", async () => {
    render(<UpdateToast update={updateView({ state: "ready", version: "0.1.1" })} onChange={() => undefined} />);
    expect(screen.getByText("Swakshar 0.1.1 is ready")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Restart when idle" }));
    await waitFor(() => expect(api.restartToUpdate).toHaveBeenCalledWith(true));
    fireEvent.click(screen.getByRole("button", { name: "Restart" }));
    await waitFor(() => expect(api.restartToUpdate).toHaveBeenCalledWith(false));
  });

  it("says when a restart is already waiting for an idle moment", () => {
    render(<UpdateToast update={updateView({ state: "ready", version: "0.1.1", restartWhenIdle: true })} onChange={() => undefined} />);
    const idle = screen.getByRole("button", { name: "Restart when idle" });
    expect(idle.hasAttribute("disabled")).toBe(true);
    expect(screen.getByText(/restarts as soon as no signing is in progress/)).toBeTruthy();
  });

  it("offers a retry after a failed download", () => {
    render(<UpdateToast update={updateView({ state: "failed", version: "0.1.1", error: "Signature check failed." })} onChange={() => undefined} />);
    expect(screen.getByText("Signature check failed.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy();
  });

  it("can be dismissed until the update moves on", () => {
    const { rerender, container } = render(<UpdateToast update={updateView({ state: "available", version: "0.1.1" })} onChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(container.innerHTML).toBe("");
    rerender(<UpdateToast update={updateView({ state: "ready", version: "0.1.1" })} onChange={() => undefined} />);
    expect(screen.getByText("Swakshar 0.1.1 is ready")).toBeTruthy();
  });
});
