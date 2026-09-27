//! Which certificates earn an expiry notification, and its wording.

use swakshar_token::CertSummary;

use super::{EXPIRY_WARNING_SECONDS, expiry_messages};

/// Now, for every test.
const NOW: i64 = 1_790_294_400;

/// A certificate for `name` expiring `seconds` from now.
fn certificate(name: &str, seconds: i64, signing: bool) -> CertSummary {
    CertSummary {
        subject_cn: name.to_owned(),
        issuer_cn: "Test CA".to_owned(),
        serial_decimal: "1".to_owned(),
        not_before: NOW - 86_400,
        not_after: NOW + seconds,
        pan_hash: None,
        classes: vec![3],
        signing,
        rsa: true,
        fingerprint: String::new(),
    }
}

/// Only valid signing certificates inside the warning window are announced.
#[test]
fn announces_only_signing_certificates_close_to_expiry() {
    let certificates = [
        certificate("SOON", 5 * 86_400, true),
        certificate("LATER", EXPIRY_WARNING_SECONDS + 86_400, true),
        certificate("AUTHORITY", 86_400, false),
        certificate("EXPIRED", -86_400, true),
    ];
    let messages = expiry_messages(&certificates, NOW);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("SOON"));
}

/// The message names the holder and the date the CA must renew by.
#[test]
fn names_the_holder_and_the_date() {
    let messages = expiry_messages(&[certificate("TEST HOLDER", 86_400, true)], NOW);
    assert_eq!(
        messages,
        ["The certificate for TEST HOLDER expires on 26-09-2026. Renew it with your CA."]
    );
}
