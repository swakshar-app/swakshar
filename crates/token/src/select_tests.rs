//! Certificate selection tests over hand-built summaries.

use std::path::PathBuf;

use super::{Criteria, PanMatch, candidates, pan_hash};
use crate::certinfo::CertSummary;
use crate::types::{PinState, TokenCertificate, TokenEntry};

/// 2026-09-25T00:00:00Z.
const NOW: i64 = 1_790_294_400;

/// A valid class 3 signing certificate for `pan`, expiring at `not_after`.
fn summary(pan: &str, not_after: i64) -> CertSummary {
    CertSummary {
        subject_cn: format!("HOLDER {pan}"),
        issuer_cn: "Example Sub CA 2022".to_owned(),
        serial_decimal: "1".to_owned(),
        not_before: NOW - 1_000,
        not_after,
        pan_hash: Some(pan_hash(pan)),
        classes: vec![2, 3],
        signing: true,
        rsa: true,
        fingerprint: String::new(),
    }
}

/// A token holding the given summaries.
fn token(serial: &str, summaries: Vec<CertSummary>) -> TokenEntry {
    TokenEntry {
        module: PathBuf::from("/usr/local/lib/test.dylib"),
        label: serial.to_owned(),
        serial: serial.to_owned(),
        manufacturer: String::new(),
        model: String::new(),
        pin: PinState::default(),
        certificates: summaries
            .into_iter()
            .map(|summary| TokenCertificate {
                id: vec![1],
                label: String::new(),
                der: Vec::new(),
                summary,
            })
            .collect(),
    }
}

/// Default criteria for `pan`, accepting classes 2 and 3.
fn criteria<'a>(pan: Option<&'a str>, classes: &'a [String]) -> Criteria<'a> {
    Criteria {
        pan,
        expiry_check: true,
        classes,
        issuer_name: None,
        now: NOW,
    }
}

/// The CCA hash of the test PAN.
#[test]
fn hashes_pan_like_cca() {
    assert_eq!(
        pan_hash(" ABCDE1234F "),
        "6442fd73a940c1186d6268bd27f89233e12429902c7805037e8aab6e717be6d9"
    );
}

/// With three tokens plugged in, the requested holder comes first.
#[test]
fn prefers_matching_pan_across_tokens() {
    let classes = vec!["2".to_owned(), "3".to_owned()];
    let tokens = vec![
        token("T1", vec![summary("PQRST6789Z", NOW + 900)]),
        token("T2", vec![summary("ABCDE1234F", NOW + 100)]),
    ];
    let found = candidates(&tokens, &criteria(Some("ABCDE1234F"), &classes));
    assert_eq!(found.len(), 2);
    assert_eq!((found[0].token, found[0].pan_match), (1, PanMatch::Match));
    assert_eq!(found[1].pan_match, PanMatch::Mismatch);
}

/// Expired, non-signing, non-RSA and wrong-class certificates are dropped.
#[test]
fn filters_ineligible_certificates() {
    let classes = vec!["3".to_owned()];
    let mut expired = summary("ABCDE1234F", NOW - 1);
    expired.not_before = NOW - 10;
    let mut encryption = summary("ABCDE1234F", NOW + 10);
    encryption.signing = false;
    let mut class_two = summary("ABCDE1234F", NOW + 10);
    class_two.classes = vec![2];
    let mut ecdsa = summary("ABCDE1234F", NOW + 10);
    ecdsa.rsa = false;
    let tokens = vec![token("T1", vec![expired, encryption, class_two, ecdsa])];
    assert!(candidates(&tokens, &criteria(Some("ABCDE1234F"), &classes)).is_empty());
}

/// Without a PAN every eligible certificate is Unknown, newest first.
#[test]
fn orders_by_expiry_without_pan() {
    let tokens = vec![token(
        "T1",
        vec![
            summary("ABCDE1234F", NOW + 10),
            summary("PQRST6789Z", NOW + 99),
        ],
    )];
    let found = candidates(&tokens, &criteria(None, &[]));
    assert_eq!(found[0].certificate, 1);
    assert!(
        found
            .iter()
            .all(|candidate| candidate.pan_match == PanMatch::Unknown)
    );
}
