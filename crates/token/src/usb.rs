//! Recognising a plugged-in DSC token on the USB bus when no PKCS#11 module
//! could read it, so the app can name the token and the driver it needs.
//!
//! Identification uses public USB ids and the device's own descriptor
//! strings. The serial number is never read.

use std::process::Command;

use serde::Deserialize;

use crate::types::ModuleStatus;

/// macOS `system_profiler`, always by absolute path, never through `PATH`.
const SYSTEM_PROFILER: &str = "/usr/sbin/system_profiler";
/// The `system_profiler` data type that lists USB devices.
const USB_DATA_TYPE: &str = "SPUSBDataType";
/// USB vendor id of Feitian Technologies.
const VENDOR_FEITIAN: u16 = 0x096e;
/// USB vendor id of Aladdin Knowledge Systems, now SafeNet and Thales.
const VENDOR_ALADDIN: u16 = 0x0529;
/// USB vendor id of Hypersecu.
const VENDOR_HYPERSECU: u16 = 0x2ccf;

/// A token family we can recognise on the bus.
struct KnownToken {
    /// Shown to the user.
    family: &'static str,
    /// Substring of the matching driver families in `modules.rs`.
    driver_key: &'static str,
    /// Public USB vendor id, when one is known.
    vendor_id: Option<u16>,
    /// Public product ids; empty means every product of the vendor.
    product_ids: &'static [u16],
    /// Lowercase fragments of the device's manufacturer or product string.
    name_hints: &'static [&'static str],
    /// The vendor's driver page, when a stable one is known.
    driver_url: Option<&'static str>,
}

/// Token families and how to recognise them. Ids come from the public
/// `usb.ids` list and the CCID driver's reader list; families without a
/// public id are matched on the strings the device reports about itself.
const KNOWN_TOKENS: &[KnownToken] = &[
    KnownToken {
        family: "ePass2003",
        driver_key: "ePass2003",
        vendor_id: Some(VENDOR_FEITIAN),
        product_ids: &[0x0807, 0x080a],
        name_hints: &["epass"],
        driver_url: None,
    },
    KnownToken {
        family: "HYP2003",
        driver_key: "HYP2003",
        vendor_id: Some(VENDOR_HYPERSECU),
        product_ids: &[],
        name_hints: &["hypersecu", "hyp2003", "hyperpki"],
        driver_url: Some("https://hypersecu.com/copy-of-downloads-for-india"),
    },
    KnownToken {
        family: "SafeNet eToken",
        driver_key: "SafeNet",
        vendor_id: Some(VENDOR_ALADDIN),
        product_ids: &[],
        name_hints: &["etoken", "safenet"],
        driver_url: None,
    },
    KnownToken {
        family: "Watchdata ProxKey",
        driver_key: "Watchdata",
        vendor_id: None,
        product_ids: &[],
        name_hints: &["watchdata", "proxkey"],
        driver_url: None,
    },
    KnownToken {
        family: "mToken CryptoID",
        driver_key: "mToken",
        vendor_id: None,
        product_ids: &[],
        name_hints: &["longmai", "mtoken", "cryptoid"],
        driver_url: Some("https://www.longmai.net"),
    },
    KnownToken {
        family: "TrustKey",
        driver_key: "TrustKey",
        vendor_id: None,
        product_ids: &[],
        name_hints: &["trustkey"],
        driver_url: None,
    },
];

/// A USB device recognised as a DSC token that no driver exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedToken {
    /// Token family, for display.
    pub family: &'static str,
    /// The device's own product string.
    pub name: String,
    /// The vendor's driver page, when a stable one is known.
    pub driver_url: Option<&'static str>,
    /// A driver file for this family exists but did not load.
    pub driver_present: bool,
}

/// One node of the USB tree. Only the fields needed to recognise a token are
/// read; the serial number is deliberately left out.
#[derive(Debug, Default, Deserialize)]
struct UsbNode {
    /// Product string.
    #[serde(rename = "_name", default)]
    name: String,
    /// Manufacturer string.
    #[serde(default)]
    manufacturer: String,
    /// `0x096e` or `0x096e  (Vendor Name)`.
    #[serde(default)]
    vendor_id: String,
    /// `0x0807`.
    #[serde(default)]
    product_id: String,
    /// Devices behind this hub or controller.
    #[serde(rename = "_items", default)]
    items: Vec<UsbNode>,
}

/// The whole `system_profiler` report.
#[derive(Debug, Default, Deserialize)]
struct UsbReport {
    /// One entry per USB controller.
    #[serde(rename = "SPUSBDataType", default)]
    buses: Vec<UsbNode>,
}

/// Tokens on the USB bus whose family has no loaded driver. Call this only
/// when the inventory found no tokens: the probe takes about a second and
/// is only useful when something is missing.
pub fn tokens_needing_driver(modules: &[ModuleStatus]) -> Vec<DetectedToken> {
    needing_driver(&probe(), modules)
}

/// Runs `system_profiler` on macOS and parses its USB tree. Empty elsewhere
/// and on any failure, which is logged and never fatal.
fn probe() -> Vec<UsbNode> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }
    match Command::new(SYSTEM_PROFILER)
        .args([USB_DATA_TYPE, "-json"])
        .output()
    {
        Ok(output) if output.status.success() => parse_report(&output.stdout),
        Ok(output) => {
            log::warn!("system_profiler exited with {}", output.status);
            Vec::new()
        }
        Err(error) => {
            log::warn!("could not run system_profiler: {error}");
            Vec::new()
        }
    }
}

/// The controllers in a JSON report, or none when it does not parse.
fn parse_report(json: &[u8]) -> Vec<UsbNode> {
    match serde_json::from_slice::<UsbReport>(json) {
        Ok(report) => report.buses,
        Err(error) => {
            log::warn!("could not parse the USB device list: {error}");
            Vec::new()
        }
    }
}

/// Recognised tokens whose family has no loaded driver, one per family.
fn needing_driver(nodes: &[UsbNode], modules: &[ModuleStatus]) -> Vec<DetectedToken> {
    let mut found = Vec::new();
    collect(nodes, &mut found);
    let mut detected = Vec::<DetectedToken>::new();
    for (known, name) in found {
        if detected.iter().any(|token| token.family == known.family) {
            continue;
        }
        let family_modules: Vec<&ModuleStatus> = modules
            .iter()
            .filter(|status| status.candidate.family.contains(known.driver_key))
            .collect();
        if family_modules.iter().any(|status| status.loaded) {
            continue;
        }
        detected.push(DetectedToken {
            family: known.family,
            name,
            driver_url: known.driver_url,
            driver_present: family_modules.iter().any(|status| status.candidate.exists),
        });
    }
    detected
}

/// Every recognised device in the tree, depth first, with its product string.
fn collect(nodes: &[UsbNode], out: &mut Vec<(&'static KnownToken, String)>) {
    for node in nodes {
        if let Some(known) = recognise(node) {
            out.push((known, node.name.trim().to_owned()));
        }
        collect(&node.items, out);
    }
}

/// The known family a device belongs to, by id first, then by name.
fn recognise(node: &UsbNode) -> Option<&'static KnownToken> {
    let vendor = parse_hex_id(&node.vendor_id);
    let product = parse_hex_id(&node.product_id);
    let haystack = format!("{} {} {}", node.manufacturer, node.name, node.vendor_id).to_lowercase();
    KNOWN_TOKENS.iter().find(|known| {
        let by_id = match (known.vendor_id, vendor) {
            (Some(want), Some(have)) if want == have => {
                known.product_ids.is_empty()
                    || product.is_some_and(|id| known.product_ids.contains(&id))
            }
            _ => false,
        };
        by_id || known.name_hints.iter().any(|hint| haystack.contains(hint))
    })
}

/// Parses `0x096e` or `0x096e  (Vendor Name)`; `None` for anything else.
fn parse_hex_id(text: &str) -> Option<u16> {
    let token = text.split_whitespace().next()?;
    let digits = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .unwrap_or(token);
    u16::from_str_radix(digits, 16).ok()
}

#[cfg(test)]
#[path = "usb_tests.rs"]
mod tests;
