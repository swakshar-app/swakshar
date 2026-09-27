//! Starting, pausing and resuming the loopback signer.

use std::sync::Arc;

use swakshar_server::{Stopper, bind_signer_port, serve, stop_pair, tls_config};
use swakshar_tls::{ensure_identity, tls_dir};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager as _};

use crate::broker::UiBroker;
use crate::state::{AppState, lock, unix_now};
use crate::tray;

/// Where the signer is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum ServerState {
    /// Getting the certificate and a port.
    #[default]
    Starting,
    /// Listening on this port.
    Running(u16),
    /// Stopped by the user.
    Paused,
    /// Could not start.
    Failed(String),
}

/// Server state plus the task serving connections.
#[derive(Default)]
pub(crate) struct ServerControl {
    /// Current state.
    pub(crate) state: ServerState,
    /// Accept loop, aborted on pause.
    task: Option<JoinHandle<()>>,
    /// Tells the accept loop to stop.
    stopper: Option<Stopper>,
    /// Bumped on every start and stop; a launch that finishes after a newer
    /// start or stop discards itself instead of replacing the current state.
    generation: u64,
}

/// Starts the signer in the background.
pub(crate) fn start(app: &AppHandle) {
    let generation = transition(app, ServerState::Starting);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let launched = launch(&app).await;
        {
            let state = app.state::<AppState>();
            let mut control = lock(&state.server);
            if control.generation != generation {
                if let Ok((_, task, _)) = launched {
                    task.abort();
                }
                return;
            }
            match launched {
                Ok((port, task, stopper)) => {
                    control.state = ServerState::Running(port);
                    control.task = Some(task);
                    control.stopper = Some(stopper);
                }
                Err(error) => {
                    log::error!("the signer could not start: {error}");
                    control.state = ServerState::Failed(error);
                }
            }
        }
        tray::refresh(&app);
    });
}

/// Stops accepting connections and frees the port.
pub(crate) fn stop(app: &AppHandle) {
    transition(app, ServerState::Paused);
}

/// Turns signing on or off at the user's request and remembers the choice,
/// so the next launch starts the same way.
///
/// # Errors
///
/// Returns the I/O error when the settings file cannot be written.
pub(crate) fn set_signing(app: &AppHandle, enabled: bool) -> std::io::Result<()> {
    {
        let state = app.state::<AppState>();
        let mut settings = lock(&state.settings);
        settings.signing_enabled = enabled;
        settings.save(&state.data_dir)?;
    }
    if !enabled {
        stop(app);
    } else if matches!(current(app), ServerState::Paused | ServerState::Failed(_)) {
        start(app);
    }
    Ok(())
}

/// Applies new settings by starting again.
pub(crate) fn restart(app: &AppHandle) {
    start(app);
}

/// The current state.
pub(crate) fn current(app: &AppHandle) -> ServerState {
    lock(&app.state::<AppState>().server).state.clone()
}

/// Aborts any accept loop, moves to `state`, and returns the new generation.
fn transition(app: &AppHandle, state: ServerState) -> u64 {
    let generation = {
        let app_state = app.state::<AppState>();
        let mut control = lock(&app_state.server);
        if let Some(task) = control.task.take() {
            task.abort();
        }
        control.stopper = None;
        control.state = state;
        control.generation += 1;
        control.generation
    };
    tray::refresh(app);
    generation
}

/// Loads or mints the certificate, binds a port and spawns the accept loop.
async fn launch(app: &AppHandle) -> Result<(u16, JoinHandle<()>, Stopper), String> {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let dir = tls_dir(&state.data_dir);
    let (identity, _) =
        tauri::async_runtime::spawn_blocking(move || ensure_identity(&dir, unix_now()))
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
    let tls =
        tls_config(&identity.cert_der, &identity.key_der).map_err(|error| error.to_string())?;
    let (listener, port) = bind_signer_port(settings.preferred_port)
        .await
        .map_err(|error| error.to_string())?;
    let broker = Arc::new(UiBroker::new(app.clone()));
    let server_settings = settings.server_settings();
    let (stopper, signal) = stop_pair();
    let task = tauri::async_runtime::spawn(async move {
        if let Err(error) = serve(listener, port, tls, server_settings, broker, signal).await {
            log::error!("the signer stopped: {error}");
        }
    });
    log::info!("listening on wss://127.0.0.1:{port}");
    Ok((port, task, stopper))
}
