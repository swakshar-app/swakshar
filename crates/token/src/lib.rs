//! PKCS#11 access to Indian DSC tokens.
//!
//! Every call into a vendor module happens on one dedicated thread
//! ([`TokenService`]), because vendor PKCS#11 modules are frequently not
//! thread-safe. The pure parts (certificate summaries, PAN matching, module
//! discovery) are separate and testable without a token.

mod actor;
mod certinfo;
mod error;
mod macho;
mod modules;
mod pkcs11;
mod portal;
mod select;
mod service;
mod types;
mod usb;

pub use certinfo::{CertSummary, summarize};
pub use cryptoki::types::AuthPin;
pub use error::TokenError;
pub use macho::ArchSupport;
pub use modules::{ModuleCandidate, candidate_modules};
pub use portal::{criteria_for, portal_reply};
pub use select::{Candidate, Criteria, PanMatch, candidates, pan_hash};
pub use service::TokenService;
pub use types::{
    CertRef, Inventory, ModuleStatus, PinState, SignJob, SignMechanism, SignedOutput,
    TokenCertificate, TokenEntry,
};
pub use usb::{DriverState, UsbToken, attached_tokens, driver_page, driver_state};
