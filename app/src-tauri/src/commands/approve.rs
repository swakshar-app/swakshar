//! The approval window's three commands.

use std::time::Duration;

use serde::Serialize;
use swakshar_protocol::{REPLY_CANCELED, SignRequest};
use swakshar_token::{AuthPin, CertRef, SignJob, TokenError, portal_reply};
use tauri::{AppHandle, Manager as _, State};

use crate::error::CommandResult;
use crate::pending::{self, Outcome, Pending};
use crate::state::{AppState, lock, unix_now};
use crate::views::PendingView;

/// How an approval attempt ended.
#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "status",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum ApproveResult {
    /// Signed and sent to the portal.
    Signed {
        /// Certificate holder.
        holder: String,
    },
    /// Wrong PIN; the request stays open.
    PinIncorrect {
        /// A wrong PIN was entered before.
        count_low: bool,
        /// One more wrong PIN locks the token.
        final_try: bool,
        /// The PIN is now locked.
        locked: bool,
    },
    /// The token PIN is locked.
    PinLocked,
    /// The PIN has an invalid length or characters.
    PinInvalid,
    /// No PIN was entered.
    PinRequired,
    /// The request ended before the signature was ready.
    Expired,
    /// Anything else; the request stays open.
    Failed {
        /// What went wrong.
        message: String,
    },
}

/// The request waiting for approval, if any.
#[tauri::command]
pub(crate) fn get_pending_request(state: State<'_, AppState>) -> Option<PendingView> {
    lock(&state.pending)
        .as_ref()
        .map(|pending| pending.view.clone())
}

/// Signs request `id` with candidate `candidate`. The PIN becomes an
/// `AuthPin` immediately and is never stored or returned.
#[tauri::command]
pub(crate) async fn approve_request(
    app: AppHandle,
    id: u64,
    candidate: usize,
    pin: Option<String>,
) -> CommandResult<ApproveResult> {
    let now = unix_now();
    let prepared = {
        let state = app.state::<AppState>();
        let slot = lock(&state.pending);
        match slot.as_ref().filter(|pending| pending.id == id) {
            Some(pending) => prepare(pending, candidate, pin, now),
            None => Err(ApproveResult::Expired),
        }
    };
    let (job, request, serial) = match prepared {
        Ok(prepared) => prepared,
        Err(result) => return Ok(result),
    };
    let signed = app.state::<AppState>().token.sign(job).await;
    Ok(match signed {
        Ok(output) => {
            let reply = portal_reply(&output, &request, now);
            if pending::finish(
                &app,
                id,
                Outcome::Signed,
                Some(reply),
                Some((&output.summary, &serial)),
            ) {
                ApproveResult::Signed {
                    holder: output.summary.subject_cn,
                }
            } else {
                ApproveResult::Expired
            }
        }
        Err(TokenError::PinIncorrect(pin)) => ApproveResult::PinIncorrect {
            count_low: pin.count_low,
            final_try: pin.final_try,
            locked: pin.locked,
        },
        Err(TokenError::PinLocked) => ApproveResult::PinLocked,
        Err(TokenError::PinInvalid) => ApproveResult::PinInvalid,
        Err(TokenError::PinRequired) => ApproveResult::PinRequired,
        Err(error) => ApproveResult::Failed {
            message: error.to_string(),
        },
    })
}

/// Declines request `id`.
#[tauri::command]
pub(crate) fn cancel_request(app: AppHandle, id: u64) {
    pending::finish(
        &app,
        id,
        Outcome::Canceled,
        Some(REPLY_CANCELED.to_owned()),
        None,
    );
}

/// Builds the signing job for the chosen certificate.
fn prepare(
    pending: &Pending,
    candidate: usize,
    pin: Option<String>,
    now: i64,
) -> Result<(SignJob, SignRequest, String), ApproveResult> {
    let gone = || ApproveResult::Failed {
        message: "That certificate is no longer available.".to_owned(),
    };
    let chosen = pending.candidates.get(candidate).ok_or_else(gone)?;
    let token = pending
        .inventory
        .tokens
        .get(chosen.token)
        .ok_or_else(gone)?;
    let certificate = token
        .certificates
        .get(chosen.certificate)
        .ok_or_else(gone)?;
    let pin = if token.pin.protected_path {
        None
    } else {
        Some(AuthPin::from(
            pin.filter(|pin| !pin.is_empty())
                .ok_or(ApproveResult::PinRequired)?,
        ))
    };
    let job = SignJob {
        cert: CertRef {
            module: token.module.clone(),
            token_serial: token.serial.clone(),
            cert_id: certificate.id.clone(),
        },
        pin,
        content: pending.request.content.as_bytes().to_vec(),
        signing_time: Duration::from_secs(u64::try_from(now).unwrap_or_default()),
    };
    Ok((job, pending.request.clone(), token.serial.clone()))
}
