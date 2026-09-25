//! Storing and loading the loopback TLS identity.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use der::Decode;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use sha1::{Digest as _, Sha1};
use x509_cert::Certificate;

use crate::error::TlsError;
use crate::generate::{Generated, generate};

/// CA certificate file. Public: this is what the OS is asked to trust.
const CA_FILE: &str = "ca.pem";
/// Leaf certificate file.
const CERT_FILE: &str = "server.pem";
/// Leaf private key file, readable by the owner only.
const KEY_FILE: &str = "server-key.pem";
/// Mint a new identity this long before the leaf expires.
const RENEW_BEFORE_SECONDS: i64 = 30 * 86_400;

/// The loopback server's certificate and key, and the CA that issued them.
/// Deliberately not `Debug`: it holds a private key.
#[derive(Clone)]
pub struct TlsIdentity {
    /// Directory holding the PEM files.
    pub dir: PathBuf,
    /// Leaf certificate, DER.
    pub cert_der: Vec<u8>,
    /// Leaf private key, PKCS#8 DER.
    pub key_der: Vec<u8>,
    /// CA certificate, DER.
    pub ca_der: Vec<u8>,
    /// Leaf `notAfter`, Unix seconds.
    pub not_after: i64,
    /// SHA-1 of the CA certificate, uppercase hex, as `security -Z` expects.
    pub ca_sha1: String,
}

impl TlsIdentity {
    /// Path of the CA certificate the OS trusts.
    pub fn ca_path(&self) -> PathBuf {
        self.dir.join(CA_FILE)
    }

    /// Path of the leaf certificate.
    pub fn cert_path(&self) -> PathBuf {
        self.dir.join(CERT_FILE)
    }
}

/// Loads the identity in `dir`, minting a new one when it is missing,
/// unreadable, or within 30 days of expiry. Returns whether it was minted.
///
/// # Errors
///
/// Returns [`TlsError`] when generation or file access fails.
pub fn ensure_identity(dir: &Path, now: i64) -> Result<(TlsIdentity, bool), TlsError> {
    match load_identity(dir) {
        Ok(identity) if identity.not_after - now > RENEW_BEFORE_SECONDS => {
            return Ok((identity, false));
        }
        Ok(_) => log::info!("the local certificate expires soon; minting a new one"),
        Err(error) => log::info!("minting a local certificate ({error})"),
    }
    write_identity(dir, &generate(now)?)?;
    Ok((load_identity(dir)?, true))
}

/// Loads an existing identity from `dir`.
///
/// # Errors
///
/// Returns [`TlsError`] when a file is missing or invalid.
pub fn load_identity(dir: &Path) -> Result<TlsIdentity, TlsError> {
    let cert = CertificateDer::from_pem_file(dir.join(CERT_FILE)).map_err(pem_error)?;
    let key = PrivateKeyDer::from_pem_file(dir.join(KEY_FILE)).map_err(pem_error)?;
    let ca = CertificateDer::from_pem_file(dir.join(CA_FILE)).map_err(pem_error)?;
    let leaf = Certificate::from_der(cert.as_ref())
        .map_err(|error| TlsError::Certificate(error.to_string()))?;
    let not_after = leaf
        .tbs_certificate
        .validity
        .not_after
        .to_unix_duration()
        .as_secs();
    Ok(TlsIdentity {
        dir: dir.to_path_buf(),
        cert_der: cert.to_vec(),
        key_der: key.secret_der().to_vec(),
        ca_der: ca.to_vec(),
        not_after: i64::try_from(not_after).unwrap_or(i64::MAX),
        ca_sha1: Sha1::digest(ca.as_ref())
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect(),
    })
}

/// Writes the PEM files; the directory and key are owner-only on Unix.
fn write_identity(dir: &Path, generated: &Generated) -> Result<(), TlsError> {
    fs::create_dir_all(dir)?;
    restrict(dir, 0o700)?;
    write_file(&dir.join(CA_FILE), &generated.ca_pem)?;
    write_file(&dir.join(CERT_FILE), &generated.cert_pem)?;
    let key_path = dir.join(KEY_FILE);
    write_file(&key_path, &generated.key_pem)?;
    restrict(&key_path, 0o600)
}

/// Replaces a file's contents.
fn write_file(path: &Path, contents: &str) -> Result<(), TlsError> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// Sets Unix permission bits; a no-op elsewhere.
fn restrict(path: &Path, mode: u32) -> Result<(), TlsError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}

/// Wraps a PEM error.
fn pem_error(error: rustls_pki_types::pem::Error) -> TlsError {
    TlsError::Pem(error.to_string())
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
