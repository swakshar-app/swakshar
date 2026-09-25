//! Signing on the token with PIN prompts, shared by `serve` and `selftest`.

use std::time::Duration;

use swakshar_token::{
    Candidate, CertRef, Inventory, SignJob, SignedOutput, TokenError, TokenService,
};

use crate::error::CliError;
use crate::prompt::{confirm, pin};

/// Wrong PINs allowed per request before giving up. The token keeps its
/// own, stricter counter; this only stops a loop of typos.
const MAX_PIN_ATTEMPTS: u32 = 3;

/// How an interactive signing attempt ended.
pub(crate) enum Signed {
    /// The signature, verified locally.
    Done(Box<SignedOutput>),
    /// The user backed out.
    Canceled,
}

/// Asks for the PIN (unless the token has a PIN pad) and signs, warning
/// before the attempt that would lock the token.
pub(crate) fn sign_interactively(
    service: &TokenService,
    inventory: &Inventory,
    candidate: &Candidate,
    content: &[u8],
    signing_time: i64,
) -> Result<Signed, CliError> {
    let token = inventory
        .tokens
        .get(candidate.token)
        .ok_or(TokenError::TokenNotFound)?;
    let certificate = token
        .certificates
        .get(candidate.certificate)
        .ok_or(TokenError::CertificateNotFound)?;
    let mut pin_state = token.pin;
    for _ in 0..MAX_PIN_ATTEMPTS {
        if pin_state.locked {
            return Err(TokenError::PinLocked.into());
        }
        if pin_state.final_try && !confirm("One more wrong PIN locks this token. Continue?") {
            return Ok(Signed::Canceled);
        }
        let pin = if token.pin.protected_path {
            println!("Enter the PIN on the token's own PIN pad.");
            None
        } else {
            match pin() {
                Some(pin) => Some(pin),
                None => return Ok(Signed::Canceled),
            }
        };
        let job = SignJob {
            cert: CertRef {
                module: token.module.clone(),
                token_serial: token.serial.clone(),
                cert_id: certificate.id.clone(),
            },
            pin,
            content: content.to_vec(),
            signing_time: Duration::from_secs(u64::try_from(signing_time).unwrap_or_default()),
        };
        match service.sign_blocking(job) {
            Ok(output) => return Ok(Signed::Done(Box::new(output))),
            Err(TokenError::PinIncorrect(state)) => {
                println!("Wrong PIN.");
                pin_state = state;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(CliError::Message(
        "too many wrong PINs for one request".to_owned(),
    ))
}
