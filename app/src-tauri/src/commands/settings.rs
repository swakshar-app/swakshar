//! Reading and saving settings.

use tauri::{AppHandle, Manager as _, State};
use tauri_plugin_autostart::ManagerExt as _;

use crate::error::{CommandError, CommandResult};
use crate::server_task::{self, ServerState};
use crate::settings::Settings;
use crate::state::{AppState, lock};

/// Current settings.
#[tauri::command]
pub(crate) fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

/// Validates, saves and applies settings. The signer restarts when its
/// port, greeting or allowed origins change.
#[tauri::command]
pub(crate) fn save_settings(app: AppHandle, settings: Settings) -> CommandResult<Settings> {
    settings.validate().map_err(CommandError::Message)?;
    let state = app.state::<AppState>();
    let previous = state.settings();
    let mut settings = settings;
    settings.onboarding_complete = previous.onboarding_complete;
    settings.signing_enabled = previous.signing_enabled;
    settings.save(&state.data_dir)?;
    *lock(&state.settings) = settings.clone();
    if settings.start_at_login != previous.start_at_login {
        let manager = app.autolaunch();
        let applied = if settings.start_at_login {
            manager.enable()
        } else {
            manager.disable()
        };
        if let Err(error) = applied {
            log::warn!("could not change start at login: {error}");
        }
    }
    let network_changed = settings.preferred_port != previous.preferred_port
        || settings.greeting_version != previous.greeting_version
        || settings.extra_origins != previous.extra_origins
        || settings.modules != previous.modules;
    if network_changed && server_task::current(&app) != ServerState::Paused {
        server_task::restart(&app);
    }
    Ok(settings)
}
