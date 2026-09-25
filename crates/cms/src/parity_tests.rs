//! Byte-for-byte parity with the reference signer that the GST portal has
//! already accepted. `fixtures/reference` holds its output for a throwaway
//! test identity: the signer certificate, and the CMS it built and really
//! signed for `ABCDE1234F` at 2026-09-25T00:00:00Z.

use std::time::Duration;

use der::Decode;
use x509_cert::Certificate;

use crate::{SignedDataInput, build_signed_data, inspect_signed_data};

/// Test signer certificate.
const CERT: &[u8] = include_bytes!("../../../fixtures/reference/signer-cert.der");
/// The reference signer's CMS.
const REFERENCE: &[u8] = include_bytes!("../../../fixtures/reference/python-signtype1.der");
/// Signing time used for the fixture.
const SIGNING_TIME: Duration = Duration::from_secs(1_790_294_400);
/// Content signed for the fixture.
const CONTENT: &[u8] = b"ABCDE1234F";

/// The reference CMS passes our full inspection, real RSA signature included.
#[test]
fn reference_cms_verifies() {
    let inspection = inspect_signed_data(REFERENCE).unwrap();
    assert_eq!(inspection.content, CONTENT);
    assert_eq!(inspection.certificate, Certificate::from_der(CERT).unwrap());
}

/// Given the same certificate, content, time and signature, the builder
/// reproduces the reference bytes exactly, and hands the signer exactly the
/// bytes the reference signed.
#[test]
fn builder_matches_reference_bytes() {
    let reference = inspect_signed_data(REFERENCE).unwrap();
    let certificate = Certificate::from_der(CERT).unwrap();
    let input = SignedDataInput {
        content: CONTENT,
        certificate: &certificate,
        signing_time: SIGNING_TIME,
    };
    let built = build_signed_data(&input, |attrs: &[u8]| {
        assert_eq!(attrs, reference.signed_attributes_der.as_slice());
        Ok::<_, ()>(reference.signature.clone())
    })
    .unwrap();
    assert_eq!(built, REFERENCE);
}
