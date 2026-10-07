/**
 * Typed wrappers over the backend commands. Each window's capability allows
 * only its own subset; calling another window's command is refused.
 */
import { invoke } from "@tauri-apps/api/core";

import { listenHere, type UnlistenFn } from "./events";

import type {
  ActivityEntry,
  ApproveResult,
  DoctorCheck,
  FinishedView,
  InventoryView,
  Overview,
  PendingView,
  Settings,
  UpdateView,
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
  /** Opens a file picker and adds the chosen driver; `null` when cancelled. */
  addDriver: (): Promise<Settings | null> => invoke<Settings | null>("add_driver"),
  /** Removes a driver the user added. */
  removeDriver: (path: string): Promise<Settings> => invoke<Settings>("remove_driver", { path }),
  /** Opens the token maker's official driver download page in the browser. */
  openDriverPage: (vendorId: number): Promise<void> => invoke<void>("open_driver_page", { vendorId }),
  /** Trusts the local certificate; macOS shows its own prompt. */
  installTrust: (): Promise<Overview> => invoke<Overview>("install_trust"),
  /** Removes the local certificate's trust. */
  removeTrust: (): Promise<Overview> => invoke<Overview>("remove_trust"),
  /** Checks for an update now. */
  checkForUpdate: (): Promise<UpdateView> => invoke<UpdateView>("check_for_update"),
  /** Starts downloading the offered update. */
  downloadUpdate: (): Promise<UpdateView> => invoke<UpdateView>("download_update"),
  /** Restarts into the downloaded update now, or when Swakshar is idle. */
  restartToUpdate: (whenIdle: boolean): Promise<UpdateView> => invoke<UpdateView>("restart_to_update", { whenIdle }),
  /** A diagnostic report for bug reports; never sent anywhere. */
  diagnosticReport: (): Promise<string> => invoke<string>("diagnostic_report"),
  /** Opens a new GitHub issue in the browser. */
  openIssuePage: (): Promise<void> => invoke<void>("open_issue_page"),
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
  /** Re-reads the tokens for the request, after plugging one in. */
  refresh: (id: number): Promise<PendingView | null> => invoke<PendingView | null>("refresh_request", { id }),
  /** Declines the request. */
  cancel: (id: number): Promise<void> => invoke<void>("cancel_request", { id }),
  /** Subscribes to new requests. */
  onRequest: (handler: (view: PendingView) => void): Promise<UnlistenFn> => listenHere<PendingView>(EVENT_SIGN_REQUEST, handler),
  /** Subscribes to finished requests. */
  onFinished: (handler: (view: FinishedView) => void): Promise<UnlistenFn> => listenHere<FinishedView>(EVENT_SIGN_FINISHED, handler),
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
