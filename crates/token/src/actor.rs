//! The token thread: owns every loaded module and serves one job at a time.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};

use cryptoki::context::Pkcs11;
use swakshar_cms::{BuildError, SignedDataInput, build_signed_data};
use tokio::sync::oneshot;

use crate::certinfo::{parse_canonical, summarize};
use crate::error::TokenError;
use crate::macho::ArchSupport;
use crate::modules::candidate_modules;
use crate::pkcs11::{
    find_certificate, find_private_key, find_slot, load, login, read_tokens, sign_verified,
};
use crate::types::{Inventory, ModuleStatus, SignJob, SignMechanism, SignedOutput};

/// Work for the token thread; each job carries its reply channel.
pub(crate) enum Job {
    /// List modules, tokens and certificates.
    Inventory {
        /// User-added module paths, on top of the known ones.
        extra_modules: Vec<PathBuf>,
        /// Where to send the result.
        reply: oneshot::Sender<Inventory>,
    },
    /// Produce one signature.
    Sign {
        /// What to sign and with which key.
        job: Box<SignJob>,
        /// Where to send the result.
        reply: oneshot::Sender<Result<SignedOutput, TokenError>>,
    },
    /// Close every module and end the thread.
    Shutdown {
        /// Told once every module is closed.
        done: Sender<()>,
    },
}

/// State owned by the token thread.
#[derive(Default)]
pub(crate) struct Actor {
    /// Loaded and initialised modules, by path.
    contexts: HashMap<PathBuf, Pkcs11>,
}

impl Actor {
    /// Serves jobs until a shutdown or until every sender is dropped. Once
    /// `stopping` is set, queued jobs are dropped unanswered instead of
    /// starting another driver call. The modules stay loaded: the process
    /// is about to exit, and a driver's own finalize and unload can take
    /// seconds or repeat its teardown at exit. What matters is that no call
    /// is in flight when the thread ends here.
    pub(crate) fn run(mut self, jobs: &Receiver<Job>, stopping: &AtomicBool) {
        let mut done = None;
        for job in jobs {
            match job {
                Job::Shutdown { done: sender } => {
                    done = Some(sender);
                    break;
                }
                _ if stopping.load(Ordering::Acquire) => break,
                Job::Inventory {
                    extra_modules,
                    reply,
                } => {
                    let _ = reply.send(self.inventory(&extra_modules));
                }
                Job::Sign { job, reply } => {
                    let _ = reply.send(self.sign(*job));
                }
            }
        }
        if let Some(done) = done {
            let _ = done.send(());
        }
        std::mem::forget(self);
    }

    /// Probes every candidate module and lists the tokens they expose.
    fn inventory(&mut self, extra_modules: &[PathBuf]) -> Inventory {
        let mut inventory = Inventory::default();
        for candidate in candidate_modules(extra_modules) {
            let mut status = ModuleStatus {
                candidate: candidate.clone(),
                loaded: false,
                error: None,
            };
            if candidate.exists && candidate.arch == ArchSupport::Incompatible {
                status.error =
                    Some("built for another processor; open Swakshar using Rosetta".to_owned());
            } else if candidate.exists {
                match self
                    .context(&candidate.path)
                    .and_then(|pkcs11| read_tokens(pkcs11, &candidate.path))
                {
                    Ok(tokens) => {
                        status.loaded = true;
                        inventory.tokens.extend(tokens);
                    }
                    Err(error) => status.error = Some(error.to_string()),
                }
            }
            inventory.modules.push(status);
        }
        inventory
    }

    /// The loaded module at `path`, loading it on first use.
    fn context(&mut self, path: &Path) -> Result<&Pkcs11, TokenError> {
        if !self.contexts.contains_key(path) {
            let pkcs11 = load(path)?;
            self.contexts.insert(path.to_path_buf(), pkcs11);
        }
        self.contexts
            .get(path)
            .ok_or_else(|| TokenError::ModuleLoad(path.display().to_string()))
    }

    /// Logs in, builds and verifies the CMS, and always logs out again.
    fn sign(&mut self, job: SignJob) -> Result<SignedOutput, TokenError> {
        let pkcs11 = self.context(&job.cert.module)?;
        let slot = find_slot(pkcs11, &job.cert.token_serial)?;
        let mechanisms = pkcs11.get_mechanism_list(slot)?;
        let session = pkcs11.open_ro_session(slot)?;
        let cert_der = find_certificate(&session, &job.cert.cert_id)?;
        let certificate = parse_canonical(&cert_der)?;
        let summary = summarize(&cert_der)?;
        login(pkcs11, slot, &session, job.pin.as_ref())?;
        let mut mechanism = SignMechanism::Sha1RsaPkcs;
        let built = find_private_key(&session, &job.cert.cert_id, &certificate).and_then(|key| {
            let input = SignedDataInput {
                content: &job.content,
                certificate: &certificate,
                signing_time: job.signing_time,
            };
            build_signed_data(&input, |attrs: &[u8]| {
                let (signature, used) =
                    sign_verified(&session, key, &mechanisms, &certificate, attrs)?;
                mechanism = used;
                Ok(signature)
            })
            .map_err(|error| match error {
                BuildError::Signer(token_error) => token_error,
                BuildError::Cms(cms_error) => TokenError::Cms(cms_error.to_string()),
            })
        });
        if let Err(error) = session.logout() {
            log::warn!("logout after signing failed: {error}");
        }
        Ok(SignedOutput {
            cms_der: built?,
            summary,
            mechanism,
        })
    }
}

#[cfg(test)]
#[path = "actor_tests.rs"]
mod tests;
