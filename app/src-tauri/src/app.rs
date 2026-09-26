//! Assembles the Tauri app: plugins, state, tray, windows, commands.

use swakshar_tls::data_dir;
use swakshar_token::TokenService;
use tauri::{App, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::commands::{
    approve, doctor, drivers, history, overview, settings as settings_commands, trust,
};
use crate::settings::Settings;
use crate::state::AppState;
use crate::windows::{self, MAIN};
use crate::{server_task, tray};

/// Builds and runs the app until the user quits from the tray.
pub(crate) fn run() -> Result<(), String> {
    let data_dir = data_dir().ok_or("could not find a per-user data directory")?;
    let settings = Settings::load(&data_dir);
    let show_setup = !settings.onboarding_complete;
    let token = TokenService::spawn().map_err(|error| error.to_string())?;
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show(app, MAIN)
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(data_dir, token, settings))
        .setup(move |app| setup(app, show_setup))
        .on_window_event(windows::on_event)
        .invoke_handler(tauri::generate_handler![
            overview::get_overview,
            overview::list_tokens,
            overview::open_status_page,
            overview::set_paused,
            overview::complete_onboarding,
            doctor::run_doctor,
            approve::get_pending_request,
            approve::approve_request,
            approve::cancel_request,
            approve::refresh_request,
            settings_commands::get_settings,
            settings_commands::save_settings,
            drivers::add_driver,
            drivers::remove_driver,
            trust::install_trust,
            trust::remove_trust,
            history::get_activity,
            history::clear_activity,
        ])
        .build(tauri::generate_context!())
        .map_err(|error| error.to_string())?
        .run(|_app, event| {
            if let RunEvent::ExitRequested {
                api, code: None, ..
            } = event
            {
                api.prevent_exit();
            }
        });
    Ok(())
}

/// Menu bar only (no Dock icon), tray, signer, and the setup window on first run.
fn setup(app: &mut App, show_setup: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    tray::create(app)?;
    server_task::start(app.handle());
    if show_setup {
        windows::show(app.handle(), MAIN);
    }
    Ok(())
}
