//! Driver state for plugged-in tokens, without hardware.

use std::path::PathBuf;

use super::{DriverState, HYPERSECU_INDIA_DOWNLOADS, UsbToken, driver_page, driver_state};
use crate::macho::ArchSupport;
use crate::modules::ModuleCandidate;
use crate::types::{Inventory, ModuleStatus, PinState, TokenEntry};

/// Families that serve the test token.
const FAMILIES: &[&str] = &["ePass2003"];

/// A Feitian-like token, or an unknown smart card when `known` is false.
fn usb(known: bool) -> UsbToken {
    UsbToken {
        vendor_id: 0x096E,
        product_id: 0x0807,
        maker: Some("Feitian".to_owned()),
        product: Some("ePass2003".to_owned()),
        family: known.then_some("ePass2003"),
        driver_families: if known { FAMILIES } else { &[] },
        driver_page: None,
    }
}

/// A driver status.
fn module(
    family: &str,
    path: &str,
    loaded: bool,
    arch: ArchSupport,
    user_added: bool,
) -> ModuleStatus {
    ModuleStatus {
        candidate: ModuleCandidate {
            path: PathBuf::from(path),
            family: family.to_owned(),
            exists: true,
            arch,
            user_added,
        },
        loaded,
        error: (!loaded).then(|| "CKR_GENERAL_ERROR".to_owned()),
    }
}

/// A token exposed by `module`.
fn token(module: &str) -> TokenEntry {
    TokenEntry {
        module: PathBuf::from(module),
        label: "test".to_owned(),
        serial: "0001".to_owned(),
        manufacturer: "Test".to_owned(),
        model: "Test".to_owned(),
        pin: PinState::default(),
        certificates: Vec::new(),
    }
}

/// Inventory from modules and tokens.
fn inventory(modules: Vec<ModuleStatus>, tokens: Vec<TokenEntry>) -> Inventory {
    Inventory { modules, tokens }
}

/// Only a driver for another family is installed.
#[test]
fn missing_without_a_matching_driver() {
    let found = inventory(
        vec![module(
            "SafeNet eToken",
            "/s.dylib",
            true,
            ArchSupport::Native,
            false,
        )],
        Vec::new(),
    );
    assert_eq!(driver_state(&usb(true), &found), DriverState::Missing);
}

/// The family's driver loaded and exposes a token.
#[test]
fn ready_when_the_driver_sees_a_token() {
    let found = inventory(
        vec![module(
            "ePass2003",
            "/e.dylib",
            true,
            ArchSupport::Native,
            false,
        )],
        vec![token("/e.dylib")],
    );
    assert_eq!(driver_state(&usb(true), &found), DriverState::Ready);
}

/// The driver loaded but no token appeared.
#[test]
fn not_seen_when_the_driver_sees_nothing() {
    let found = inventory(
        vec![module(
            "ePass2003",
            "/e.dylib",
            true,
            ArchSupport::Native,
            false,
        )],
        Vec::new(),
    );
    assert_eq!(driver_state(&usb(true), &found), DriverState::NotSeen);
}

/// The only matching driver is built for the other processor.
#[test]
fn flags_drivers_for_the_other_processor() {
    let found = inventory(
        vec![module(
            "ePass2003",
            "/e.dylib",
            false,
            ArchSupport::Incompatible,
            false,
        )],
        Vec::new(),
    );
    assert_eq!(
        driver_state(&usb(true), &found),
        DriverState::OtherArchitecture
    );
}

/// A matching driver failed to start; its error is passed on.
#[test]
fn reports_the_driver_error() {
    let found = inventory(
        vec![module(
            "ePass2003",
            "/e.dylib",
            false,
            ArchSupport::Native,
            false,
        )],
        Vec::new(),
    );
    assert_eq!(
        driver_state(&usb(true), &found),
        DriverState::Failed("CKR_GENERAL_ERROR".to_owned())
    );
}

/// An unknown smart card is ready when any driver sees a token.
#[test]
fn unknown_tokens_use_any_driver() {
    let found = inventory(
        vec![module(
            "SafeNet eToken",
            "/s.dylib",
            true,
            ArchSupport::Native,
            false,
        )],
        vec![token("/s.dylib")],
    );
    assert_eq!(driver_state(&usb(false), &found), DriverState::Ready);
}

/// A driver the user added counts even though its family is unknown.
#[test]
fn user_added_drivers_count() {
    let found = inventory(
        vec![module(
            "Added by you",
            "/mine.dylib",
            true,
            ArchSupport::Native,
            true,
        )],
        vec![token("/mine.dylib")],
    );
    assert_eq!(driver_state(&usb(true), &found), DriverState::Ready);
}

/// Hypersecu publishes a driver page; Feitian and unknown makers do not.
#[test]
fn knows_the_makers_driver_pages() {
    assert_eq!(driver_page(0x2CCF), Some(HYPERSECU_INDIA_DOWNLOADS));
    assert_eq!(driver_page(0x096E), None);
    assert_eq!(driver_page(0x1234), None);
}
