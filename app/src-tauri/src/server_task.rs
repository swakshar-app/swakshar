//! Starting, pausing and resuming the loopback signer.

use std::sync::Arc;

use swakshar_server::{bind_signer_port, serve, tls_config};
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
}

/// Starts the signer in the background.
pub(crate) fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        set_state(&app, ServerState::Starting, None);
        match launch(&app).await {
            Ok((port, task)) => set_state(&app, ServerState::Running(port), Some(task)),
            Err(error) => {
                log::error!("the signer could not start: {error}");
                set_state(&app, ServerState::Failed(error), None);
            }
        }
    });
}

/// Stops accepting connections and frees the port.
pub(crate) fn stop(app: &AppHandle) {
    set_state(app, ServerState::Paused, None);
}

/// Applies new settings by starting again.
pub(crate) fn restart(app: &AppHandle) {
    stop(app);
    start(app);
}

/// The current state.
pub(crate) fn current(app: &AppHandle) -> ServerState {
    lock(&app.state::<AppState>().server).state.clone()
}

/// Loads or mints the certificate, binds a port and spawns the accept loop.
async fn launch(app: &AppHandle) -> Result<(u16, JoinHandle<()>), String> {
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
    let task = tauri::async_runtime::spawn(async move {
        if let Err(error) = serve(listener, port, tls, server_settings, broker).await {
            log::error!("the signer stopped: {error}");
        }
    });
    log::info!("listening on wss://127.0.0.1:{port}");
    Ok((port, task))
}

/// Replaces the state, aborting any previous accept loop, and refreshes the tray.
fn set_state(app: &AppHandle, state: ServerState, task: Option<JoinHandle<()>>) {
    {
        let app_state = app.state::<AppState>();
        let mut control = lock(&app_state.server);
        if let Some(previous) = control.task.take() {
            previous.abort();
        }
        control.state = state;
        control.task = task;
    }
    tray::refresh(app);
}
