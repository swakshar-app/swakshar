//! TLS identity for the loopback signer.
//!
//! The GST portal opens `wss://127.0.0.1:<port>`, so the browser must trust
//! the signer's certificate. Each install mints its own CA, name-constrained
//! to `127.0.0.1` and `localhost`, signs one leaf with it, and drops the CA
//! key without ever writing it to disk. Trusting that CA can therefore never
//! help anyone impersonate a real website.

mod error;
mod generate;
mod identity;
mod paths;
mod trust;

pub use error::TlsError;
pub use identity::{TlsIdentity, ensure_identity, load_identity};
pub use paths::{APP_IDENTIFIER, data_dir, tls_dir};
pub use trust::{TrustStatus, install_command, install_trust, remove_trust, trust_status};
