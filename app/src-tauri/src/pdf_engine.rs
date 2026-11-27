//! Runs the bundled PDF signer with a transient PIN and no shell.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use secrecy::{ExposeSecret as _, SecretString};
use serde_json::json;
use sha2::{Digest as _, Sha256};
use swakshar_token::{TokenCertificate, TokenEntry};
use tauri::{AppHandle, Manager as _};

use crate::error::{CommandError, CommandResult};
use crate::pdf_files::PdfWorkspace;
use crate::pdf_types::{PdfPlacement, PdfState};
use crate::state::lock;

/// Deadline keeps a blocked token driver from leaving a permanent signing job.
const SIGN_DEADLINE: Duration = Duration::from_secs(120);
/// Observe cancellation and child exit without holding the process mutex.
const CHILD_POLL: Duration = Duration::from_millis(100);
/// pyHanko reads the PIN from this child-only variable, never a command argument.
const PIN_ENV: &str = "PYHANKO_PKCS11_PIN";

/// Refuses PINs that the upstream CLI would silently trim before token login.
pub(crate) fn validate_pin(pin: &SecretString) -> CommandResult<()> {
    let value = pin.expose_secret();
    let trimmed = value.trim_matches(|character: char| {
        character.is_whitespace() || ('\u{001c}'..='\u{001f}').contains(&character)
    });
    if value != trimmed || value.is_empty() {
        return Err(CommandError::Message(
            "The PDF signer cannot use an empty PIN or a PIN with leading or trailing whitespace. No token login was attempted.".to_owned(),
        ));
    }
    Ok(())
}

/// Distinguishes identical certificates on different tokens without disclosing serials.
pub(crate) fn certificate_id(token: &TokenEntry, certificate: &TokenCertificate) -> String {
    let mut hash = Sha256::new();
    for value in [
        token.module.as_os_str().as_encoded_bytes(),
        token.serial.as_bytes(),
        certificate.summary.fingerprint.as_bytes(),
    ] {
        hash.update(value.len().to_le_bytes());
        hash.update(value);
    }
    hex(&hash.finalize())
}

/// Resolves only inside the app bundle; never searches PATH or a user's scripts.
pub(crate) fn engine_path(app: &AppHandle) -> CommandResult<PathBuf> {
    let name = if cfg!(target_os = "windows") {
        "swakshar-pdf.exe"
    } else {
        "swakshar-pdf"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    Ok(app
        .path()
        .resource_dir()?
        .join("pdf-engine")
        .join(arch)
        .join(name))
}

/// Serializes the token profile without the PIN; JSON is accepted by pyHanko.
pub(crate) fn profile(token: &TokenEntry, certificate: &TokenCertificate) -> serde_json::Value {
    let serial = hex(token.serial.as_bytes());
    let id = hex(&certificate.id);
    json!({
        "pkcs11-setups": { "swakshar": {
            "module-path": token.module,
            "token-criteria": { "serial": serial },
            "cert-id": id, "key-id": id,
            "prompt-pin": if token.pin.protected_path { "defer" } else { "prompt" },
            "other-certs-to-pull": []
        }},
        "stamp-styles": { "swakshar": {
            "type": "text", "stamp-text": "", "border-width": 0,
            "background-opacity": 1,
            "background-layout": { "x-align": "mid", "y-align": "mid",
                "margins": { "left": 0, "right": 0, "top": 0, "bottom": 0 } }
        }}
    })
}

/// Runs one offline PAdES signature and discards vendor diagnostics.
pub(crate) fn sign(
    app: &AppHandle,
    workspace: &PdfWorkspace,
    placement: &PdfPlacement,
    id: u64,
    pin: Option<SecretString>,
) -> CommandResult<()> {
    let executable = engine_path(app)?;
    if !executable.is_file() {
        return Err(CommandError::Message(
            "The bundled PDF engine is missing. Reinstall Swakshar.".to_owned(),
        ));
    }
    let mut command = Command::new(executable);
    command
        .args(["--no-plugins", "--config"])
        .arg(workspace.directory.join("profile.json"))
        .args([
            "sign",
            "addsig",
            "--use-pades",
            "--field",
            &placement.field_spec(id),
            "--style-name",
            "swakshar",
            "pkcs11",
            "--p11-setup",
            "swakshar",
        ])
        .arg(workspace.directory.join("input.pdf"))
        .arg(workspace.directory.join("signed.pdf"))
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME")
        .env_remove(PIN_ENV)
        .env_remove("PYHANKO_PKCS11_SIGNING_PIN")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(pin) = &pin {
        command.env(PIN_ENV, pin.expose_secret());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(0x0800_0000);
    }
    let state = app.state::<PdfState>();
    {
        let mut child = lock(&state.child);
        if state.cancelled.load(Ordering::Acquire) {
            return Err(CommandError::Message("PDF signing cancelled.".to_owned()));
        }
        *child = Some(command.spawn()?);
    }
    drop(command);
    drop(pin);
    wait(&state)
}

/// Cancels and reaps the child, including on app exit. Never stops emBridge.
pub(crate) fn cancel(state: &PdfState) {
    cancel_matching(state, None);
}

/// Ignores a delayed cancellation from an editor that no longer owns the job.
pub(crate) fn cancel_document(state: &PdfState, document_id: u64) {
    cancel_matching(state, Some(document_id));
}

/// Serializes ownership checks with job initialization before stopping its child.
fn cancel_matching(state: &PdfState, document_id: Option<u64>) {
    let mut slot = lock(&state.child);
    if document_id
        .is_some_and(|id| id == 0 || state.active_document_id.load(Ordering::Acquire) != id)
    {
        return;
    }
    state.cancelled.store(true, Ordering::Release);
    if let Some(mut child) = slot.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Watches the process until it finishes, times out, or is explicitly cancelled.
fn wait(state: &PdfState) -> CommandResult<()> {
    let started = Instant::now();
    loop {
        let finished = {
            let mut slot = lock(&state.child);
            let child = slot
                .as_mut()
                .ok_or_else(|| CommandError::Message("PDF signing cancelled.".to_owned()))?;
            match child.try_wait() {
                Ok(Some(status)) => {
                    slot.take();
                    Some(status.success())
                }
                Ok(None) => None,
                Err(error) => {
                    drop(slot);
                    cancel(state);
                    return Err(error.into());
                }
            }
        };
        match finished {
            Some(true) => return Ok(()),
            Some(false) => return Err(CommandError::Message("PDF signing failed. Check the token PIN, certificate and document permissions, then refresh certificates before trying again.".to_owned())),
            None if started.elapsed() >= SIGN_DEADLINE => {
                cancel(state);
                return Err(CommandError::Message("PDF signing timed out. Reconnect the token before trying again.".to_owned()));
            }
            None => std::thread::sleep(CHILD_POLL),
        }
    }
}

/// Encodes token serial bytes and PKCS#11 object ids without interpreting them.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
