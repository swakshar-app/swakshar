//! Status for the main window, and the actions next to it.

use std::time::{Duration, Instant};

use serde::Serialize;
use swakshar_protocol::format_date_utc;
use swakshar_tls::{TrustStatus, install_command, load_identity, tls_dir, trust_status};
use tauri::{AppHandle, Manager as _};
use tauri_plugin_opener::OpenerExt as _;

use crate::error::{CommandError, CommandResult};
use crate::server_task::{self, ServerState};
use crate::state::{AppState, lock, unix_now};
use crate::views::{InventoryView, inventory_view};

/// How long a trust check stays fresh.
const TRUST_CACHE_TTL: Duration = Duration::from_secs(10);

/// Everything the Home view shows.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OverviewView {
    /// Signer state.
    pub(crate) server: ServerView,
    /// Local certificate trust.
    pub(crate) trust: TrustView,
    /// Guided setup finished.
    pub(crate) onboarding_complete: bool,
    /// A request is waiting for approval.
    pub(crate) waiting: bool,
    /// Last GST page that connected.
    pub(crate) last_connection: Option<EventView>,
    /// Last certificate rejection by a browser.
    pub(crate) last_tls_failure: Option<EventView>,
    /// A browser loaded the status page.
    pub(crate) status_page_seen: bool,
    /// Greeting version in use.
    pub(crate) greeting_version: String,
    /// App version.
    pub(crate) app_version: String,
}

/// Signer state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerView {
    /// `starting`, `running`, `paused` or `failed`.
    pub(crate) state: &'static str,
    /// Port, when running.
    pub(crate) port: Option<u16>,
    /// Error, when failed.
    pub(crate) error: Option<String>,
}

/// Local certificate trust.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrustView {
    /// `trusted`, `not-trusted`, `unsupported` or `missing`.
    pub(crate) status: &'static str,
    /// Leaf expiry, `dd-MM-yyyy`.
    pub(crate) valid_until: Option<String>,
    /// The exact install command, for people who want to see it.
    pub(crate) command: Option<String>,
}

/// Something that happened, and when.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EventView {
    /// What happened.
    pub(crate) detail: String,
    /// When, Unix seconds.
    pub(crate) at: i64,
}

/// Home view status.
#[tauri::command]
pub(crate) async fn get_overview(app: AppHandle) -> CommandResult<OverviewView> {
    overview(&app).await
}

/// Drivers, tokens and certificates. No PIN needed.
#[tauri::command]
pub(crate) async fn list_tokens(app: AppHandle) -> CommandResult<InventoryView> {
    let state = app.state::<AppState>();
    let inventory = state
        .token
        .inventory(state.settings().module_paths())
        .await?;
    Ok(inventory_view(&inventory, unix_now()))
}

/// Opens the status page in the default browser, to test certificate trust.
#[tauri::command]
pub(crate) fn open_status_page(app: AppHandle) -> CommandResult<()> {
    let ServerState::Running(port) = server_task::current(&app) else {
        return Err(CommandError::Message(
            "The signer is not running.".to_owned(),
        ));
    };
    app.opener()
        .open_url(format!("https://127.0.0.1:{port}/"), None::<&str>)
        .map_err(|error| CommandError::Message(error.to_string()))
}

/// Pauses or resumes signing.
#[tauri::command]
pub(crate) async fn set_paused(app: AppHandle, paused: bool) -> CommandResult<OverviewView> {
    if paused {
        server_task::stop(&app);
    } else if matches!(
        server_task::current(&app),
        ServerState::Paused | ServerState::Failed(_)
    ) {
        server_task::start(&app);
    }
    overview(&app).await
}

/// Marks guided setup as finished.
#[tauri::command]
pub(crate) async fn complete_onboarding(app: AppHandle) -> CommandResult<OverviewView> {
    {
        let state = app.state::<AppState>();
        let mut settings = lock(&state.settings);
        settings.onboarding_complete = true;
        settings.save(&state.data_dir)?;
    }
    overview(&app).await
}

/// Builds the Home view status.
pub(crate) async fn overview(app: &AppHandle) -> CommandResult<OverviewView> {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let diagnostics = lock(&state.diagnostics).clone();
    let waiting = lock(&state.pending).is_some();
    Ok(OverviewView {
        server: server_view(server_task::current(app)),
        trust: cached_trust_view(app).await?,
        onboarding_complete: settings.onboarding_complete,
        waiting,
        last_connection: diagnostics
            .last_connection
            .map(|(detail, at)| EventView { detail, at }),
        last_tls_failure: diagnostics
            .last_tls_failure
            .map(|(detail, at)| EventView { detail, at }),
        status_page_seen: diagnostics.status_page_seen,
        greeting_version: settings.greeting_version,
        app_version: app.package_info().version.to_string(),
    })
}

/// Server state for the webview.
fn server_view(state: ServerState) -> ServerView {
    let (name, port, error) = match state {
        ServerState::Starting => ("starting", None, None),
        ServerState::Running(port) => ("running", Some(port), None),
        ServerState::Paused => ("paused", None, None),
        ServerState::Failed(error) => ("failed", None, Some(error)),
    };
    ServerView {
        state: name,
        port,
        error,
    }
}

/// Trust state, asked of macOS at most every `TRUST_CACHE_TTL`: the check
/// spawns `security`, and the Home and Help views poll.
async fn cached_trust_view(app: &AppHandle) -> CommandResult<TrustView> {
    let state = app.state::<AppState>();
    let cached = lock(&state.trust_cache)
        .as_ref()
        .filter(|(checked, _)| checked.elapsed() < TRUST_CACHE_TTL)
        .map(|(_, view)| view.clone());
    if let Some(view) = cached {
        return Ok(view);
    }
    let view = trust_view(tls_dir(&state.data_dir)).await?;
    *lock(&state.trust_cache) = Some((Instant::now(), view.clone()));
    Ok(view)
}

/// Forgets the cached trust state, after installing or removing trust.
pub(crate) fn forget_trust(app: &AppHandle) {
    *lock(&app.state::<AppState>().trust_cache) = None;
}

/// Trust state; asks macOS on a blocking thread.
async fn trust_view(dir: std::path::PathBuf) -> CommandResult<TrustView> {
    let view = tauri::async_runtime::spawn_blocking(move || match load_identity(&dir) {
        Ok(identity) => TrustView {
            status: match trust_status(&identity) {
                TrustStatus::Trusted => "trusted",
                TrustStatus::NotTrusted => "not-trusted",
                TrustStatus::Unsupported => "unsupported",
            },
            valid_until: Some(format_date_utc(identity.not_after)),
            command: Some(install_command(&identity)),
        },
        Err(_) => TrustView {
            status: "missing",
            valid_until: None,
            command: None,
        },
    })
    .await
    .map_err(|error| CommandError::Message(error.to_string()))?;
    Ok(view)
}
