//! Wire protocol between the GST portal page and a local DSC signer.
//!
//! Everything here is pure: no I/O, no clock, no cryptography. The protocol is
//! specified in `docs/PROTOCOL.md`, written only from public sources: the
//! portal's own JavaScript, captured frames, and our own signatures.

mod constants;
mod datefmt;
mod origin;
mod pan;
mod reply;
mod request;

pub use constants::{
    DEFAULT_GREETING_VERSION, MAX_REQUEST_BYTES, SIGNER_ID, SIGNER_PORTS, SUPPORTED_SIGNTYPE,
};
pub use datefmt::{format_date_utc, format_datetime_ist};
pub use origin::OriginPolicy;
pub use pan::{is_valid_pan, mask_pan};
pub use reply::{
    REPLY_CANCELED, REPLY_FAILED, SuccessReply, greeting, portal_extract_signature, render_success,
};
pub use request::{RequestError, RequestKind, SignRequest, parse_request};
