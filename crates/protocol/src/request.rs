//! Parsing of the portal's `action=sign` request frame.
//!
//! The frame is Java Properties style text: one `key=value` per line. Keys are
//! matched case-insensitively and values are trimmed, as the reference signer
//! does.

use std::collections::HashMap;

use crate::constants::MAX_REQUEST_BYTES;
use crate::pan::is_valid_pan;

/// What the portal is asking the user to sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    /// DSC registration: the content is the signer's own PAN.
    Registration,
    /// Any other document, typically the hex digest of a return.
    Document,
}

/// A parsed `action=sign` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignRequest {
    /// Exact text to sign (`tobesigned`), trimmed. Signed as UTF-8 bytes.
    pub content: String,
    /// Signer PAN from `panNo`, uppercased, when present and well formed.
    pub pan: Option<String>,
    /// Raw `signtype`; the portal sends `1`, the only supported value.
    pub signtype: String,
    /// Whether expired certificates must be excluded (`expirycheck`).
    pub expiry_check: bool,
    /// Accepted certificate classes from `certclass`, for example `2` and `3`.
    pub cert_classes: Vec<String>,
    /// Issuer filter from `issuername`, when not empty.
    pub issuer_name: Option<String>,
    /// Correlation id echoed back as `uniqueID`, when present.
    pub unique_id: Option<String>,
}

impl SignRequest {
    /// Classifies the request: registration when the content is the PAN itself.
    pub fn kind(&self) -> RequestKind {
        match &self.pan {
            Some(pan) if self.content.eq_ignore_ascii_case(pan) => RequestKind::Registration,
            _ => RequestKind::Document,
        }
    }
}

/// Why a frame could not be treated as a sign request.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RequestError {
    /// The frame exceeds [`MAX_REQUEST_BYTES`].
    #[error("request is larger than {MAX_REQUEST_BYTES} bytes")]
    TooLarge,
    /// The frame asks for something other than `sign`.
    #[error("unsupported action {0:?}")]
    UnsupportedAction(String),
    /// The frame has no `tobesigned` value.
    #[error("request has no content to sign")]
    MissingContent,
}

/// Parses one request frame into a [`SignRequest`].
///
/// # Errors
///
/// Returns [`RequestError`] when the frame is too large, is not a sign action,
/// or carries nothing to sign.
pub fn parse_request(text: &str) -> Result<SignRequest, RequestError> {
    if text.len() > MAX_REQUEST_BYTES {
        return Err(RequestError::TooLarge);
    }
    let fields = parse_fields(text);
    let action = fields.get("action").map(String::as_str).unwrap_or_default();
    if !action.eq_ignore_ascii_case("sign") {
        return Err(RequestError::UnsupportedAction(action.to_owned()));
    }
    let content = fields.get("tobesigned").cloned().unwrap_or_default();
    if content.is_empty() {
        return Err(RequestError::MissingContent);
    }
    Ok(SignRequest {
        content,
        pan: fields
            .get("panno")
            .map(|pan| pan.to_ascii_uppercase())
            .filter(|pan| is_valid_pan(pan)),
        signtype: non_empty(fields.get("signtype")).unwrap_or_else(|| "1".to_owned()),
        expiry_check: fields
            .get("expirycheck")
            .is_none_or(|value| !value.eq_ignore_ascii_case("false")),
        cert_classes: fields
            .get("certclass")
            .map(|value| split_classes(value))
            .unwrap_or_default(),
        issuer_name: non_empty(fields.get("issuername")),
        unique_id: non_empty(fields.get("uniqueid")),
    })
}

/// Splits the frame into lowercase keys and trimmed values. Comment lines
/// (`#`, `!`) and lines without `=` are skipped; a repeated key keeps its last
/// value, as `java.util.Properties` does.
fn parse_fields(text: &str) -> HashMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('!'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect()
}

/// Splits `2|3` into `["2", "3"]`, dropping empty parts.
fn split_classes(value: &str) -> Vec<String> {
    value
        .split('|')
        .map(str::trim)
        .filter(|class| !class.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns the value when present and not empty.
fn non_empty(value: Option<&String>) -> Option<String> {
    value.filter(|text| !text.is_empty()).cloned()
}

#[cfg(test)]
mod tests {
    use super::{RequestError, RequestKind, parse_request};

    /// The exact frame the portal's DSC directive builds.
    const PORTAL_FRAME: &str = "action=sign\ntobesigned=ABCDE1234F\npanNo=ABCDE1234F\nsigntype=1\nexpirycheck=true\nissuername=\ncertclass=2|3\ncerttype=DSC\ncertdetails=";

    /// Parses the portal frame field by field.
    #[test]
    fn parses_portal_frame() {
        let request = parse_request(PORTAL_FRAME).unwrap();
        assert_eq!(request.content, "ABCDE1234F");
        assert_eq!(request.pan.as_deref(), Some("ABCDE1234F"));
        assert_eq!(request.signtype, "1");
        assert!(request.expiry_check);
        assert_eq!(request.cert_classes, ["2", "3"]);
        assert_eq!(request.issuer_name, None);
        assert_eq!(request.kind(), RequestKind::Registration);
    }

    /// A 64-character digest with a PAN is a document, not a registration.
    #[test]
    fn classifies_documents() {
        let frame = "action=sign\ntobesigned=a21824d977b52f3a139d8fb717f97653570dacf3af9260b19013a23f512ea12e\npanNo=ABCDE1234F\nsigntype=1";
        let request = parse_request(frame).unwrap();
        assert_eq!(request.kind(), RequestKind::Document);
        assert_eq!(request.content.len(), 64);
    }

    /// Keys are case-insensitive, values trimmed, bad PANs dropped.
    #[test]
    fn normalises_fields() {
        let request =
            parse_request("ACTION = Sign\r\n TOBESIGNED = abc \r\npanno=bad\r\nexpirycheck=FALSE")
                .unwrap();
        assert_eq!(request.content, "abc");
        assert_eq!(request.pan, None);
        assert!(!request.expiry_check);
        assert_eq!(request.signtype, "1");
    }

    /// Rejects other actions, empty content and oversized frames.
    #[test]
    fn rejects_bad_frames() {
        assert_eq!(
            parse_request("action=verify\ntobesigned=x"),
            Err(RequestError::UnsupportedAction("verify".to_owned()))
        );
        assert_eq!(
            parse_request("action=sign\ntobesigned="),
            Err(RequestError::MissingContent)
        );
        let huge = format!("action=sign\ntobesigned={}", "a".repeat(20_000));
        assert_eq!(parse_request(&huge), Err(RequestError::TooLarge));
    }
}
