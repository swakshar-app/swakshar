//! The menu bar icon and its menu.

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager as _, Wry};

use crate::server_task::{self, ServerState};
use crate::state::{AppState, lock};
use crate::updates::{self, Phase, Updates};
use crate::windows::{self, MAIN};

/// Monochrome template icon for the menu bar.
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-template.png");
/// Tray id.
const TRAY_ID: &str = "swakshar";
/// Menu item ids.
const ITEM_OPEN: &str = "open";
/// Pause or resume.
const ITEM_PAUSE: &str = "pause";
/// Quit.
const ITEM_QUIT: &str = "quit";
/// Check for, or open, an update.
const ITEM_UPDATE: &str = "update";

/// Menu items updated as the state changes.
pub(crate) struct TrayItems {
    /// Disabled first line showing the state.
    status: MenuItem<Wry>,
    /// Pause or resume.
    pause: MenuItem<Wry>,
    /// Check for updates, or the offered version.
    update: MenuItem<Wry>,
}

/// Builds the tray icon and menu.
pub(crate) fn create(app: &App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Starting", false, None::<&str>)?;
    let open = MenuItem::with_id(app, ITEM_OPEN, "Open Swakshar", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, ITEM_PAUSE, "Turn on signing", true, None::<&str>)?;
    let update = MenuItem::with_id(app, ITEM_UPDATE, "Check for updates", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ITEM_QUIT, "Quit Swakshar", true, None::<&str>)?;
    let first = PredefinedMenuItem::separator(app)?;
    let second = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&status, &first, &open, &pause, &update, &second, &quit],
    )?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .tooltip("Swakshar")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event)
        .build(app)?;
    app.manage(TrayItems {
        status,
        pause,
        update,
    });
    Ok(())
}

/// Updates the status line and the pause item.
pub(crate) fn refresh(app: &AppHandle) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };
    let waiting = lock(&app.state::<AppState>().pending).is_some();
    let state = server_task::current(app);
    let status = match (&state, waiting) {
        (_, true) => "Waiting for your approval".to_owned(),
        (ServerState::Starting, _) => "Starting".to_owned(),
        (ServerState::Running(port), _) => format!("Ready on port {port}"),
        (ServerState::Paused, _) => "Signing is off".to_owned(),
        (ServerState::Failed(_), _) => "Not running, open Swakshar".to_owned(),
    };
    let pause = if matches!(state, ServerState::Paused | ServerState::Failed(_)) {
        "Turn on signing"
    } else {
        "Turn off signing"
    };
    let update = match app
        .try_state::<Updates>()
        .map(|updates| lock(&updates.phase).clone())
    {
        Some(Phase::Available(update) | Phase::Failed(update, _)) => {
            format!("Update to {}", update.version)
        }
        Some(Phase::Downloading(..)) => "Downloading update".to_owned(),
        Some(Phase::Ready(update, _)) => format!("Restart to update to {}", update.version),
        _ => "Check for updates".to_owned(),
    };
    let updatable = updates::configured(app);
    for result in [
        items.status.set_text(status),
        items.pause.set_text(pause),
        items.update.set_text(update),
        items.update.set_enabled(updatable),
    ] {
        if let Err(error) = result {
            log::warn!("could not update the tray: {error}");
        }
    }
}

/// Menu actions.
fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        ITEM_OPEN => windows::show(app, MAIN),
        ITEM_PAUSE => {
            let off = matches!(
                server_task::current(app),
                ServerState::Paused | ServerState::Failed(_)
            );
            if let Err(error) = server_task::set_signing(app, off) {
                log::warn!("could not save the signing choice: {error}");
            }
        }
        ITEM_UPDATE => on_update_item(app),
        ITEM_QUIT => crate::quit::quit(app),
        _ => {}
    }
}

/// Opens Home when an update is on offer; otherwise checks now and says so
/// when Swakshar is already up to date.
fn on_update_item(app: &AppHandle) {
    if !matches!(*lock(&app.state::<Updates>().phase), Phase::Idle) {
        windows::show(app, MAIN);
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let view = updates::check(&app).await;
        match (view.state, view.error) {
            ("idle", None) => crate::notify::post(&app, "Swakshar", "Swakshar is up to date."),
            ("idle", Some(error)) => {
                crate::notify::post(&app, "Could not check for updates", &error)
            }
            _ => {}
        }
    });
}
