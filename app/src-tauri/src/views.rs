//! Serializable views of app state for the webview. Masked, never secret.

use serde::Serialize;
use swakshar_protocol::{RequestKind, format_date_utc, mask_pan};
use swakshar_server::PortalRequest;
use swakshar_token::{ArchSupport, Candidate, Inventory, PanMatch, TokenCertificate, TokenEntry};

use crate::state::mask_serial;

/// Certificates expiring within this many seconds are flagged.
const EXPIRY_WARNING_SECONDS: i64 = 30 * 86_400;

/// A certificate the user may sign with.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CandidateView {
    /// Position in the pending request's candidate list.
    pub(crate) index: usize,
    /// Certificate holder.
    pub(crate) holder: String,
    /// Issuing CA.
    pub(crate) issuer: String,
    /// `Class 3` and so on.
    pub(crate) class_label: String,
    /// `dd-MM-yyyy`.
    pub(crate) valid_until: String,
    /// `match`, `mismatch` or `unknown`.
    pub(crate) pan_match: &'static str,
    /// Token manufacturer and model.
    pub(crate) token_name: String,
    /// Masked token serial.
    pub(crate) token_serial: String,
    /// The token has its own PIN pad.
    pub(crate) pin_pad: bool,
    /// A wrong PIN was entered before.
    pub(crate) pin_count_low: bool,
    /// One more wrong PIN locks the token.
    pub(crate) pin_final_try: bool,
    /// The PIN is locked.
    pub(crate) pin_locked: bool,
}

/// A request waiting for approval.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingView {
    /// Request id.
    pub(crate) id: u64,
    /// Page origin.
    pub(crate) origin: String,
    /// `registration` or `document`.
    pub(crate) kind: &'static str,
    /// Masked PAN from the request.
    pub(crate) pan_masked: Option<String>,
    /// What is signed: masked PAN or the full document digest.
    pub(crate) content: String,
    /// Eligible certificates, best first.
    pub(crate) candidates: Vec<CandidateView>,
    /// When the request times out, Unix seconds.
    pub(crate) expires_at: i64,
}

/// Payload of the `sign-finished` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FinishedView {
    /// Request id.
    pub(crate) id: u64,
    /// How it ended.
    pub(crate) outcome: &'static str,
}

/// Drivers and tokens.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryView {
    /// Drivers that exist or were added.
    pub(crate) modules: Vec<ModuleView>,
    /// Connected tokens.
    pub(crate) tokens: Vec<TokenView>,
}

/// One driver.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleView {
    /// Token family.
    pub(crate) family: String,
    /// Driver path.
    pub(crate) path: String,
    /// The file exists.
    pub(crate) found: bool,
    /// Loaded and initialised.
    pub(crate) loaded: bool,
    /// Load error, if any.
    pub(crate) error: Option<String>,
    /// Built for the other processor.
    pub(crate) needs_rosetta: bool,
    /// Added in settings.
    pub(crate) user_added: bool,
}

/// One token.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenView {
    /// Manufacturer and model.
    pub(crate) name: String,
    /// Masked serial.
    pub(crate) serial: String,
    /// Own PIN pad.
    pub(crate) pin_pad: bool,
    /// PIN locked.
    pub(crate) pin_locked: bool,
    /// One PIN try left.
    pub(crate) pin_final_try: bool,
    /// Certificates on it.
    pub(crate) certificates: Vec<CertView>,
}

/// One certificate.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CertView {
    /// Holder.
    pub(crate) holder: String,
    /// Issuer.
    pub(crate) issuer: String,
    /// Class label.
    pub(crate) class_label: String,
    /// `dd-MM-yyyy`.
    pub(crate) valid_until: String,
    /// Inside its validity period now.
    pub(crate) valid: bool,
    /// Expires within 30 days.
    pub(crate) expires_soon: bool,
    /// Usable for signing.
    pub(crate) signing: bool,
    /// Carries a PAN hash.
    pub(crate) has_pan: bool,
}

/// The approval window's view of a request.
pub(crate) fn pending_view(
    id: u64,
    portal: &PortalRequest,
    inventory: &Inventory,
    candidates: &[Candidate],
    expires_at: i64,
) -> PendingView {
    let request = &portal.request;
    let (kind, content) = match request.kind() {
        RequestKind::Registration => ("registration", mask_pan(&request.content)),
        RequestKind::Document => ("document", request.content.clone()),
    };
    PendingView {
        id,
        origin: portal.origin.clone(),
        kind,
        pan_masked: request.pan.as_deref().map(mask_pan),
        content,
        candidates: candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| candidate_view(index, candidate, inventory))
            .collect(),
        expires_at,
    }
}

/// Drivers (found or added) and tokens.
pub(crate) fn inventory_view(inventory: &Inventory, now: i64) -> InventoryView {
    InventoryView {
        modules: inventory
            .modules
            .iter()
            .filter(|status| status.candidate.exists || status.candidate.user_added)
            .map(|status| ModuleView {
                family: status.candidate.family.clone(),
                path: status.candidate.path.display().to_string(),
                found: status.candidate.exists,
                loaded: status.loaded,
                error: status.error.clone(),
                needs_rosetta: status.candidate.arch == ArchSupport::Incompatible,
                user_added: status.candidate.user_added,
            })
            .collect(),
        tokens: inventory
            .tokens
            .iter()
            .map(|token| token_view(token, now))
            .collect(),
    }
}

/// One token and its certificates.
fn token_view(token: &TokenEntry, now: i64) -> TokenView {
    TokenView {
        name: token_name(token),
        serial: mask_serial(&token.serial),
        pin_pad: token.pin.protected_path,
        pin_locked: token.pin.locked,
        pin_final_try: token.pin.final_try,
        certificates: token
            .certificates
            .iter()
            .map(|certificate| cert_view(certificate, now))
            .collect(),
    }
}

/// One certificate.
fn cert_view(certificate: &TokenCertificate, now: i64) -> CertView {
    let summary = &certificate.summary;
    CertView {
        holder: summary.subject_cn.clone(),
        issuer: summary.issuer_cn.clone(),
        class_label: summary.class_label().to_owned(),
        valid_until: format_date_utc(summary.not_after),
        valid: summary.valid_at(now),
        expires_soon: summary.valid_at(now) && summary.not_after - now < EXPIRY_WARNING_SECONDS,
        signing: summary.signing,
        has_pan: summary.pan_hash.is_some(),
    }
}

/// A candidate, when its token and certificate still exist.
fn candidate_view(
    index: usize,
    candidate: &Candidate,
    inventory: &Inventory,
) -> Option<CandidateView> {
    let token = inventory.tokens.get(candidate.token)?;
    let summary = &token.certificates.get(candidate.certificate)?.summary;
    Some(CandidateView {
        index,
        holder: summary.subject_cn.clone(),
        issuer: summary.issuer_cn.clone(),
        class_label: summary.class_label().to_owned(),
        valid_until: format_date_utc(summary.not_after),
        pan_match: match candidate.pan_match {
            PanMatch::Match => "match",
            PanMatch::Mismatch => "mismatch",
            PanMatch::Unknown => "unknown",
        },
        token_name: token_name(token),
        token_serial: mask_serial(&token.serial),
        pin_pad: token.pin.protected_path,
        pin_count_low: token.pin.count_low,
        pin_final_try: token.pin.final_try,
        pin_locked: token.pin.locked,
    })
}

/// Manufacturer and model, or the label when those are empty.
fn token_name(token: &TokenEntry) -> String {
    let name = format!("{} {}", token.manufacturer, token.model)
        .trim()
        .to_owned();
    if name.is_empty() {
        token.label.clone()
    } else {
        name
    }
}
