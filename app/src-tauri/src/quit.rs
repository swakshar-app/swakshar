//! Quitting: answer the waiting page, free the signer's port, close the
//! windows, then exit, with a deadline so a stuck driver cannot hold it open.

use std::time::Duration;

use swakshar_protocol::REPLY_CANCELED;
use tauri::{AppHandle, Manager as _};

use crate::pending::{self, Outcome};
use crate::server_task;
use crate::state::{AppState, lock};
use crate::windows::{self, APPROVE, MAIN};

/// Longest the normal exit may take before the process ends regardless.
const EXIT_DEADLINE: Duration = Duration::from_secs(3);

/// Quits Swakshar. A page waiting on a request is told it was cancelled,
/// the listening port closes at once, and the windows disappear before the
/// slower teardown of the webviews and the token driver.
pub(crate) fn quit(app: &AppHandle) {
    let waiting = lock(&app.state::<AppState>().pending)
        .as_ref()
        .map(|pending| pending.id);
    if let Some(id) = waiting {
        pending::finish(
            app,
            id,
            Outcome::Canceled,
            Some(REPLY_CANCELED.to_owned()),
            None,
        );
    }
    server_task::stop(app);
    for label in [APPROVE, MAIN] {
        windows::hide(app, label);
    }
    std::thread::spawn(|| {
        std::thread::sleep(EXIT_DEADLINE);
        log::warn!("exit took longer than {EXIT_DEADLINE:?}; ending the process");
        std::process::exit(0);
    });
    app.exit(0);
}
