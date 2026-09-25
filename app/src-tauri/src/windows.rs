//! The two windows: `main` (setup, activity, settings, help) and `approve`.

use tauri::{AppHandle, Manager as _, Window, WindowEvent};

use crate::pending::{self, Outcome};
use crate::state::{AppState, lock};

/// Main window label.
pub(crate) const MAIN: &str = "main";
/// Approval window label.
pub(crate) const APPROVE: &str = "approve";

/// Shows, restores and focuses a window.
pub(crate) fn show(app: &AppHandle, label: &str) {
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    for result in [window.unminimize(), window.show(), window.set_focus()] {
        if let Err(error) = result {
            log::warn!("could not bring {label} forward: {error}");
        }
    }
}

/// Hides a window.
pub(crate) fn hide(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label)
        && let Err(error) = window.hide()
    {
        log::warn!("could not hide {label}: {error}");
    }
}

/// Closing a window hides it; closing the approval window cancels its request.
pub(crate) fn on_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    api.prevent_close();
    let app = window.app_handle();
    if window.label() == APPROVE {
        let id = lock(&app.state::<AppState>().pending)
            .as_ref()
            .map(|pending| pending.id);
        if let Some(id) = id {
            pending::finish(
                app,
                id,
                Outcome::Canceled,
                Some(swakshar_protocol::REPLY_CANCELED.to_owned()),
                None,
            );
        }
    }
    if let Err(error) = window.hide() {
        log::warn!("could not hide {}: {error}", window.label());
    }
}
