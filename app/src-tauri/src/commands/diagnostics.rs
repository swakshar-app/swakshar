//! A diagnostic report the user copies into a bug report. Swakshar never
//! sends it anywhere. It holds versions, the checklist, drivers, token
//! makers as USB reports them and recent log lines; never names, token
//! labels, PANs, token serials or the activity history, and the home folder
//! is shown as `~`.

use std::fmt::Write as _;
use std::fs;

use swakshar_protocol::format_date_utc;
use tauri::{AppHandle, Manager as _};
use tauri_plugin_opener::OpenerExt as _;

use crate::commands::doctor::{checks, gather};
use crate::error::{CommandError, CommandResult};
use crate::state::unix_now;
use crate::updates;

/// Where people report problems.
const ISSUES_URL: &str = "https://github.com/swakshar-app/swakshar/issues/new";
/// Log lines included from the end of the log file.
const LOG_LINES: usize = 80;

/// Builds the report.
#[tauri::command]
pub(crate) async fn diagnostic_report(app: AppHandle) -> CommandResult<String> {
    let (status, tokens) = gather(&app).await?;
    let system = tauri::async_runtime::spawn_blocking(system_version)
        .await
        .unwrap_or_else(|_| std::env::consts::OS.to_owned());
    let update = updates::view(&app);
    let mut report = String::new();
    let _ = writeln!(
        report,
        "Swakshar diagnostic report, {} UTC",
        format_date_utc(unix_now())
    );
    let _ = writeln!(
        report,
        "App: {} ({})",
        status.app_version,
        app.config().identifier
    );
    let _ = writeln!(report, "System: {system}, {}", std::env::consts::ARCH);
    let _ = writeln!(
        report,
        "Signing: {}, port {:?}",
        status.server.state, status.server.port
    );
    let _ = writeln!(report, "Local certificate: {}", status.trust.status);
    let _ = writeln!(
        report,
        "Updates: {}, last error {:?}",
        update.state, update.error
    );
    let _ = writeln!(report, "\nChecklist");
    for check in checks(&status, &tokens) {
        let _ = writeln!(
            report,
            "  [{}] {}: {}",
            check.status, check.label, check.detail
        );
    }
    let _ = writeln!(report, "\nDrivers");
    for module in &tokens.modules {
        let _ = writeln!(
            report,
            "  {}: {} found={} loaded={} other-processor={} error={:?}",
            module.family,
            module.path,
            module.found,
            module.loaded,
            module.needs_rosetta,
            module.error
        );
    }
    let _ = writeln!(report, "\nTokens");
    for (number, token) in tokens.tokens.iter().enumerate() {
        let _ = writeln!(
            report,
            "  Token {}: pin-pad={} pin-locked={} certificates={}",
            number + 1,
            token.pin_pad,
            token.pin_locked,
            token.certificates.len()
        );
    }
    let _ = writeln!(report, "\nPlugged in over USB");
    for attached in &tokens.attached {
        let _ = writeln!(
            report,
            "  {:?} {:?} ({:?}): {}",
            attached.maker, attached.product, attached.family, attached.state
        );
    }
    let _ = writeln!(report, "\nRecent log");
    report.push_str(&recent_log(&app));
    Ok(redact_home(&report))
}

/// Opens a new GitHub issue in the browser.
#[tauri::command]
pub(crate) fn open_issue_page(app: AppHandle) -> CommandResult<()> {
    app.opener()
        .open_url(ISSUES_URL, None::<&str>)
        .map_err(|error| CommandError::Message(error.to_string()))
}

/// The operating system and its version.
fn system_version() -> String {
    let version = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    match version {
        Some(version) => format!("macOS {version}"),
        None => std::env::consts::OS.to_owned(),
    }
}

/// The last `LOG_LINES` lines of the app's log file.
fn recent_log(app: &AppHandle) -> String {
    let Ok(dir) = app.path().app_log_dir() else {
        return "  (no log folder)\n".to_owned();
    };
    let file = dir.join(format!("{}.log", app.package_info().name));
    let text = fs::read_to_string(&file).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(LOG_LINES);
    lines
        .get(start..)
        .unwrap_or_default()
        .iter()
        .map(|line| format!("  {line}\n"))
        .collect()
}

/// Replaces the home folder with `~`, which also hides the user name.
fn redact_home(text: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => text.replace(&home, "~"),
        _ => text.to_owned(),
    }
}
