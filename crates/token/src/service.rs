//! Handle to the token thread, usable from async and blocking code.

use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};
use std::thread;

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
}

impl TokenService {
    /// Starts the token thread.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ServiceStopped`] when the thread cannot start.
    pub fn spawn() -> Result<Self, TokenError> {
        let (jobs, receiver) = channel();
        thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || Actor::default().run(&receiver))
            .map_err(|_| TokenError::ServiceStopped)?;
        Ok(Self { jobs })
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
