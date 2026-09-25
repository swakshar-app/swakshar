//! Frames the signer sends: the greeting and the reply to a sign request.

use crate::constants::SIGNER_ID;

/// Reply sent when the user declines or the request times out.
pub const REPLY_CANCELED: &str = "signing canceled";

/// Reply sent for every other failure.
pub const REPLY_FAILED: &str = "signing failed";

/// Length of the literal `signature= ` prefix the portal skips.
const SIGNATURE_PREFIX_LEN: usize = 11;

/// Builds the greeting the server must send as soon as the WebSocket opens.
/// The portal waits for it, then checks `version` and `ID`; the literal
/// `status = success` (with spaces) unblocks its page.
pub fn greeting(port: u16, version: &str) -> String {
    format!("status = success\nport = {port}\nversion = {version}\nID = {SIGNER_ID}")
}

/// Certificate and signature details carried by a successful reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuccessReply<'a> {
    /// Base64url CMS, without padding.
    pub signature: &'a str,
    /// Certificate serial number in decimal.
    pub serial_number: &'a str,
    /// Subject common name.
    pub common_name: &'a str,
    /// Issuer common name.
    pub issuer_name: &'a str,
    /// Certificate `notBefore`, `dd-MM-yyyy`.
    pub issued_date: &'a str,
    /// Certificate `notAfter`, `dd-MM-yyyy`.
    pub expiry_date: &'a str,
    /// For example `Class 3`.
    pub cert_class: &'a str,
    /// Signing time, `dd-MM-yyyy HH:mm:ss`.
    pub signing_time: &'a str,
    /// Echo of the request's `uniqueId`, possibly empty.
    pub unique_id: &'a str,
}

/// Renders the success frame.
///
/// The portal extracts the signature with
/// `data.substring(11, data.indexOf("SerialNo") - 1)`, so the frame starts
/// with exactly `signature= ` and has a single `\n` before `SerialNo`. Field
/// values are flattened to one line so they cannot break that layout.
pub fn render_success(reply: &SuccessReply<'_>) -> String {
    format!(
        "signature= {}\nSerialNo= {}\nCommonName= {}\nIssuerName= {}\nIssuedDate= {}\nExpiryDate= {}\nCertClass= {}\nSigningTime= {}\nuniqueID= {}",
        one_line(reply.signature),
        one_line(reply.serial_number),
        one_line(reply.common_name),
        one_line(reply.issuer_name),
        one_line(reply.issued_date),
        one_line(reply.expiry_date),
        one_line(reply.cert_class),
        one_line(reply.signing_time),
        one_line(reply.unique_id),
    )
}

/// Extracts the signature from a reply exactly the way the portal's
/// JavaScript does. Returns `None` where the portal would report a failure.
pub fn portal_extract_signature(frame: &str) -> Option<&str> {
    if frame == REPLY_CANCELED || frame == REPLY_FAILED || frame.contains("status = success") {
        return None;
    }
    let end = frame.find("SerialNo")?.checked_sub(1)?;
    frame
        .get(SIGNATURE_PREFIX_LEN..end)
        .filter(|signature| !signature.is_empty())
}

/// Replaces line breaks so a value always stays on its own line.
fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::{SuccessReply, greeting, portal_extract_signature, render_success};

    /// A reply with every field filled in.
    fn sample(signature: &str) -> SuccessReply<'_> {
        SuccessReply {
            signature,
            serial_number: "1234567890",
            common_name: "SIGNER NAME",
            issuer_name: "Example Sub CA 2022",
            issued_date: "13-02-2026",
            expiry_date: "13-02-2028",
            cert_class: "Class 3",
            signing_time: "25-09-2026 15:30:00",
            unique_id: "",
        }
    }

    /// The greeting matches the reference byte for byte.
    #[test]
    fn renders_greeting() {
        assert_eq!(
            greeting(1585, "2.8"),
            "status = success\nport = 1585\nversion = 2.8\nID = gstnInfy"
        );
    }

    /// Whatever the signature, the portal's slicing gets it back exactly.
    #[test]
    fn portal_slicing_recovers_signature() {
        for signature in [
            "MIIB-_x",
            "a",
            "MIAGCSqGSIb3DQEHAqCAMIACAQExCzAJBgUrDgMCGgUA",
        ] {
            let frame = render_success(&sample(signature));
            assert!(frame.starts_with("signature= "));
            assert_eq!(portal_extract_signature(&frame), Some(signature));
        }
    }

    /// Line breaks inside a value cannot corrupt the layout.
    #[test]
    fn flattens_multiline_values() {
        let mut reply = sample("SIG");
        reply.common_name = "A\nSerialNo= fake";
        let frame = render_success(&reply);
        assert_eq!(portal_extract_signature(&frame), Some("SIG"));
    }

    /// Failure frames and the greeting never yield a signature.
    #[test]
    fn failure_frames_have_no_signature() {
        assert_eq!(portal_extract_signature("signing failed"), None);
        assert_eq!(portal_extract_signature("signing canceled"), None);
        assert_eq!(portal_extract_signature(&greeting(1585, "2.8")), None);
    }
}
