import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { Settings } from "../api/types";
import { overview, settings } from "../testing/fixtures";
import { SettingsView } from "./SettingsView";

const api = vi.hoisted(() => ({
  settings: vi.fn(),
  saveSettings: vi.fn((value: Settings) => Promise.resolve(value)),
  overview: vi.fn(),
}));
const platform = vi.hoisted(() => ({ onMac: vi.fn(() => true) }));

vi.mock("../api/commands", () => ({ mainApi: api, errorText: (reason: unknown) => String(reason) }));
vi.mock("../components/platform", () => platform);

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

/** Renders the view once the settings have loaded. */
async function renderLoaded(): Promise<void> {
  api.settings.mockResolvedValue(settings());
  api.overview.mockResolvedValue(overview());
  render(<SettingsView />);
  await screen.findByText("General");
}

describe("SettingsView", () => {
  it("saves the choice to keep Swakshar in the Dock on a Mac", async () => {
    platform.onMac.mockReturnValue(true);
    await renderLoaded();
    const keep = screen.getByRole("checkbox", { name: "Keep Swakshar in the Dock when its windows are closed" });
    expect((keep as HTMLInputElement).checked).toBe(false);
    fireEvent.click(keep);
    fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
    await waitFor(() => expect(api.saveSettings).toHaveBeenCalledOnce());
    expect(api.saveSettings.mock.calls[0]?.[0].keepInDock).toBe(true);
  });

  it("leaves the Dock choice out on other systems", async () => {
    platform.onMac.mockReturnValue(false);
    await renderLoaded();
    expect(screen.queryByRole("checkbox", { name: /Dock/ })).toBeNull();
  });
});
