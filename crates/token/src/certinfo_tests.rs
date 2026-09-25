//! Certificate summary tests, using certificates minted with rcgen.

use der::Encode;
use rcgen::{CertificateParams, CustomExtension, DnType, KeyPair, KeyUsagePurpose};
use x509_cert::ext::pkix::CertificatePolicies;
use x509_cert::ext::pkix::certpolicy::PolicyInformation;

use super::{CCA_CLASS_POLICIES, summarize, to_decimal};

/// SHA-256 of the test PAN `ABCDE1234F`.
const PAN_HASH: &str = "6442fd73a940c1186d6268bd27f89233e12429902c7805037e8aab6e717be6d9";

/// DER of a certificatePolicies extension naming both CCA classes.
fn cca_policies() -> Vec<u8> {
    CertificatePolicies(
        CCA_CLASS_POLICIES
            .iter()
            .map(|(_, oid)| PolicyInformation {
                policy_identifier: *oid,
                policy_qualifiers: None,
            })
            .collect(),
    )
    .to_der()
    .unwrap()
}

/// A DSC-shaped certificate: holder name, PAN hash, signing key usage, classes.
fn dsc_like() -> Vec<u8> {
    let key = KeyPair::generate().unwrap();
    let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "SIGNER NAME");
    params.distinguished_name.push(
        DnType::CustomDnType(vec![2, 5, 4, 5]),
        PAN_HASH.to_ascii_uppercase(),
    );
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::ContentCommitment,
    ];
    params.custom_extensions = vec![CustomExtension::from_oid_content(
        &[2, 5, 29, 32],
        cca_policies(),
    )];
    params.self_signed(&key).unwrap().der().to_vec()
}

/// Reads holder, PAN hash, classes and signing capability.
#[test]
fn summarises_dsc_fields() {
    let summary = summarize(&dsc_like()).unwrap();
    assert_eq!(summary.subject_cn, "SIGNER NAME");
    assert_eq!(summary.pan_hash.as_deref(), Some(PAN_HASH));
    assert_eq!(summary.classes, [2, 3]);
    assert_eq!(summary.class_label(), "Class 3");
    assert!(summary.signing);
    assert!(!summary.rsa);
    assert_eq!(summary.fingerprint.len(), 64);
    assert!(summary.valid_at(summary.not_before));
}

/// Encryption-only key usage is not a signing certificate.
#[test]
fn detects_encryption_only_keys() {
    let key = KeyPair::generate().unwrap();
    let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
    params.key_usages = vec![KeyUsagePurpose::KeyEncipherment];
    let der = params.self_signed(&key).unwrap().der().to_vec();
    let summary = summarize(&der).unwrap();
    assert!(!summary.signing);
    assert_eq!(summary.pan_hash, None);
    assert_eq!(summary.class_label(), "Unclassified");
}

/// Big-endian serials become decimal text.
#[test]
fn converts_serials_to_decimal() {
    assert_eq!(to_decimal(&[]), "0");
    assert_eq!(to_decimal(&[0x00, 0xff]), "255");
    assert_eq!(to_decimal(&[0x01, 0x00]), "256");
    assert_eq!(
        to_decimal(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a]),
        "4759477275222530853130"
    );
}
