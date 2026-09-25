//! Layout tests with a stand-in signer; real RSA signing is covered by the
//! token crate against a SoftHSM token.

use std::cell::RefCell;
use std::time::Duration;

use der::Decode;
use x509_cert::Certificate;

use crate::{
    BuildError, CmsError, SignedDataInput, build_signed_data, inspect_layout, inspect_signed_data,
    to_portal_base64,
};

/// 2026-09-25T00:00:00Z.
const SIGNING_TIME: Duration = Duration::from_secs(1_790_294_400);

/// A throwaway certificate; the builder only needs issuer, serial and DER.
fn test_certificate() -> Certificate {
    let key = rcgen::KeyPair::generate().unwrap();
    let params = rcgen::CertificateParams::new(vec!["signer.test".to_owned()]).unwrap();
    let certificate = params.self_signed(&key).unwrap();
    Certificate::from_der(certificate.der().as_ref()).unwrap()
}

/// The layout matches the portal's expectations and the signer saw exactly
/// the encoded signed attributes.
#[test]
fn builds_portal_layout() {
    let certificate = test_certificate();
    let seen = RefCell::new(Vec::new());
    let input = SignedDataInput {
        content: b"ABCDE1234F",
        certificate: &certificate,
        signing_time: SIGNING_TIME,
    };
    let der = build_signed_data(&input, |attrs: &[u8]| {
        seen.replace(attrs.to_vec());
        Ok::<_, ()>(vec![0x5a; 256])
    })
    .unwrap();
    let inspection = inspect_layout(&der).unwrap();
    assert_eq!(inspection.content, b"ABCDE1234F");
    assert_eq!(inspection.certificate, certificate);
    assert_eq!(inspection.signed_attributes_der, *seen.borrow());
    assert_eq!(inspection.signature, vec![0x5a; 256]);
    assert_eq!(inspection.signed_attributes_der.first(), Some(&0x31));
}

/// The digest check passes and only the stand-in signature fails.
#[test]
fn digest_matches_content() {
    let certificate = test_certificate();
    let input = SignedDataInput {
        content: b"a21824d977b52f3a139d8fb717f97653570dacf3af9260b19013a23f512ea12e",
        certificate: &certificate,
        signing_time: SIGNING_TIME,
    };
    let der = build_signed_data(&input, |_: &[u8]| Ok::<_, ()>(vec![1; 256])).unwrap();
    assert!(matches!(
        inspect_signed_data(&der),
        Err(CmsError::BadSignature)
    ));
}

/// The signer's own error comes back untouched.
#[test]
fn propagates_signer_errors() {
    let certificate = test_certificate();
    let input = SignedDataInput {
        content: b"x",
        certificate: &certificate,
        signing_time: SIGNING_TIME,
    };
    let result = build_signed_data(&input, |_: &[u8]| Err::<Vec<u8>, _>("token removed"));
    assert!(matches!(result, Err(BuildError::Signer("token removed"))));
}

/// UTCTime cannot express 2050 or later, so building refuses.
#[test]
fn rejects_out_of_range_time() {
    let certificate = test_certificate();
    let input = SignedDataInput {
        content: b"x",
        certificate: &certificate,
        signing_time: Duration::from_secs(2_556_144_000),
    };
    let result = build_signed_data(&input, |_: &[u8]| Ok::<_, ()>(vec![0; 8]));
    assert!(matches!(
        result,
        Err(BuildError::Cms(CmsError::SigningTime))
    ));
}

/// The reply encoding never contains padding or the standard alphabet's symbols.
#[test]
fn portal_encoding_is_url_safe() {
    let certificate = test_certificate();
    let input = SignedDataInput {
        content: b"x",
        certificate: &certificate,
        signing_time: SIGNING_TIME,
    };
    let der = build_signed_data(&input, |_: &[u8]| Ok::<_, ()>(vec![0xff; 256])).unwrap();
    let text = to_portal_base64(&der);
    assert!(!text.contains(['=', '+', '/']));
}
