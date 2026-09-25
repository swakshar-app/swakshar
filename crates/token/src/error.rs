//! Errors from the token layer, phrased for people rather than PKCS#11.

use cryptoki::error::{Error as CryptokiError, RvError};

use crate::types::PinState;

/// Something went wrong between the app and the token.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TokenError {
    /// The driver could not be loaded or initialised.
    #[error("could not load the token driver: {0}")]
    ModuleLoad(String),
    /// No connected token has the requested serial number.
    #[error("the token is not connected")]
    TokenNotFound,
    /// The certificate object is no longer on the token.
    #[error("the certificate is no longer on the token")]
    CertificateNotFound,
    /// No private key on the token matches the certificate.
    #[error("no private key on the token matches this certificate")]
    KeyNotFound,
    /// The PIN was wrong; carries the token's PIN flags afterwards.
    #[error("wrong PIN")]
    PinIncorrect(PinState),
    /// The user PIN is locked.
    #[error("the token PIN is locked")]
    PinLocked,
    /// The PIN is too short, too long or has invalid characters.
    #[error("the PIN has an invalid length or characters")]
    PinInvalid,
    /// A PIN is required and none was given.
    #[error("a PIN is required")]
    PinRequired,
    /// The token disappeared mid-operation.
    #[error("the token was removed")]
    TokenRemoved,
    /// The certificate could not be parsed.
    #[error("the certificate could not be read: {0}")]
    Certificate(String),
    /// The token's signature failed verification with its own certificate.
    #[error("the token's signature did not verify with its certificate")]
    SignatureMismatch,
    /// Building the CMS failed.
    #[error("building the signature failed: {0}")]
    Cms(String),
    /// Any other PKCS#11 failure.
    #[error("token error: {0}")]
    Pkcs11(String),
    /// The token thread has stopped.
    #[error("the token service has stopped")]
    ServiceStopped,
}

impl From<CryptokiError> for TokenError {
    /// Maps PKCS#11 return values onto the cases people can act on.
    fn from(error: CryptokiError) -> Self {
        match &error {
            CryptokiError::Pkcs11(rv, _) => match rv {
                RvError::PinIncorrect => Self::PinIncorrect(PinState::default()),
                RvError::PinLocked => Self::PinLocked,
                RvError::PinLenRange | RvError::PinInvalid => Self::PinInvalid,
                RvError::TokenNotPresent
                | RvError::DeviceRemoved
                | RvError::SessionHandleInvalid
                | RvError::SlotIdInvalid => Self::TokenRemoved,
                _ => Self::Pkcs11(error.to_string()),
            },
            CryptokiError::LibraryLoading(_) => Self::ModuleLoad(error.to_string()),
            _ => Self::Pkcs11(error.to_string()),
        }
    }
}

impl From<der::Error> for TokenError {
    /// A certificate that does not parse.
    fn from(error: der::Error) -> Self {
        Self::Certificate(error.to_string())
    }
}
