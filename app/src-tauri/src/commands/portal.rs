//! Main-window companion controls; no portal signing protocol is exposed here.

use std::path::PathBuf;

use tauri::{AppHandle, Manager as _};
use tauri_plugin_opener::OpenerExt as _;

use crate::embridge::{self, BridgeAction, BridgeStatus};
use crate::error::{CommandError, CommandResult};

/// Inspects the installed companion off the UI and async threads.
#[tauri::command]
pub(crate) async fn get_embridge_status(app: AppHandle) -> CommandResult<BridgeStatus> {
    let mut status = tauri::async_runtime::spawn_blocking(embridge::status)
        .await
        .map_err(|error| CommandError::Message(error.to_string()))?;
    status.installer_available = installer_path(&app)?.is_file();
    Ok(status)
}

/// macOS presents administrator approval for this explicit button action.
#[tauri::command]
pub(crate) async fn control_embridge(
    app: AppHandle,
    action: BridgeAction,
) -> CommandResult<BridgeStatus> {
    let mut status = tauri::async_runtime::spawn_blocking(move || embridge::control(action))
        .await
        .map_err(|error| CommandError::Message(error.to_string()))??;
    status.installer_available = installer_path(&app)?.is_file();
    Ok(status)
}

/// Fixed installer location; no command accepts a webview-supplied executable path.
fn installer_path(app: &AppHandle) -> CommandResult<PathBuf> {
    Ok(app
        .path()
        .resource_dir()?
        .join("embridge")
        .join("embridge_3.2.7.0.dmg"))
}

/// Opens the bundled, unmodified installer for the user's native installation flow.
#[tauri::command]
pub(crate) fn open_embridge_installer(app: AppHandle) -> CommandResult<()> {
    let path = installer_path(&app)?;
    if !cfg!(target_os = "macos") || !path.is_file() {
        return Err(CommandError::Message(
            "This build does not contain the macOS emBridge installer.".to_owned(),
        ));
    }
    app.opener()
        .open_path(path.display().to_string(), None::<&str>)
        .map_err(|error| CommandError::Message(error.to_string()))
}

/// Opens the fixed official download page rather than downloading a vendor binary.
#[tauri::command]
pub(crate) fn open_embridge_download(app: AppHandle) -> CommandResult<()> {
    app.opener()
        .open_url(embridge::DOWNLOAD_URL, None::<&str>)
        .map_err(|error| CommandError::Message(error.to_string()))
}
