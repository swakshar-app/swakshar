//! One request at a time: from the portal, through the approval window, back.

use std::time::Duration;

use swakshar_protocol::{
    REPLY_CANCELED, REPLY_FAILED, RequestKind, SUPPORTED_SIGNTYPE, SignRequest, mask_pan,
};
use swakshar_server::PortalRequest;
use swakshar_token::{Candidate, CertSummary, Inventory, candidates, criteria_for};
use tauri::{AppHandle, Emitter as _, Manager as _};
use tokio::sync::oneshot;
use tokio::time::timeout;

use crate::activity::ActivityEntry;
use crate::state::{AppState, lock, mask_serial, unix_now};
use crate::views::{FinishedView, PendingView, pending_view};
use crate::windows::{self, APPROVE};

/// How long the user has to decide.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// How long a success message stays before the window hides.
const SUCCESS_LINGER: Duration = Duration::from_millis(1_400);
/// Event carrying a new request to the approval window.
pub(crate) const EVENT_SIGN_REQUEST: &str = "sign-request";
/// Event telling the approval window a request ended.
pub(crate) const EVENT_SIGN_FINISHED: &str = "sign-finished";

/// How a request ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Signed and answered.
    Signed,
    /// The user cancelled.
    Canceled,
    /// Nobody answered in time.
    TimedOut,
    /// The page left first.
    Abandoned,
}

impl Outcome {
    /// Name used in events and history.
    fn as_str(self) -> &'static str {
        match self {
            Self::Signed => "signed",
            Self::Canceled => "canceled",
            Self::TimedOut => "timed-out",
            Self::Abandoned => "abandoned",
        }
    }
}

/// The request waiting for approval.
pub(crate) struct Pending {
    /// Request id.
    pub(crate) id: u64,
    /// What the approval window shows.
    pub(crate) view: PendingView,
    /// The parsed request.
    pub(crate) request: SignRequest,
    /// Page origin.
    origin: String,
    /// Token snapshot the candidates point into.
    pub(crate) inventory: Inventory,
    /// Eligible certificates, best first.
    pub(crate) candidates: Vec<Candidate>,
    /// Where the final reply goes.
    reply: Option<oneshot::Sender<String>>,
}

/// Drops a pending request that is still open when its future is dropped,
/// for example because the page closed the connection.
struct PendingGuard {
    /// App handle.
    app: AppHandle,
    /// Request id.
    id: u64,
}

impl Drop for PendingGuard {
    /// Abandons the request if nothing else finished it.
    fn drop(&mut self) {
        finish(&self.app, self.id, Outcome::Abandoned, None, None);
    }
}

/// Handles one portal request end to end and returns the reply frame.
pub(crate) async fn run(app: &AppHandle, portal: PortalRequest) -> String {
    let state = app.state::<AppState>();
    if portal.request.signtype != SUPPORTED_SIGNTYPE || lock(&state.pending).is_some() {
        log::warn!(
            "refusing a request (signtype {}, or another is waiting)",
            portal.request.signtype
        );
        state
            .activity
            .append(&entry(&portal.origin, &portal.request, None, "refused"));
        return REPLY_FAILED.to_owned();
    }
    let inventory = state
        .token
        .inventory(state.settings().module_paths())
        .await
        .unwrap_or_default();
    let now = unix_now();
    let found = candidates(&inventory.tokens, &criteria_for(&portal.request, now));
    let id = state.next_id();
    let expires_at = now + i64::try_from(APPROVAL_TIMEOUT.as_secs()).unwrap_or_default();
    let view = pending_view(id, &portal, &inventory, &found, expires_at);
    let (sender, receiver) = oneshot::channel();
    *lock(&state.pending) = Some(Pending {
        id,
        view: view.clone(),
        request: portal.request,
        origin: portal.origin,
        inventory,
        candidates: found,
        reply: Some(sender),
    });
    let _guard = PendingGuard {
        app: app.clone(),
        id,
    };
    crate::tray::refresh(app);
    if let Err(error) = app.emit_to(APPROVE, EVENT_SIGN_REQUEST, &view) {
        log::warn!("could not notify the approval window: {error}");
    }
    windows::show(app, APPROVE);
    match timeout(APPROVAL_TIMEOUT, receiver).await {
        Ok(Ok(reply)) => reply,
        Ok(Err(_)) => REPLY_CANCELED.to_owned(),
        Err(_) => {
            finish(app, id, Outcome::TimedOut, None, None);
            REPLY_CANCELED.to_owned()
        }
    }
}

/// Ends request `id`: sends the reply, records history, tells the window.
/// Returns false when the request had already ended.
pub(crate) fn finish(
    app: &AppHandle,
    id: u64,
    outcome: Outcome,
    reply: Option<String>,
    signer: Option<(&CertSummary, &str)>,
) -> bool {
    let state = app.state::<AppState>();
    let taken = {
        let mut slot = lock(&state.pending);
        if slot.as_ref().is_some_and(|pending| pending.id == id) {
            slot.take()
        } else {
            None
        }
    };
    let Some(mut pending) = taken else {
        return false;
    };
    if let (Some(reply), Some(sender)) = (reply, pending.reply.take()) {
        let _ = sender.send(reply);
    }
    let mut record = entry(&pending.origin, &pending.request, signer, outcome.as_str());
    record.at = unix_now();
    state.activity.append(&record);
    let finished = FinishedView {
        id,
        outcome: outcome.as_str(),
    };
    if let Err(error) = app.emit_to(APPROVE, EVENT_SIGN_FINISHED, &finished) {
        log::warn!("could not notify the approval window: {error}");
    }
    hide_later(
        app,
        if outcome == Outcome::Signed {
            SUCCESS_LINGER
        } else {
            Duration::ZERO
        },
    );
    crate::tray::refresh(app);
    true
}

/// Hides the approval window after `delay`, unless a new request arrived.
fn hide_later(app: &AppHandle, delay: Duration) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(delay).await;
        if lock(&app.state::<AppState>().pending).is_none() {
            windows::hide(&app, APPROVE);
        }
    });
}

/// A history entry for a request.
fn entry(
    origin: &str,
    request: &SignRequest,
    signer: Option<(&CertSummary, &str)>,
    outcome: &str,
) -> ActivityEntry {
    ActivityEntry {
        at: unix_now(),
        origin: origin.to_owned(),
        kind: match request.kind() {
            RequestKind::Registration => "registration".to_owned(),
            RequestKind::Document => "document".to_owned(),
        },
        pan_masked: request.pan.as_deref().map(mask_pan),
        holder: signer.map(|(summary, _)| summary.subject_cn.clone()),
        token: signer.map(|(_, serial)| mask_serial(serial)),
        outcome: outcome.to_owned(),
    }
}
