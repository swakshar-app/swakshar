//! Where Swakshar shows on macOS. While one of its windows is open it is a
//! regular app, with a Dock icon, a place in the app switcher and its own
//! menu bar. With every window closed it lives in the menu bar alone, unless
//! the user asked to keep it in the Dock.

use tauri::{AppHandle, Manager as _};

use crate::state::{AppState, lock};
use crate::windows::{APPROVE, MAIN};

/// True when Swakshar belongs in the Dock and the app switcher.
pub(crate) fn shows_in_dock(window_open: bool, keep_in_dock: bool) -> bool {
    window_open || keep_in_dock
}

/// Puts Swakshar in the Dock or takes it out. `opening` is true just before
/// a window is shown, so the Dock icon and the menu bar are in place when
/// the window takes focus.
pub(crate) fn refresh(app: &AppHandle, opening: bool) {
    let open = opening
        || [MAIN, APPROVE]
            .into_iter()
            .any(|label| window_open(app, label));
    let keep = lock(&app.state::<AppState>().settings).keep_in_dock;
    apply(app, shows_in_dock(open, keep));
}

/// True when the window labelled `label` is on screen or minimized to the
/// Dock.
fn window_open(app: &AppHandle, label: &str) -> bool {
    app.get_webview_window(label).is_some_and(|window| {
        window.is_visible().unwrap_or(false) || window.is_minimized().unwrap_or(false)
    })
}

/// Switches between a regular app and a menu bar only one.
#[cfg(target_os = "macos")]
fn apply(app: &AppHandle, in_dock: bool) {
    let policy = if in_dock {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    };
    if let Err(error) = app.set_activation_policy(policy) {
        log::warn!("could not change the Dock icon: {error}");
    }
}

/// Other systems list open windows in their taskbar on their own.
#[cfg(not(target_os = "macos"))]
fn apply(_app: &AppHandle, _in_dock: bool) {}

#[cfg(test)]
#[path = "dock_tests.rs"]
mod tests;
