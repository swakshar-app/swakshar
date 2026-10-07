//! USB token recognition tests over a hand-written `system_profiler` report.

use std::path::PathBuf;

use super::{DetectedToken, needing_driver, parse_hex_id, parse_report};
use crate::macho::ArchSupport;
use crate::modules::ModuleCandidate;
use crate::types::ModuleStatus;

/// A report with a token behind a hub, an unrelated keyboard and a token
/// known only by name. Public ids only; no serial numbers.
const REPORT: &str = r#"{"SPUSBDataType":[{"_name":"USB 3.1 Bus","_items":[
    {"_name":"USB2.0 Hub","_items":[{"_name":"ePass2003",
     "manufacturer":"Feitian Technologies","product_id":"0x0807",
     "vendor_id":"0x096e  (Feitian Technologies, Inc.)"}]},
    {"_name":"Keyboard","manufacturer":"Apple Inc.","product_id":"0x0267",
     "vendor_id":"apple_vendor_id"},
    {"_name":"USB Token","manufacturer":"WatchData","product_id":"0x0001",
     "vendor_id":"0x1234"}]}]}"#;

/// A module status for one driver family.
fn status(family: &str, exists: bool, loaded: bool) -> ModuleStatus {
    ModuleStatus {
        candidate: ModuleCandidate {
            path: PathBuf::from("/test"),
            family: family.to_owned(),
            exists,
            arch: ArchSupport::Unknown,
            user_added: false,
        },
        loaded,
        error: None,
    }
}

/// Hex ids parse with or without the vendor name; other text does not.
#[test]
fn parses_ids() {
    assert_eq!(
        parse_hex_id("0x096e  (Feitian Technologies, Inc.)"),
        Some(0x096e)
    );
    assert_eq!(parse_hex_id("0x0807"), Some(0x0807));
    assert_eq!(parse_hex_id("apple_vendor_id"), None);
    assert_eq!(parse_hex_id(""), None);
}

/// Tokens are found behind hubs, by id and by name, and other devices are
/// ignored.
#[test]
fn recognises_nested_tokens() {
    let detected = needing_driver(&parse_report(REPORT.as_bytes()), &[]);
    let families: Vec<&str> = detected.iter().map(|token| token.family).collect();
    assert_eq!(families, ["ePass2003", "Watchdata ProxKey"]);
    assert_eq!(
        detected.first().map(|token| token.name.as_str()),
        Some("ePass2003")
    );
    assert!(detected.iter().all(|token| !token.driver_present));
}

/// A loaded driver hides its family; a present one is reported as such.
#[test]
fn reflects_driver_state() {
    let nodes = parse_report(REPORT.as_bytes());
    let loaded = needing_driver(&nodes, &[status("ePass2003 / HYP2003", true, true)]);
    assert_eq!(
        loaded.iter().map(|token| token.family).collect::<Vec<_>>(),
        ["Watchdata ProxKey"]
    );
    let present = needing_driver(&nodes, &[status("Watchdata", true, false)]);
    assert_eq!(
        present
            .iter()
            .find(|token| token.family == "Watchdata ProxKey"),
        Some(&DetectedToken {
            family: "Watchdata ProxKey",
            name: "USB Token".to_owned(),
            driver_url: None,
            driver_present: true,
        })
    );
}

/// Bad JSON yields nothing rather than an error.
#[test]
fn tolerates_bad_report() {
    assert!(parse_report(b"not json").is_empty());
    assert!(needing_driver(&parse_report(b"{}"), &[]).is_empty());
}
