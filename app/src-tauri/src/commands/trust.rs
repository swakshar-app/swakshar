//! Trusting and untrusting the local certificate. macOS shows its own prompt.

use swakshar_tls::{TlsIdentity, ensure_identity, tls_dir};
use tauri::{AppHandle, Manager as _};

use crate::commands::overview::{OverviewView, forget_trust, overview};
use crate::error::{CommandError, CommandResult};
use crate::state::{AppState, unix_now};

/// Adds the local CA to the login keychain.
#[tauri::command]
pub(crate) async fn install_trust(app: AppHandle) -> CommandResult<OverviewView> {
    let identity = identity(&app)?;
    let installed = run_blocking(move || swakshar_tls::install_trust(&identity)).await;
    forget_trust(&app);
    installed?;
    overview(&app).await
}

/// Removes the local CA's trust and deletes it from the keychain.
#[tauri::command]
pub(crate) async fn remove_trust(app: AppHandle) -> CommandResult<OverviewView> {
    let identity = identity(&app)?;
    let removed = run_blocking(move || swakshar_tls::remove_trust(&identity)).await;
    forget_trust(&app);
    removed?;
    overview(&app).await
}

/// The current identity, minted if missing.
fn identity(app: &AppHandle) -> CommandResult<TlsIdentity> {
    let state = app.state::<AppState>();
    Ok(ensure_identity(&tls_dir(&state.data_dir), unix_now())?.0)
}

/// Runs a blocking trust operation off the async runtime.
async fn run_blocking<F>(operation: F) -> CommandResult<()>
where
    F: FnOnce() -> Result<(), swakshar_tls::TlsError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| CommandError::Message(error.to_string()))?
        .map_err(CommandError::from)
}
