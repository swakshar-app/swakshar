//! The attached-token view keeps what the guide needs to act on a missing
//! driver: the maker's vendor id and its download page.

use swakshar_token::{Inventory, UsbToken};

use super::attached_views;

/// A maker's download page.
const PAGE: &str = "https://example.invalid/drivers";

/// A HYP2003-like token whose maker may publish a driver page.
fn token(driver_page: Option<&'static str>) -> UsbToken {
    UsbToken {
        vendor_id: 0x2CCF,
        product_id: 0x080A,
        maker: Some("Hypersecu".to_owned()),
        product: Some("USB TOKEN".to_owned()),
        family: Some("ePass2003 / HYP2003"),
        driver_families: &["ePass2003 / HYP2003"],
        driver_page,
    }
}

/// With no driver installed the token is missing, and the view carries the
/// vendor id and page so the guide can offer the download.
#[test]
fn missing_token_carries_its_driver_page() {
    let views = attached_views(&[token(Some(PAGE))], &Inventory::default());
    let seen: Vec<(u16, Option<&str>, &str)> = views
        .iter()
        .map(|view| (view.vendor_id, view.driver_page.as_deref(), view.state))
        .collect();
    assert_eq!(seen, [(0x2CCF, Some(PAGE), "missing")]);
}

/// A maker without a public page leaves the view without one.
#[test]
fn unknown_page_stays_absent() {
    let views = attached_views(&[token(None)], &Inventory::default());
    let pages: Vec<Option<&str>> = views
        .iter()
        .map(|view| view.driver_page.as_deref())
        .collect();
    assert_eq!(pages, [None]);
}
