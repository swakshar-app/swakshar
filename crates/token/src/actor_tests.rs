//! The token thread's loop.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;

use tokio::sync::oneshot;

use super::{Actor, Job};

/// Once a shutdown is asked for, jobs still queued are dropped unanswered
/// rather than started, so no driver call begins while the app exits.
#[test]
fn queued_jobs_are_skipped_once_stopping() {
    let (jobs, receiver) = channel();
    let (reply, answer) = oneshot::channel();
    jobs.send(Job::Inventory {
        extra_modules: Vec::new(),
        reply,
    })
    .unwrap();
    drop(jobs);
    Actor::default().run(&receiver, &AtomicBool::new(true));
    assert!(answer.blocking_recv().is_err());
}
