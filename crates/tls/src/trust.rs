//! Trusting the local CA in the operating system's certificate store.
//!
//! macOS first: the login keychain is enough for Chrome, Edge, Brave and
//! Safari, and Firefox 120+ imports user-added roots from it. Windows and
//! Linux report [`TrustStatus::Unsupported`] until their stores are added.

use std::path::PathBuf;
use std::process::Command;

use crate::error::TlsError;
use crate::identity::TlsIdentity;

/// macOS `security` tool, always by absolute path, never through `PATH`.
const SECURITY_TOOL: &str = "/usr/bin/security";
/// Login keychain, relative to the home directory.
const LOGIN_KEYCHAIN: &str = "Library/Keychains/login.keychain-db";
/// Host the certificate must be valid for.
const LOOPBACK_HOST: &str = "127.0.0.1";

/// Whether browsers on this machine will accept the local certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    /// The leaf verifies for SSL against the user's trust settings.
    Trusted,
    /// Not trusted yet (or trust was removed).
    NotTrusted,
    /// This OS is not supported yet.
    Unsupported,
}

/// Checks trust by asking the OS to verify the leaf for `127.0.0.1`.
pub fn trust_status(identity: &TlsIdentity) -> TrustStatus {
    if !cfg!(target_os = "macos") {
        return TrustStatus::Unsupported;
    }
    let verified = Command::new(SECURITY_TOOL)
        .arg("verify-cert")
        .arg("-c")
        .arg(identity.cert_path())
        .args(["-p", "ssl", "-s", LOOPBACK_HOST, "-L", "-q"])
        .output()
        .is_ok_and(|output| output.status.success());
    if verified {
        TrustStatus::Trusted
    } else {
        TrustStatus::NotTrusted
    }
}

/// Adds the CA to the login keychain as an SSL trust root. macOS shows its
/// own password or Touch ID prompt; no administrator rights are needed.
///
/// # Errors
///
/// Returns [`TlsError::Command`] when the user cancels or the tool fails.
pub fn install_trust(identity: &TlsIdentity) -> Result<(), TlsError> {
    if !cfg!(target_os = "macos") {
        return Err(TlsError::Unsupported);
    }
    let mut command = Command::new(SECURITY_TOOL);
    command
        .args(["add-trusted-cert", "-r", "trustRoot", "-p", "ssl", "-k"])
        .arg(login_keychain()?)
        .arg(identity.ca_path());
    run("add-trusted-cert", &mut command)
}

/// Removes the trust setting and deletes the CA from the login keychain.
///
/// # Errors
///
/// Returns [`TlsError::Command`] when the user cancels or the tool fails.
pub fn remove_trust(identity: &TlsIdentity) -> Result<(), TlsError> {
    if !cfg!(target_os = "macos") {
        return Err(TlsError::Unsupported);
    }
    let mut untrust = Command::new(SECURITY_TOOL);
    untrust.arg("remove-trusted-cert").arg(identity.ca_path());
    run("remove-trusted-cert", &mut untrust)?;
    let mut delete = Command::new(SECURITY_TOOL);
    delete
        .args(["delete-certificate", "-Z", &identity.ca_sha1])
        .arg(login_keychain()?);
    run("delete-certificate", &mut delete)
}

/// The exact install command, shown to users who want to see what runs.
pub fn install_command(identity: &TlsIdentity) -> String {
    format!(
        "security add-trusted-cert -r trustRoot -p ssl -k ~/{LOGIN_KEYCHAIN} \"{}\"",
        identity.ca_path().display()
    )
}

/// Absolute path of the user's login keychain.
fn login_keychain() -> Result<PathBuf, TlsError> {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(LOGIN_KEYCHAIN))
        .ok_or(TlsError::NoDataDir)
}

/// Runs a `security` subcommand and maps failure to [`TlsError::Command`].
fn run(name: &'static str, command: &mut Command) -> Result<(), TlsError> {
    let output = command.output().map_err(|error| TlsError::Command {
        command: name,
        detail: error.to_string(),
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TlsError::Command {
        command: name,
        detail: if stderr.is_empty() {
            output.status.to_string()
        } else {
            stderr
        },
    })
}
