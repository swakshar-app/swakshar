//! The Help view's checklist: every link in the chain, pass or not.

use serde::Serialize;
use tauri::{AppHandle, Manager as _};

use crate::commands::overview::{OverviewView, overview};
use crate::error::CommandResult;
use crate::state::{AppState, unix_now};
use crate::views::{InventoryView, inventory_view};

/// One check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DoctorCheck {
    /// What is checked.
    pub(crate) label: &'static str,
    /// `pass`, `warn` or `fail`.
    pub(crate) status: &'static str,
    /// What was found, or what to do.
    pub(crate) detail: String,
}

/// Runs every check.
#[tauri::command]
pub(crate) async fn run_doctor(app: AppHandle) -> CommandResult<Vec<DoctorCheck>> {
    let status = overview(&app).await?;
    let state = app.state::<AppState>();
    let inventory = state
        .token
        .inventory(state.settings().module_paths())
        .await?;
    let tokens = inventory_view(&inventory, unix_now());
    Ok(vec![
        driver_check(&tokens),
        token_check(&tokens),
        certificate_check(&tokens),
        trust_check(&status),
        signer_check(&status),
        portal_check(&status),
    ])
}

/// Builds a check.
fn check(
    label: &'static str,
    pass: bool,
    fail_status: &'static str,
    detail: String,
) -> DoctorCheck {
    DoctorCheck {
        label,
        status: if pass { "pass" } else { fail_status },
        detail,
    }
}

/// A driver loaded.
fn driver_check(tokens: &InventoryView) -> DoctorCheck {
    let loaded: Vec<&str> = tokens
        .modules
        .iter()
        .filter(|module| module.loaded)
        .map(|module| module.family.as_str())
        .collect();
    let detail = if loaded.is_empty() {
        "No token driver found. Install your token's macOS driver, or add its path in Settings."
            .to_owned()
    } else {
        format!("Loaded: {}.", loaded.join(", "))
    };
    check("Token driver", !loaded.is_empty(), "fail", detail)
}

/// A token is connected.
fn token_check(tokens: &InventoryView) -> DoctorCheck {
    let count = tokens.tokens.len();
    check(
        "Token connected",
        count > 0,
        "fail",
        if count > 0 {
            format!("{count} connected.")
        } else {
            "Plug in your DSC token.".to_owned()
        },
    )
}

/// A valid signing certificate exists.
fn certificate_check(tokens: &InventoryView) -> DoctorCheck {
    let usable = tokens
        .tokens
        .iter()
        .flat_map(|token| &token.certificates)
        .filter(|cert| cert.valid && cert.signing);
    let soon = usable.clone().any(|cert| cert.expires_soon);
    let count = usable.count();
    let detail = match (count, soon) {
        (0, _) => "No valid signing certificate on the connected tokens.".to_owned(),
        (_, true) => "A certificate expires within 30 days. Renew it with your CA.".to_owned(),
        (count, false) => format!("{count} valid signing certificate(s)."),
    };
    check(
        "Signing certificate",
        count > 0 && !soon,
        if count > 0 { "warn" } else { "fail" },
        detail,
    )
}

/// The local certificate is trusted.
fn trust_check(status: &OverviewView) -> DoctorCheck {
    let detail = match status.trust.status {
        "trusted" => "This Mac trusts Swakshar's local certificate.".to_owned(),
        "missing" => "The local certificate has not been created yet.".to_owned(),
        "unsupported" => "Install the local certificate manually on this system.".to_owned(),
        _ => "Not trusted yet. Use Install certificate on the Home view.".to_owned(),
    };
    check(
        "Local certificate",
        status.trust.status == "trusted",
        "fail",
        detail,
    )
}

/// The signer is listening.
fn signer_check(status: &OverviewView) -> DoctorCheck {
    let detail = match (
        status.server.state,
        status.server.port,
        &status.server.error,
    ) {
        ("running", Some(port), _) => format!("Listening on 127.0.0.1:{port}."),
        ("paused", _, _) => "Paused. Resume it from Home or the menu bar.".to_owned(),
        (_, _, Some(error)) => error.clone(),
        _ => "Starting.".to_owned(),
    };
    check("Signer", status.server.state == "running", "fail", detail)
}

/// A GST page has connected, or at least a browser trusted the certificate.
fn portal_check(status: &OverviewView) -> DoctorCheck {
    let detail = match (&status.last_connection, &status.last_tls_failure) {
        (Some(connection), _) => format!("Last connected from {}.", connection.detail),
        (None, Some(failure)) => format!("A browser refused the local certificate: {}", failure.detail),
        (None, None) if status.status_page_seen => "A browser trusts the certificate. Allow local network access on the GST site when asked.".to_owned(),
        (None, None) => "No GST page has connected yet.".to_owned(),
    };
    check(
        "GST portal",
        status.last_connection.is_some(),
        "warn",
        detail,
    )
}
