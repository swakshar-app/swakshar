//! Waiting for the signer to drain at exit.

use std::time::Duration;

use super::wait_for_drain;

/// The exit runs on the main thread, outside any async runtime. The wait
/// must work there: building a tokio timeout outside a runtime panics, and
/// a panic on the exit path stopped the restart into an update and aborted
/// Command-Q.
#[test]
fn waits_from_a_plain_thread() {
    let finished = tauri::async_runtime::spawn(async {});
    let stuck = tauri::async_runtime::spawn(std::future::pending::<()>());
    let outcome = std::thread::spawn(move || {
        (
            wait_for_drain(finished, Duration::from_secs(2)),
            wait_for_drain(stuck, Duration::from_millis(50)),
        )
    })
    .join()
    .expect("the wait must not panic off the runtime");
    assert_eq!(outcome, (true, false));
}
