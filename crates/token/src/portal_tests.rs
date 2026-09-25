//! Parity with the reference signer's reply frame and certificate fields,
//! using the fixtures in `fixtures/reference`.

use swakshar_protocol::parse_request;

use super::portal_reply;
use crate::certinfo::{parse_canonical, summarize};
use crate::select::pan_hash;
use crate::types::{SignMechanism, SignedOutput};

/// Test signer certificate.
const CERT: &[u8] = include_bytes!("../../../fixtures/reference/signer-cert.der");
/// The reference signer's CMS for `ABCDE1234F`.
const REFERENCE_CMS: &[u8] = include_bytes!("../../../fixtures/reference/python-signtype1.der");
/// The reference signer's reply frame for that CMS.
const REFERENCE_REPLY: &str = include_str!("../../../fixtures/reference/python-reply.txt");
/// 2026-09-25T00:00:00Z.
const SIGNING_TIME: i64 = 1_790_294_400;

/// The reply frame is identical to the reference signer's, line for line.
#[test]
fn reply_matches_reference() {
    let request = parse_request("action=sign\ntobesigned=ABCDE1234F\npanNo=ABCDE1234F\nsigntype=1\nexpirycheck=true\ncertclass=2|3").unwrap();
    let output = SignedOutput {
        cms_der: REFERENCE_CMS.to_vec(),
        summary: summarize(CERT).unwrap(),
        mechanism: SignMechanism::Sha1RsaPkcs,
    };
    assert_eq!(
        portal_reply(&output, &request, SIGNING_TIME),
        REFERENCE_REPLY
    );
}

/// The CCA-style fields of the reference certificate read correctly.
#[test]
fn reference_certificate_fields() {
    let summary = summarize(CERT).unwrap();
    assert_eq!(summary.subject_cn, "SWAKSHAR TEST SIGNER");
    assert_eq!(summary.issuer_cn, "Swakshar Test CA 2026");
    assert_eq!(summary.serial_decimal, "1311768467294899695");
    assert_eq!(
        summary.pan_hash.as_deref(),
        Some(pan_hash("ABCDE1234F").as_str())
    );
    assert_eq!(summary.classes, [2, 3]);
    assert!(summary.signing && summary.rsa);
}

/// A canonical certificate is accepted for embedding unchanged.
#[test]
fn canonical_certificate_is_accepted() {
    assert!(parse_canonical(CERT).is_ok());
}
