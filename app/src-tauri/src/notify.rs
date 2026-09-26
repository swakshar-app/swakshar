//! Local macOS notifications, when the Settings switch allows them: an
//! update, a DSC close to expiry, a request the user may not have seen. No
//! server is involved.

use std::time::Duration;

use swakshar_protocol::format_date_utc;
use tauri::{AppHandle, Manager as _};
use tauri_plugin_notification::NotificationExt as _;

use crate::state::{AppState, lock, unix_now};
use crate::windows::APPROVE;

/// Wait after launch before the first expiry check.
const EXPIRY_FIRST_DELAY: Duration = Duration::from_secs(60);
/// Time between expiry checks.
const EXPIRY_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Certificates expiring within this many seconds are announced.
const EXPIRY_WARNING_SECONDS: i64 = 30 * 86_400;
/// Wait before deciding the approval window went unnoticed.
const NUDGE_DELAY: Duration = Duration::from_secs(2);

/// Shows a notification unless the user turned them off.
pub(crate) fn post(app: &AppHandle, title: &str, body: &str) {
    if !app.state::<AppState>().settings().notifications {
        return;
    }
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        log::warn!("could not show a notification: {error}");
    }
}

/// Once a day, announces signing certificates on connected tokens that
/// expire within 30 days.
pub(crate) fn spawn_expiry_watch(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(EXPIRY_FIRST_DELAY).await;
        loop {
            announce_expiring(&app).await;
            tokio::time::sleep(EXPIRY_INTERVAL).await;
        }
    });
}

/// One expiry check.
async fn announce_expiring(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.settings().notifications {
        return;
    }
    let Ok(inventory) = state.token.inventory(state.settings().module_paths()).await else {
        return;
    };
    let now = unix_now();
    for certificate in inventory
        .tokens
        .iter()
        .flat_map(|token| &token.certificates)
        .map(|certificate| &certificate.summary)
        .filter(|summary| {
            summary.signing
                && summary.valid_at(now)
                && summary.not_after - now < EXPIRY_WARNING_SECONDS
        })
    {
        post(
            app,
            "Your DSC expires soon",
            &format!(
                "The certificate for {} expires on {}. Renew it with your CA.",
                certificate.subject_cn,
                format_date_utc(certificate.not_after)
            ),
        );
    }
}

/// After a request opens the approval window, notifies when that window
/// still lacks focus, for example behind a full-screen app.
pub(crate) fn nudge_if_unseen(app: &AppHandle, id: u64) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(NUDGE_DELAY).await;
        let waiting = lock(&app.state::<AppState>().pending)
            .as_ref()
            .is_some_and(|pending| pending.id == id);
        let focused = app
            .get_webview_window(APPROVE)
            .and_then(|window| window.is_focused().ok())
            .unwrap_or(false);
        if waiting && !focused {
            post(
                &app,
                "Signature request",
                "A signature request is waiting for your approval in Swakshar.",
            );
        }
    });
}
