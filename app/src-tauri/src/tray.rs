//! The menu bar icon and its menu.

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager as _, Wry};

use crate::server_task::{self, ServerState};
use crate::state::{AppState, lock};
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

/// Menu items updated as the state changes.
pub(crate) struct TrayItems {
    /// Disabled first line showing the state.
    status: MenuItem<Wry>,
    /// Pause or resume.
    pause: MenuItem<Wry>,
}

/// Builds the tray icon and menu.
pub(crate) fn create(app: &App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Starting", false, None::<&str>)?;
    let open = MenuItem::with_id(app, ITEM_OPEN, "Open Swakshar", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, ITEM_PAUSE, "Turn on signing", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ITEM_QUIT, "Quit Swakshar", true, None::<&str>)?;
    let first = PredefinedMenuItem::separator(app)?;
    let second = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&status, &first, &open, &pause, &second, &quit])?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .tooltip("Swakshar")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event)
        .build(app)?;
    app.manage(TrayItems { status, pause });
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
    for result in [items.status.set_text(status), items.pause.set_text(pause)] {
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
        ITEM_QUIT => crate::quit::quit(app),
        _ => {}
    }
}
