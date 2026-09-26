//! In-app updates: check the newest published release, tell the user, and
//! leave downloading and restarting to their click (`update_install`). The
//! check is the only request Swakshar makes to the internet.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager as _};
use tauri_plugin_updater::{Update, UpdaterExt as _};

use crate::notify;
use crate::state::{AppState, lock, unix_now};

/// Wait after launch before the first check, so start-up stays quick.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(10);
/// Time between automatic checks.
const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);
/// Opening the main window checks again when the last check is this old.
const STALE_AFTER: Duration = Duration::from_secs(15 * 60);
/// Debug builds only: a manifest URL to test updates against.
const ENDPOINT_OVERRIDE: &str = "SWAKSHAR_UPDATE_ENDPOINT";

/// Where the update flow is.
#[derive(Clone, Default)]
pub(crate) enum Phase {
    /// Nothing to offer.
    #[default]
    Idle,
    /// Asking the release server.
    Checking,
    /// A newer version exists; nothing downloaded yet.
    Available(Update),
    /// Downloading, with bytes received and the total when known.
    Downloading(Update, u64, Option<u64>),
    /// Downloaded and verified; waiting for a restart.
    Ready(Update, Arc<Vec<u8>>),
    /// The download failed.
    Failed(Update, String),
}

/// Update state kept by the app.
#[derive(Default)]
pub(crate) struct Updates {
    /// Current phase.
    pub(crate) phase: Mutex<Phase>,
    /// When the last check finished, and its error if it failed.
    pub(crate) last_check: Mutex<Option<(Instant, i64, Option<String>)>>,
    /// Restart as soon as Swakshar is idle.
    pub(crate) restart_when_idle: AtomicBool,
    /// Version already announced by a notification.
    pub(crate) announced: Mutex<Option<String>>,
}

/// What the webview shows about updates.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateView {
    /// `unavailable`, `idle`, `checking`, `available`, `downloading`,
    /// `ready` or `failed`.
    pub(crate) state: &'static str,
    /// Offered version.
    pub(crate) version: Option<String>,
    /// Release notes for it.
    pub(crate) notes: Option<String>,
    /// Download progress, 0 to 100, when the size is known.
    pub(crate) progress: Option<u8>,
    /// Why the last check or download failed.
    pub(crate) error: Option<String>,
    /// Last check, Unix seconds.
    pub(crate) last_checked: Option<i64>,
    /// A restart is queued for when Swakshar is idle.
    pub(crate) restart_when_idle: bool,
}

/// True when this build carries the key that verifies updates.
pub(crate) fn configured(app: &AppHandle) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|key| !key.trim().is_empty())
}

/// The view of the current state.
pub(crate) fn view(app: &AppHandle) -> UpdateView {
    let updates = app.state::<Updates>();
    let last = lock(&updates.last_check).clone();
    let (state, update, progress, error) = match lock(&updates.phase).clone() {
        _ if !configured(app) => ("unavailable", None, None, None),
        Phase::Idle => (
            "idle",
            None,
            None,
            last.as_ref().and_then(|(_, _, error)| error.clone()),
        ),
        Phase::Checking => ("checking", None, None, None),
        Phase::Available(update) => ("available", Some(update), None, None),
        Phase::Downloading(update, received, total) => {
            let percent = total
                .filter(|total| *total > 0)
                .map(|total| u8::try_from(received.saturating_mul(100) / total).unwrap_or(100));
            ("downloading", Some(update), percent, None)
        }
        Phase::Ready(update, _) => ("ready", Some(update), Some(100), None),
        Phase::Failed(update, error) => ("failed", Some(update), None, Some(error)),
    };
    UpdateView {
        state,
        version: update.as_ref().map(|update| update.version.clone()),
        notes: update.and_then(|update| update.body),
        progress,
        error,
        last_checked: last.map(|(_, at, _)| at),
        restart_when_idle: updates
            .restart_when_idle
            .load(std::sync::atomic::Ordering::Acquire),
    }
}

/// Checks after launch and then hourly, while the Settings switch is on.
pub(crate) fn spawn_schedule(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_DELAY).await;
        loop {
            if app.state::<AppState>().settings().update_checks {
                check(&app).await;
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

/// Checks in the background when the last check is older than
/// `STALE_AFTER`; called when the main window opens.
pub(crate) fn check_if_stale(app: &AppHandle) {
    let stale = lock(&app.state::<Updates>().last_check)
        .as_ref()
        .is_none_or(|(at, _, _)| at.elapsed() > STALE_AFTER);
    if stale && app.state::<AppState>().settings().update_checks {
        let app = app.clone();
        tauri::async_runtime::spawn(async move { check(&app).await });
    }
}

/// Asks the release server once. Keeps a download in progress or finished.
pub(crate) async fn check(app: &AppHandle) -> UpdateView {
    let updates = app.state::<Updates>();
    let busy = {
        let mut phase = lock(&updates.phase);
        let busy = !configured(app)
            || matches!(
                *phase,
                Phase::Checking | Phase::Downloading(..) | Phase::Ready(..)
            );
        if !busy {
            *phase = Phase::Checking;
        }
        busy
    };
    if busy {
        return view(app);
    }
    let found = fetch(app).await;
    let error = found.as_ref().err().cloned();
    *lock(&updates.phase) = match found {
        Ok(Some(update)) => {
            announce(app, &update.version);
            Phase::Available(update)
        }
        Ok(None) | Err(_) => Phase::Idle,
    };
    *lock(&updates.last_check) = Some((Instant::now(), unix_now(), error));
    crate::tray::refresh(app);
    view(app)
}

/// Fetches the manifest and compares versions.
async fn fetch(app: &AppHandle) -> Result<Option<Update>, String> {
    let mut builder = app.updater_builder();
    if cfg!(debug_assertions)
        && let Ok(endpoint) = std::env::var(ENDPOINT_OVERRIDE)
    {
        let url = endpoint
            .parse()
            .map_err(|error| format!("{ENDPOINT_OVERRIDE}: {error}"))?;
        builder = builder
            .endpoints(vec![url])
            .map_err(|error| error.to_string())?;
    }
    let updater = builder.build().map_err(|error| error.to_string())?;
    updater.check().await.map_err(|error| error.to_string())
}

/// Posts one notification per newly offered version.
fn announce(app: &AppHandle, version: &str) {
    let updates = app.state::<Updates>();
    let fresh = {
        let mut announced = lock(&updates.announced);
        let fresh = announced.as_deref() != Some(version);
        if fresh {
            *announced = Some(version.to_owned());
        }
        fresh
    };
    if fresh {
        notify::post(
            app,
            "Update available",
            &format!("Swakshar {version} is available. Open Swakshar to update."),
        );
    }
}
