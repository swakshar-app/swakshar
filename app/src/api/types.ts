/**
 * Shapes the Rust backend sends, field for field (serde camelCase).
 * `null` stands for Rust's `None`.
 */

/** Signer state. */
export interface ServerView {
  readonly state: "starting" | "running" | "paused" | "failed";
  readonly port: number | null;
  readonly error: string | null;
}

/** Local certificate trust. */
export interface TrustView {
  readonly status: "trusted" | "not-trusted" | "unsupported" | "missing";
  readonly validUntil: string | null;
  readonly command: string | null;
}

/** Something that happened, and when (Unix seconds). */
export interface EventView {
  readonly detail: string;
  readonly at: number;
}

/** Home view status. */
export interface Overview {
  readonly server: ServerView;
  readonly trust: TrustView;
  readonly onboardingComplete: boolean;
  readonly waiting: boolean;
  readonly lastConnection: EventView | null;
  readonly lastTlsFailure: EventView | null;
  readonly statusPageSeen: boolean;
  readonly greetingVersion: string;
  readonly appVersion: string;
}

/** One token driver. */
export interface ModuleView {
  readonly family: string;
  readonly path: string;
  readonly found: boolean;
  readonly loaded: boolean;
  readonly error: string | null;
  readonly needsRosetta: boolean;
  readonly userAdded: boolean;
}

/** One certificate on a token. */
export interface CertView {
  readonly holder: string;
  readonly issuer: string;
  readonly classLabel: string;
  readonly validUntil: string;
  readonly valid: boolean;
  readonly expiresSoon: boolean;
  readonly signing: boolean;
  readonly hasPan: boolean;
}

/** One token. */
export interface TokenView {
  readonly name: string;
  readonly serial: string;
  readonly pinPad: boolean;
  readonly pinLocked: boolean;
  readonly pinFinalTry: boolean;
  readonly certificates: readonly CertView[];
}

/** A token seen on the USB bus that no driver could read. */
export interface DetectedView {
  readonly family: string;
  readonly name: string;
  readonly driverUrl: string | null;
  readonly driverPresent: boolean;
}

/** Drivers and tokens. */
export interface InventoryView {
  readonly modules: readonly ModuleView[];
  readonly tokens: readonly TokenView[];
  readonly detected: readonly DetectedView[];
}

/** A certificate the user may sign with. */
export interface CandidateView {
  readonly index: number;
  readonly holder: string;
  readonly issuer: string;
  readonly classLabel: string;
  readonly validUntil: string;
  readonly panMatch: "match" | "mismatch" | "unknown";
  readonly tokenName: string;
  readonly tokenSerial: string;
  readonly pinPad: boolean;
  readonly pinCountLow: boolean;
  readonly pinFinalTry: boolean;
  readonly pinLocked: boolean;
}

/** A request waiting for approval. */
export interface PendingView {
  readonly id: number;
  readonly origin: string;
  readonly kind: "registration" | "document";
  readonly panMasked: string | null;
  readonly content: string;
  readonly candidates: readonly CandidateView[];
  readonly expiresAt: number;
}

/** A request ended. */
export interface FinishedView {
  readonly id: number;
  readonly outcome: "signed" | "canceled" | "timed-out" | "abandoned";
}

/** Result of pressing Sign. */
export type ApproveResult =
  | { readonly status: "signed"; readonly holder: string }
  | { readonly status: "pin-incorrect"; readonly countLow: boolean; readonly finalTry: boolean; readonly locked: boolean }
  | { readonly status: "pin-locked" }
  | { readonly status: "pin-invalid" }
  | { readonly status: "pin-required" }
  | { readonly status: "expired" }
  | { readonly status: "failed"; readonly message: string };

/** User settings. */
export interface Settings {
  readonly modules: readonly string[];
  readonly preferredPort: number | null;
  readonly greetingVersion: string;
  readonly extraOrigins: readonly string[];
  readonly startAtLogin: boolean;
  readonly onboardingComplete: boolean;
}

/** One finished request. */
export interface ActivityEntry {
  readonly at: number;
  readonly origin: string;
  readonly kind: string;
  readonly panMasked: string | null;
  readonly holder: string | null;
  readonly token: string | null;
  readonly outcome: string;
}

/** One diagnostic check. */
export interface DoctorCheck {
  readonly label: string;
  readonly status: "pass" | "warn" | "fail";
  readonly detail: string;
}
