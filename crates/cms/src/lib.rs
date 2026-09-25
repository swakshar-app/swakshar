//! CMS `SignedData` in the exact layout the GST portal verifies.
//!
//! The portal sends `signtype=1` and checks the reply against the reference
//! signer's output: the raw request text as attached content, SHA-1, exactly
//! three signed attributes (contentType, signingTime, messageDigest), the
//! signer certificate only, and URL-safe base64 without padding. Modern CMS
//! builders add attributes the portal rejects, so the structure is assembled
//! here field by field.

mod builder;
mod encoding;
mod error;
mod inspect;
mod oids;

pub use builder::{SignedDataInput, build_signed_data};
pub use encoding::{from_portal_base64, sha1_digest_info, to_portal_base64};
pub use error::{BuildError, CmsError};
pub use inspect::{Inspection, inspect_layout, inspect_signed_data, verify_rsa_sha1};
pub use x509_cert::Certificate;

#[cfg(test)]
mod tests;
