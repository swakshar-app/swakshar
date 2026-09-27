/**
 * Test data shaped like the backend's views. Imported only by tests.
 */
import type { Overview, UpdateView } from "../api/types";

/** An update view with nothing on offer, overridable per test. */
export function updateView(fields: Partial<UpdateView> = {}): UpdateView {
  return { state: "idle", version: null, notes: null, progress: null, error: null, lastChecked: null, restartWhenIdle: false, ...fields };
}

/** A running, trusted, idle overview, overridable per test. */
export function overview(fields: Partial<Overview> = {}): Overview {
  return {
    server: { state: "running", port: 1585, error: null },
    trust: { status: "trusted", validUntil: "01-01-2028", command: null },
    onboardingComplete: true,
    waiting: false,
    lastConnection: null,
    lastTlsFailure: null,
    statusPageSeen: false,
    greetingVersion: "2.8",
    appVersion: "0.1.0",
    update: updateView(),
    ...fields,
  };
}
