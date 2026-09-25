//! Errors raised while building or checking a CMS structure.

/// A CMS structure could not be built, decoded or trusted.
#[derive(Debug, thiserror::Error)]
pub enum CmsError {
    /// DER encoding or decoding failed.
    #[error("DER error: {0}")]
    Der(#[from] der::Error),
    /// The text is not valid URL-safe base64.
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    /// The signing time cannot be written as UTCTime (years 1950 to 2049).
    #[error("signing time is outside the UTCTime range")]
    SigningTime,
    /// The structure differs from the portal's layout.
    #[error("unexpected CMS layout: {0}")]
    Layout(&'static str),
    /// `messageDigest` is not the SHA-1 of the content.
    #[error("messageDigest does not match the content")]
    DigestMismatch,
    /// The signature does not verify with the embedded certificate.
    #[error("signature does not verify with the embedded certificate")]
    BadSignature,
}

/// Building failed either in CMS assembly or in the caller's signer.
#[derive(Debug, thiserror::Error)]
pub enum BuildError<E> {
    /// Assembly or encoding failed.
    #[error(transparent)]
    Cms(#[from] CmsError),
    /// The signer callback failed; the token error is kept intact.
    #[error("the token could not produce a signature")]
    Signer(E),
}

impl<E> From<der::Error> for BuildError<E> {
    /// Wraps a DER error as a CMS assembly error.
    fn from(error: der::Error) -> Self {
        Self::Cms(CmsError::Der(error))
    }
}
