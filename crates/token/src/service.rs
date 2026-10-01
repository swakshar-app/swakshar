//! Handle to the token thread, usable from async and blocking code.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::Duration;

use tokio::sync::oneshot;

use crate::actor::{Actor, Job};
use crate::error::TokenError;
use crate::types::{Inventory, SignJob, SignedOutput};

/// Name of the thread that owns every PKCS#11 module.
const THREAD_NAME: &str = "swakshar-token";

/// Cheap, cloneable handle to the token thread.
#[derive(Debug, Clone)]
pub struct TokenService {
    /// Queue of jobs for the thread.
    jobs: Sender<Job>,
    /// Set by [`TokenService::shutdown`]; the thread starts no new job.
    stopping: Arc<AtomicBool>,
}

impl TokenService {
    /// Starts the token thread.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ServiceStopped`] when the thread cannot start.
    pub fn spawn() -> Result<Self, TokenError> {
        let (jobs, receiver) = channel();
        let stopping = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stopping);
        thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || Actor::default().run(&receiver, &flag))
            .map_err(|_| TokenError::ServiceStopped)?;
        Ok(Self { jobs, stopping })
    }

    /// Stops the token thread before the process exits: it finishes the
    /// driver call it is in, starts no other and ends, leaving the modules
    /// loaded for the process exit to tear down. Waits up to `deadline` and
    /// returns false if the thread was still busy. Without this, a driver's
    /// own teardown at exit can run while a call is in flight and crash the
    /// process. Safe to call twice.
    #[must_use]
    pub fn shutdown(&self, deadline: Duration) -> bool {
        self.stopping.store(true, Ordering::Release);
        let (done, finished) = channel();
        if self.jobs.send(Job::Shutdown { done }).is_err() {
            return true;
        }
        !matches!(
            finished.recv_timeout(deadline),
            Err(RecvTimeoutError::Timeout)
        )
    }

    /// Lists modules, tokens and certificates. No PIN is needed.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ServiceStopped`] when the thread is gone.
    pub async fn inventory(&self, extra_modules: Vec<PathBuf>) -> Result<Inventory, TokenError> {
        let (reply, receiver) = oneshot::channel();
        self.submit(Job::Inventory {
            extra_modules,
            reply,
        })?;
        receiver.await.map_err(|_| TokenError::ServiceStopped)
    }

    /// Signs on the token thread.
    ///
    /// # Errors
    ///
    /// Returns the token's error, for example a wrong PIN.
    pub async fn sign(&self, job: SignJob) -> Result<SignedOutput, TokenError> {
        let (reply, receiver) = oneshot::channel();
        self.submit(Job::Sign {
            job: Box::new(job),
            reply,
        })?;
        receiver.await.map_err(|_| TokenError::ServiceStopped)?
    }

    /// Blocking [`TokenService::inventory`], for command-line use outside a runtime.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ServiceStopped`] when the thread is gone.
    pub fn inventory_blocking(&self, extra_modules: Vec<PathBuf>) -> Result<Inventory, TokenError> {
        let (reply, receiver) = oneshot::channel();
        self.submit(Job::Inventory {
            extra_modules,
            reply,
        })?;
        receiver
            .blocking_recv()
            .map_err(|_| TokenError::ServiceStopped)
    }

    /// Blocking [`TokenService::sign`], for command-line use outside a runtime.
    ///
    /// # Errors
    ///
    /// Returns the token's error, for example a wrong PIN.
    pub fn sign_blocking(&self, job: SignJob) -> Result<SignedOutput, TokenError> {
        let (reply, receiver) = oneshot::channel();
        self.submit(Job::Sign {
            job: Box::new(job),
            reply,
        })?;
        receiver
            .blocking_recv()
            .map_err(|_| TokenError::ServiceStopped)?
    }

    /// Queues a job.
    fn submit(&self, job: Job) -> Result<(), TokenError> {
        self.jobs.send(job).map_err(|_| TokenError::ServiceStopped)
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
