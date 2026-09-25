/**
 * Typed wrappers over the backend commands. Each window's capability allows
 * only its own subset; calling another window's command is refused.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ActivityEntry,
  ApproveResult,
  DoctorCheck,
  FinishedView,
  InventoryView,
  Overview,
  PendingView,
  Settings,
} from "./types";

/** Event carrying a new request to the approval window. */
const EVENT_SIGN_REQUEST = "sign-request";
/** Event telling the approval window a request ended. */
const EVENT_SIGN_FINISHED = "sign-finished";

/** Main window commands. */
export const mainApi = {
  /** Home view status. */
  overview: (): Promise<Overview> => invoke<Overview>("get_overview"),
  /** Drivers, tokens and certificates. */
  tokens: (): Promise<InventoryView> => invoke<InventoryView>("list_tokens"),
  /** Opens the status page in the default browser. */
  openStatusPage: (): Promise<void> => invoke<void>("open_status_page"),
  /** Pauses or resumes signing. */
  setPaused: (paused: boolean): Promise<Overview> => invoke<Overview>("set_paused", { paused }),
  /** Finishes guided setup. */
  completeOnboarding: (): Promise<Overview> => invoke<Overview>("complete_onboarding"),
  /** Diagnostic checklist. */
  doctor: (): Promise<DoctorCheck[]> => invoke<DoctorCheck[]>("run_doctor"),
  /** Current settings. */
  settings: (): Promise<Settings> => invoke<Settings>("get_settings"),
  /** Validates, saves and applies settings. */
  saveSettings: (settings: Settings): Promise<Settings> => invoke<Settings>("save_settings", { settings }),
  /** Trusts the local certificate; macOS shows its own prompt. */
  installTrust: (): Promise<Overview> => invoke<Overview>("install_trust"),
  /** Removes the local certificate's trust. */
  removeTrust: (): Promise<Overview> => invoke<Overview>("remove_trust"),
  /** Recent requests, newest first. */
  activity: (): Promise<ActivityEntry[]> => invoke<ActivityEntry[]>("get_activity"),
  /** Deletes the history. */
  clearActivity: (): Promise<void> => invoke<void>("clear_activity"),
};

/** Approval window commands. */
export const approveApi = {
  /** The waiting request, if any. */
  pending: (): Promise<PendingView | null> => invoke<PendingView | null>("get_pending_request"),
  /** Signs with the chosen certificate. The PIN is not kept anywhere here. */
  approve: (id: number, candidate: number, pin: string | null): Promise<ApproveResult> =>
    invoke<ApproveResult>("approve_request", { id, candidate, pin }),
  /** Declines the request. */
  cancel: (id: number): Promise<void> => invoke<void>("cancel_request", { id }),
  /** Subscribes to new requests. */
  onRequest: (handler: (view: PendingView) => void): Promise<UnlistenFn> =>
    listen<PendingView>(EVENT_SIGN_REQUEST, (event) => handler(event.payload)),
  /** Subscribes to finished requests. */
  onFinished: (handler: (view: FinishedView) => void): Promise<UnlistenFn> =>
    listen<FinishedView>(EVENT_SIGN_FINISHED, (event) => handler(event.payload)),
};

/** Turns a rejected command into a sentence for the UI. */
export function errorText(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Something went wrong.";
}
