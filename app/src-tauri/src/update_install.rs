//! Downloading an offered update and restarting into it, now or once
//! Swakshar is idle. Never while a signature request is waiting.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Manager as _};

use crate::notify;
use crate::server_task;
use crate::state::{AppState, lock, unix_now};
use crate::updates::{Phase, Updates};

/// How often a queued restart looks for an idle moment.
const IDLE_POLL: Duration = Duration::from_secs(30);
/// Idle means no GST page has connected for this long.
const IDLE_AFTER_SECONDS: i64 = 5 * 60;

/// Starts downloading the offered update in the background. The plugin
/// verifies the signature against the key built into the app.
pub(crate) fn download(app: &AppHandle) -> Result<(), String> {
    let updates = app.state::<Updates>();
    let update = {
        let mut phase = lock(&updates.phase);
        let (Phase::Available(update) | Phase::Failed(update, _)) = phase.clone() else {
            return Err("There is no update to download.".to_owned());
        };
        *phase = Phase::Downloading(update.clone(), 0, None);
        update
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut received = 0_u64;
        let progress_app = app.clone();
        let progress_update = update.clone();
        let downloaded = update
            .download(
                move |chunk, total| {
                    received = received.saturating_add(u64::try_from(chunk).unwrap_or(0));
                    *lock(&progress_app.state::<Updates>().phase) =
                        Phase::Downloading(progress_update.clone(), received, total);
                },
                || {},
            )
            .await;
        let version = update.version.clone();
        *lock(&app.state::<Updates>().phase) = match downloaded {
            Ok(bytes) => {
                notify::post(
                    &app,
                    "Update ready",
                    &format!("Swakshar {version} is ready. Restart to update."),
                );
                Phase::Ready(update, Arc::new(bytes))
            }
            Err(error) => Phase::Failed(update, error.to_string()),
        };
        crate::tray::refresh(&app);
    });
    Ok(())
}

/// Restarts into the downloaded update now, or queues the restart for when
/// Swakshar is idle.
pub(crate) fn restart(app: &AppHandle, when_idle: bool) -> Result<(), String> {
    if !matches!(*lock(&app.state::<Updates>().phase), Phase::Ready(..)) {
        return Err("The update has not finished downloading.".to_owned());
    }
    if when_idle {
        let updates = app.state::<Updates>();
        if !updates.restart_when_idle.swap(true, Ordering::AcqRel) {
            watch_for_idle(app);
        }
        return Ok(());
    }
    if lock(&app.state::<AppState>().pending).is_some() {
        return Err("A signature request is waiting. Finish it, then restart.".to_owned());
    }
    install_and_restart(app)
}

/// Checks every `IDLE_POLL` for a moment with no request and no recent
/// connection, then restarts.
fn watch_for_idle(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(IDLE_POLL).await;
            if !app
                .state::<Updates>()
                .restart_when_idle
                .load(Ordering::Acquire)
            {
                return;
            }
            if idle(&app) {
                if let Err(error) = install_and_restart(&app) {
                    log::warn!("could not restart into the update: {error}");
                }
                return;
            }
        }
    });
}

/// No request waiting, and no GST page connected in the last few minutes.
fn idle(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let waiting = lock(&state.pending).is_some();
    let recent = lock(&state.diagnostics)
        .last_connection
        .as_ref()
        .is_some_and(|(_, at)| unix_now() - *at < IDLE_AFTER_SECONDS);
    !waiting && !recent
}

/// Replaces the app with the downloaded update, frees the port, relaunches.
fn install_and_restart(app: &AppHandle) -> Result<(), String> {
    let Phase::Ready(update, bytes) = lock(&app.state::<Updates>().phase).clone() else {
        return Err("The update has not finished downloading.".to_owned());
    };
    update
        .install(bytes.as_slice())
        .map_err(|error| error.to_string())?;
    server_task::stop(app);
    app.restart()
}
