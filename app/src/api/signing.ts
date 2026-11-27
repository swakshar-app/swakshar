/** Main-window commands for portal companions and local PDF signing. */
import { invoke } from "@tauri-apps/api/core";

/** The loopback service, TLS and hosts entry are reported independently. */
export interface BridgeStatus {
  readonly installerAvailable: boolean;
  readonly supported: boolean;
  readonly installed: boolean;
  readonly loaded: boolean;
  readonly listening: boolean;
  readonly tlsReady: boolean;
  readonly hostsReady: boolean;
}

/** A native-picker-selected PDF snapshot. */
export interface PdfDocument {
  readonly id: number;
  readonly name: string;
  readonly size: number;
}

/** One certificate backed by a connected token. */
export interface PdfCertificate {
  readonly fingerprint: string;
  readonly holder: string;
  readonly token: string;
  readonly protectedPath: boolean;
  readonly finalTry: boolean;
  readonly locked: boolean;
}

/** Bundled engine availability and current token choices. */
export interface PdfStatus {
  readonly available: boolean;
  readonly certificates: readonly PdfCertificate[];
}

/** PDF-space rectangle: left, bottom, right, top. */
export type PdfBox = readonly [number, number, number, number];

/** One explicit signature requested from the main window. */
export interface PdfSignRequest {
  readonly finalTryAcknowledged: boolean;
  readonly documentId: number;
  readonly fingerprint: string;
  readonly placement: { readonly page: number; readonly rect: PdfBox };
}

/** Fixed commands; file paths are chosen in Rust through native dialogs. */
export const signingApi = {
  /** Re-reads the vendor service and certificate trust. */
  bridgeStatus: (): Promise<BridgeStatus> => invoke<BridgeStatus>("get_embridge_status"),
  /** Runs a fixed privileged service action. */
  bridgeControl: (action: "start" | "stop"): Promise<BridgeStatus> => invoke<BridgeStatus>("control_embridge", { action }),
  /** Opens the vendor's official download page. */
  bridgeDownload: (): Promise<void> => invoke<void>("open_embridge_download"),
  /** Opens the offline vendor installer packaged with the app. */
  bridgeInstall: (): Promise<void> => invoke<void>("open_embridge_installer"),
  /** Reads the bundled engine and available token certificates. */
  pdfStatus: (): Promise<PdfStatus> => invoke<PdfStatus>("get_pdf_status"),
  /** Opens a native PDF chooser and returns its selected snapshot. */
  choosePdf: (): Promise<PdfDocument | null> => invoke<PdfDocument | null>("choose_pdf"),
  /** Transfers the immutable selection directly as binary without a filesystem path. */
  pdfBytes: (documentId: number): Promise<ArrayBuffer> => invoke<ArrayBuffer>("get_pdf_bytes", { documentId }),
  /** Signs once, after a native Save dialog. No automatic retry. */
  signPdf: (request: PdfSignRequest, pin: string | null): Promise<string | null> => invoke<string | null>("sign_pdf", { request, pin }),
  /** Cancels the active PDF subprocess. */
  cancelPdf: (documentId: number): Promise<void> => invoke<void>("cancel_pdf_signing", { documentId }),
};
