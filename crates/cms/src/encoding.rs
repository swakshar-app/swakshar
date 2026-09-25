//! Byte-level encodings around the signature: base64url and `DigestInfo`.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha1::{Digest as _, Sha1};

use crate::error::CmsError;

/// DER prefix of a SHA-1 `DigestInfo` with NULL parameters (RFC 8017, 9.2).
const SHA1_DIGEST_INFO_PREFIX: [u8; 15] = [
    0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14,
];

/// Encodes a DER CMS for the reply frame: URL-safe base64 without padding,
/// exactly as the reference signer does for `signtype=1`.
pub fn to_portal_base64(der: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(der)
}

/// Decodes a reply signature, tolerating trailing padding.
///
/// # Errors
///
/// Returns [`CmsError::Base64`] when the text is not URL-safe base64.
pub fn from_portal_base64(text: &str) -> Result<Vec<u8>, CmsError> {
    Ok(URL_SAFE_NO_PAD.decode(text.trim().trim_end_matches('='))?)
}

/// Builds the SHA-1 `DigestInfo` a token signs with raw `CKM_RSA_PKCS`, for
/// tokens without `CKM_SHA1_RSA_PKCS`. Always includes the NULL parameters.
pub fn sha1_digest_info(message: &[u8]) -> Vec<u8> {
    let digest = Sha1::digest(message);
    let mut info = Vec::with_capacity(SHA1_DIGEST_INFO_PREFIX.len() + digest.len());
    info.extend_from_slice(&SHA1_DIGEST_INFO_PREFIX);
    info.extend_from_slice(&digest);
    info
}

#[cfg(test)]
mod tests {
    use super::{from_portal_base64, sha1_digest_info, to_portal_base64};

    /// Round-trips bytes that exercise both URL-safe characters.
    #[test]
    fn base64url_round_trip() {
        let bytes = [0xfb_u8, 0xff, 0xbf, 0x00, 0x10];
        let text = to_portal_base64(&bytes);
        assert_eq!(text, "-_-_ABA");
        assert_eq!(from_portal_base64(&text).unwrap(), bytes);
        assert_eq!(from_portal_base64("-_-_ABA=").unwrap(), bytes);
    }

    /// The DigestInfo of "abc" ends with its well-known SHA-1.
    #[test]
    fn digest_info_layout() {
        let info = sha1_digest_info(b"abc");
        assert_eq!(info.len(), 35);
        assert_eq!(info[0], 0x30);
        let tail: Vec<u8> = info[15..].to_vec();
        let expected = [
            0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
            0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
        ];
        assert_eq!(tail, expected);
    }
}
