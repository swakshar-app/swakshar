//! Choosing the certificate for a request: filters from the portal's request,
//! then the CCA PAN hash to put the right holder first.

use std::cmp::Reverse;

use sha2::{Digest as _, Sha256};

use crate::certinfo::CertSummary;
use crate::types::TokenEntry;

/// Filters taken from the portal's request.
#[derive(Debug, Clone, Copy)]
pub struct Criteria<'a> {
    /// PAN from `panNo`.
    pub pan: Option<&'a str>,
    /// Exclude certificates outside their validity period.
    pub expiry_check: bool,
    /// Accepted CCA classes, for example `2` and `3`; empty accepts all.
    pub classes: &'a [String],
    /// Required text in the issuer name.
    pub issuer_name: Option<&'a str>,
    /// Current time, Unix seconds.
    pub now: i64,
}

/// How a certificate relates to the requested PAN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanMatch {
    /// The certificate carries the PAN's hash.
    Match,
    /// The certificate carries another PAN's hash; GST will reject it.
    Mismatch,
    /// No PAN in the request, or none in the certificate.
    Unknown,
}

/// An eligible certificate, by position in the inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    /// Index into the token list.
    pub token: usize,
    /// Index into that token's certificates.
    pub certificate: usize,
    /// Relation to the requested PAN.
    pub pan_match: PanMatch,
    /// Expiry, used to prefer the newest certificate.
    pub not_after: i64,
}

/// Lowercase hex SHA-256 of a PAN after trimming blanks, as CCA-IOG defines
/// the subject serial number.
pub fn pan_hash(pan: &str) -> String {
    Sha256::digest(pan.trim().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Eligible certificates, best first: PAN matches, then unknown, then
/// mismatches; newest expiry first within each group.
pub fn candidates(tokens: &[TokenEntry], criteria: &Criteria<'_>) -> Vec<Candidate> {
    let wanted = criteria.pan.map(pan_hash);
    let mut found: Vec<Candidate> = tokens
        .iter()
        .enumerate()
        .flat_map(|(token, entry)| {
            entry
                .certificates
                .iter()
                .enumerate()
                .map(move |(certificate, cert)| (token, certificate, &cert.summary))
        })
        .filter(|(_, _, summary)| eligible(summary, criteria))
        .map(|(token, certificate, summary)| Candidate {
            token,
            certificate,
            pan_match: pan_match(summary, wanted.as_deref()),
            not_after: summary.not_after,
        })
        .collect();
    found.sort_by_key(|candidate| (rank(candidate.pan_match), Reverse(candidate.not_after)));
    found
}

/// Whether a certificate passes the request's filters.
fn eligible(summary: &CertSummary, criteria: &Criteria<'_>) -> bool {
    let class_ok = criteria.classes.is_empty()
        || criteria
            .classes
            .iter()
            .filter_map(|class| class.parse::<u8>().ok())
            .any(|class| summary.classes.contains(&class));
    let issuer_ok = criteria.issuer_name.is_none_or(|issuer| {
        summary
            .issuer_cn
            .to_ascii_lowercase()
            .contains(&issuer.to_ascii_lowercase())
    });
    summary.signing
        && summary.rsa
        && (!criteria.expiry_check || summary.valid_at(criteria.now))
        && class_ok
        && issuer_ok
}

/// Compares the certificate's PAN hash with the wanted one.
fn pan_match(summary: &CertSummary, wanted: Option<&str>) -> PanMatch {
    match (wanted, summary.pan_hash.as_deref()) {
        (Some(wanted), Some(actual)) if wanted == actual => PanMatch::Match,
        (Some(_), Some(_)) => PanMatch::Mismatch,
        _ => PanMatch::Unknown,
    }
}

/// Sort order for [`PanMatch`].
fn rank(pan_match: PanMatch) -> u8 {
    match pan_match {
        PanMatch::Match => 0,
        PanMatch::Unknown => 1,
        PanMatch::Mismatch => 2,
    }
}

#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
