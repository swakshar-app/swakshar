//! Quitting. Every way out (the menu bar icon, Command-Q, Quit in the Dock,
//! logging out, restarting into an update) ends in [`on_exit`], which answers
//! a waiting page, frees the signer's port, lets open pages receive their
//! last reply and waits for the token thread to go idle before the process
//! ends. Each step is guarded: a panic there would abort the process inside
//! macOS's termination callback and skip the relaunch into an update.

use std::time::Duration;

use swakshar_protocol::REPLY_CANCELED;
use tauri::{AppHandle, Manager as _};

use crate::pending::{self, Outcome};
use crate::server_task;
use crate::state::{AppState, lock};
use crate::windows::{self, APPROVE, MAIN};

/// Longest the normal exit may take before the process ends regardless;
/// longer than [`DRAIN_DEADLINE`] and [`TOKEN_DEADLINE`] together.
const EXIT_DEADLINE: Duration = Duration::from_secs(12);
/// Longest quitting waits for open pages to receive their last reply; the
/// signer itself cuts them off after one second.
const DRAIN_DEADLINE: Duration = Duration::from_secs(2);
/// Longest quitting waits for the token thread to finish its driver call.
/// Reading a token through a vendor driver can take several seconds, and
/// exiting mid-call is what crashes.
const TOKEN_DEADLINE: Duration = Duration::from_secs(8);

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
/// thread finishes its driver call, so the driver's own teardown at exit
/// never races a call in flight. Safe to run more than once.
pub(crate) fn on_exit(app: &AppHandle) {
    guarded("answering the waiting page", || cancel_waiting(app));
    guarded("stopping the signer", || {
        server_task::stop_and_wait(app, DRAIN_DEADLINE);
    });
    guarded("stopping the token thread", || {
        if !app.state::<AppState>().token.shutdown(TOKEN_DEADLINE) {
            log::warn!("the token driver was still busy after {TOKEN_DEADLINE:?}");
        }
    });
}

/// Tells a page still waiting on a request that it was cancelled.
fn cancel_waiting(app: &AppHandle) {
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
}

/// Runs one step of the exit. A panic is logged (the panic hook records
/// where) and the next step still runs, so quitting always completes.
pub(crate) fn guarded(step: &str, work: impl FnOnce()) {
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err() {
        log::error!("{step} failed while quitting; carrying on");
    }
}

#[cfg(test)]
#[path = "quit_tests.rs"]
mod tests;
