//! Bundled PDF engine availability and fresh, masked certificate choices.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Manager as _};

use crate::error::{CommandError, CommandResult};
use crate::pdf_engine;
use crate::state::{AppState, mask_serial, unix_now};

/// A usable certificate from a currently connected token.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfCertificate {
    /// Opaque certificate and token selector; the token serial stays masked.
    fingerprint: String,
    /// Certificate holder.
    holder: String,
    /// Token label with masked serial.
    token: String,
    /// Token owns PIN entry.
    protected_path: bool,
    /// One wrong PIN locks the token.
    final_try: bool,
    /// The token PIN is locked.
    locked: bool,
}

/// Bundled engine and available DSC choices.
#[derive(Serialize)]
pub(crate) struct PdfStatus {
    /// A release must carry this executable and its complete runtime.
    available: bool,
    /// Eligible certificates, including locked ones for an explanatory message.
    certificates: Vec<PdfCertificate>,
}

/// Reads fresh token flags; an unsuccessful attempt never retries automatically.
#[tauri::command]
pub(crate) async fn get_pdf_status(app: AppHandle) -> CommandResult<PdfStatus> {
    let state = app.state::<AppState>();
    if state.busy.load(Ordering::Acquire) {
        return Err(CommandError::Message(
            "Finish the current signing request before refreshing certificates.".to_owned(),
        ));
    }
    let inventory = state
        .token
        .inventory(state.settings().module_paths())
        .await?;
    let now = unix_now();
    let certificates = inventory
        .tokens
        .iter()
        .flat_map(|token| {
            token
                .certificates
                .iter()
                .filter(|certificate| {
                    certificate.summary.signing
                        && certificate.summary.valid_at(now)
                        && !certificate.id.is_empty()
                })
                .map(|certificate| PdfCertificate {
                    fingerprint: pdf_engine::certificate_id(token, certificate),
                    holder: certificate.summary.subject_cn.clone(),
                    token: format!("{} ({})", token.label, mask_serial(&token.serial)),
                    protected_path: token.pin.protected_path,
                    final_try: token.pin.final_try,
                    locked: token.pin.locked,
                })
        })
        .collect();
    Ok(PdfStatus {
        available: pdf_engine::engine_path(&app)?.is_file(),
        certificates,
    })
}
