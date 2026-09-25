//! Data shared between the token thread and its callers.

use std::path::PathBuf;
use std::time::Duration;

use cryptoki::types::AuthPin;

use crate::certinfo::CertSummary;
use crate::modules::ModuleCandidate;

/// Where a certificate lives: driver module, token serial, object id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CertRef {
    /// PKCS#11 module that exposes the token.
    pub module: PathBuf,
    /// Token serial number as reported by the module.
    pub token_serial: String,
    /// `CKA_ID` of the certificate object (shared with its private key).
    pub cert_id: Vec<u8>,
}

/// PIN flags the token reports; read before asking for a PIN.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinState {
    /// At least one wrong PIN has been entered.
    pub count_low: bool,
    /// One more wrong PIN locks the token.
    pub final_try: bool,
    /// The user PIN is locked.
    pub locked: bool,
    /// The token has its own PIN pad.
    pub protected_path: bool,
}

/// A certificate object read from a token without logging in.
#[derive(Debug, Clone)]
pub struct TokenCertificate {
    /// `CKA_ID`.
    pub id: Vec<u8>,
    /// `CKA_LABEL`, lossily decoded.
    pub label: String,
    /// DER certificate.
    pub der: Vec<u8>,
    /// Parsed fields.
    pub summary: CertSummary,
}

/// A token present in one of a module's slots.
#[derive(Debug, Clone)]
pub struct TokenEntry {
    /// Module that exposes the token.
    pub module: PathBuf,
    /// Token label.
    pub label: String,
    /// Token serial number.
    pub serial: String,
    /// Manufacturer id.
    pub manufacturer: String,
    /// Model name.
    pub model: String,
    /// PIN flags.
    pub pin: PinState,
    /// Certificates on the token.
    pub certificates: Vec<TokenCertificate>,
}

/// Load result for one PKCS#11 module.
#[derive(Debug, Clone)]
pub struct ModuleStatus {
    /// The module as discovered.
    pub candidate: ModuleCandidate,
    /// Whether the module loaded and initialised.
    pub loaded: bool,
    /// Why loading failed, when it did.
    pub error: Option<String>,
}

/// Everything visible without a PIN.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    /// Every module that was probed.
    pub modules: Vec<ModuleStatus>,
    /// Every token found.
    pub tokens: Vec<TokenEntry>,
}

/// One signature to produce on the token thread.
pub struct SignJob {
    /// Certificate (and therefore key) to sign with.
    pub cert: CertRef,
    /// User PIN; `None` for tokens with a PIN pad.
    pub pin: Option<AuthPin>,
    /// Bytes to sign, attached to the CMS.
    pub content: Vec<u8>,
    /// Signing time since the Unix epoch.
    pub signing_time: Duration,
}

/// Which mechanism produced the signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignMechanism {
    /// `CKM_SHA1_RSA_PKCS`: the token hashed and signed.
    Sha1RsaPkcs,
    /// `CKM_RSA_PKCS` over a locally built SHA-1 `DigestInfo`.
    RsaPkcsDigestInfo,
}

/// A finished, locally verified signature.
#[derive(Debug, Clone)]
pub struct SignedOutput {
    /// DER CMS for the reply.
    pub cms_der: Vec<u8>,
    /// The signer certificate.
    pub summary: CertSummary,
    /// Mechanism used.
    pub mechanism: SignMechanism,
}
