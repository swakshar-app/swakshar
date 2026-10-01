//! Assembles the Tauri app: plugins, state, tray, windows, commands.

use swakshar_tls::data_dir;
use swakshar_token::TokenService;
use tauri::{App, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::commands::{
    approve, diagnostics, doctor, drivers, history, overview, settings as settings_commands, trust,
    updates as update_commands,
};
use crate::settings::Settings;
use crate::state::AppState;
use crate::updates::Updates;
use crate::windows::{self, MAIN};
use crate::{dock, notify, quit, relaunch, server_task, tray, updates};

/// Argument the login item passes, so a start at login stays in the menu bar.
const LOGIN_ARG: &str = "--at-login";
/// cryptoki logs every mechanism a driver lists that it has no name for as
/// an error, about twenty per signature. They are harmless and would read as
/// failures in the diagnostic report, so this logger is muted.
const CRYPTOKI_MECHANISM_LOG: &str = "cryptoki::mechanism";

/// Builds and runs the app until the user quits; every exit runs the orderly
/// shutdown in [`quit::on_exit`].
pub(crate) fn run() -> Result<(), String> {
    let data_dir = data_dir().ok_or("could not find a per-user data directory")?;
    let settings = Settings::load(&data_dir);
    let at_login = std::env::args().any(|arg| arg == LOGIN_ARG);
    let after_update = relaunch::take(&data_dir);
    let show_window = relaunch::shows_window(settings.onboarding_complete, at_login, after_update);
    let signing = settings.signing_enabled;
    let token = TokenService::spawn().map_err(|error| error.to_string())?;
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show(app, MAIN)
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for(CRYPTOKI_MECHANISM_LOG, log::LevelFilter::Off)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![LOGIN_ARG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::new(data_dir, token, settings))
        .manage(Updates::default())
        .setup(move |app| setup(app, show_window, signing))
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
            update_commands::check_for_update,
            update_commands::download_update,
            update_commands::restart_to_update,
            diagnostics::diagnostic_report,
            diagnostics::open_issue_page,
            trust::install_trust,
            trust::remove_trust,
            history::get_activity,
            history::clear_activity,
        ])
        .build(tauri::generate_context!())
        .map_err(|error| error.to_string())?
        .run(|app, event| {
            if let RunEvent::ExitRequested {
                api, code: None, ..
            } = &event
            {
                api.prevent_exit();
            }
            if matches!(event, RunEvent::Exit) {
                quit::on_exit(app);
            }
            if reopened(&event) {
                windows::show(app, MAIN);
            }
        });
    Ok(())
}

/// Tray, and the signer when the user left it on. The main window opens,
/// with a Dock icon, when the user starts the app or until setup is done; a
/// start at login stays in the menu bar.
fn setup(
    app: &mut App,
    show_window: bool,
    signing: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    tray::create(app)?;
    if signing {
        server_task::start(app.handle());
    } else {
        server_task::stop(app.handle());
    }
    if show_window {
        windows::show(app.handle(), MAIN);
    } else {
        dock::refresh(app.handle(), false);
    }
    updates::spawn_schedule(app.handle());
    notify::spawn_expiry_watch(app.handle());
    Ok(())
}

/// True when macOS asks the running app to reopen, because the user opened
/// it again from Applications, Spotlight or the Dock.
#[cfg(target_os = "macos")]
fn reopened(event: &RunEvent) -> bool {
    matches!(event, RunEvent::Reopen { .. })
}

/// Other systems have no reopen event; a second start reaches the running
/// app through the single-instance plugin instead.
#[cfg(not(target_os = "macos"))]
fn reopened(_event: &RunEvent) -> bool {
    false
}
