//! The main window's update commands. Checking by hand works even when
//! automatic checks are off.

use tauri::AppHandle;

use crate::error::{CommandError, CommandResult};
use crate::update_install;
use crate::updates::{self, UpdateView};

/// Checks for an update now.
#[tauri::command]
pub(crate) async fn check_for_update(app: AppHandle) -> CommandResult<UpdateView> {
    Ok(updates::check(&app).await)
}

/// Starts downloading the offered update.
#[tauri::command]
pub(crate) fn download_update(app: AppHandle) -> CommandResult<UpdateView> {
    update_install::download(&app).map_err(CommandError::Message)?;
    Ok(updates::view(&app))
}

/// Restarts into the downloaded update now, or once Swakshar is idle.
#[tauri::command]
pub(crate) fn restart_to_update(app: AppHandle, when_idle: bool) -> CommandResult<UpdateView> {
    update_install::restart(&app, when_idle).map_err(CommandError::Message)?;
    Ok(updates::view(&app))
}
