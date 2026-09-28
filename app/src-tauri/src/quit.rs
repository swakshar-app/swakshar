//! Quitting. Every way out (the menu bar icon, Command-Q, Quit in the Dock,
//! logging out, restarting into an update) ends in [`on_exit`], which answers
//! a waiting page, frees the signer's port, lets open pages receive their
//! last reply and closes the token driver before the process ends.

use std::time::Duration;

use swakshar_protocol::REPLY_CANCELED;
use tauri::{AppHandle, Manager as _};

use crate::pending::{self, Outcome};
use crate::server_task;
use crate::state::{AppState, lock};
use crate::windows::{self, APPROVE, MAIN};

/// Longest the normal exit may take before the process ends regardless;
/// longer than [`DRAIN_DEADLINE`] and [`TOKEN_DEADLINE`] together.
const EXIT_DEADLINE: Duration = Duration::from_secs(5);
/// Longest quitting waits for open pages to receive their last reply; the
/// signer itself cuts them off after one second.
const DRAIN_DEADLINE: Duration = Duration::from_secs(2);
/// Longest quitting waits for the token thread to finish its driver call
/// and close the driver.
const TOKEN_DEADLINE: Duration = Duration::from_secs(2);

/// Quits from the menu bar icon. The windows disappear at once; the exit
/// then runs [`on_exit`], with a deadline so a stuck driver cannot hold the
/// process open.
pub(crate) fn quit(app: &AppHandle) {
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

/// The last step of every exit, on the main thread just before the process
/// ends. A page waiting on a request is told it was cancelled, the port
/// closes, open pages get their reply and a close frame, and the token
/// thread closes the driver, so the driver's own teardown at exit never
/// races a call in flight. Safe to run more than once.
pub(crate) fn on_exit(app: &AppHandle) {
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
    server_task::stop_and_wait(app, DRAIN_DEADLINE);
    if !app.state::<AppState>().token.shutdown(TOKEN_DEADLINE) {
        log::warn!("the token driver was still busy after {TOKEN_DEADLINE:?}");
    }
}
