//! Tokens seen on USB, found without any vendor driver, so the app can name
//! a plugged-in token and say which driver it still needs.

use nusb::MaybeFuture as _;

use crate::macho::ArchSupport;
use crate::types::{Inventory, ModuleStatus};

/// USB interface class for smart cards (CCID), which most DSC tokens use.
const CLASS_SMART_CARD: u8 = 0x0B;
/// Hypersecu's download page for tokens sold in India, HYP2003 included.
const HYPERSECU_INDIA_DOWNLOADS: &str = "https://hypersecu.com/copy-of-downloads-for-india";

/// A token maker by USB vendor ID, and the driver families in
/// [`crate::candidate_modules`] that serve its tokens.
struct KnownVendor {
    /// USB vendor ID.
    vendor_id: u16,
    /// Maker, for display.
    maker: &'static str,
    /// Driver family shown to the user.
    family: &'static str,
    /// Families of known driver paths that work with these tokens.
    driver_families: &'static [&'static str],
    /// The maker's official driver download page, when one is public.
    driver_page: Option<&'static str>,
}

/// Families served by Feitian's ePass2003 driver, also sold as HYP2003.
const EPASS2003_DRIVERS: &[&str] = &[
    "ePass2003 / HYP2003",
    "ePass2003",
    "OpenSC",
    "OpenSC (Homebrew)",
];

/// Makers whose USB vendor IDs are public (`usb.ids`).
const KNOWN_VENDORS: &[KnownVendor] = &[
    KnownVendor {
        vendor_id: 0x096E,
        maker: "Feitian",
        family: "ePass2003 / HYP2003",
        driver_families: EPASS2003_DRIVERS,
        driver_page: None,
    },
    KnownVendor {
        vendor_id: 0x2CCF,
        maker: "Hypersecu",
        family: "ePass2003 / HYP2003",
        driver_families: EPASS2003_DRIVERS,
        driver_page: Some(HYPERSECU_INDIA_DOWNLOADS),
    },
    KnownVendor {
        vendor_id: 0x0529,
        maker: "SafeNet",
        family: "SafeNet eToken",
        driver_families: &["SafeNet eToken"],
        driver_page: None,
    },
];

/// A USB device that looks like a DSC token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbToken {
    /// USB vendor ID.
    pub vendor_id: u16,
    /// USB product ID.
    pub product_id: u16,
    /// Maker, from our table or the device's own string.
    pub maker: Option<String>,
    /// Product name the device reports.
    pub product: Option<String>,
    /// Driver family to install, when the maker is known.
    pub family: Option<&'static str>,
    /// Driver families that serve this token; empty when unknown.
    pub driver_families: &'static [&'static str],
    /// The maker's official driver download page, when one is public.
    pub driver_page: Option<&'static str>,
}

/// USB devices that are known token models or smart card devices. Returns an
/// empty list when USB cannot be enumerated.
pub fn attached_tokens() -> Vec<UsbToken> {
    match nusb::list_devices().wait() {
        Ok(devices) => devices.filter_map(|device| classify(&device)).collect(),
        Err(error) => {
            log::debug!("could not list USB devices: {error}");
            Vec::new()
        }
    }
}

/// The official driver download page of the maker with `vendor_id`, when
/// one is public. Looked up here so the webview never supplies a URL.
pub fn driver_page(vendor_id: u16) -> Option<&'static str> {
    KNOWN_VENDORS
        .iter()
        .find(|vendor| vendor.vendor_id == vendor_id)
        .and_then(|vendor| vendor.driver_page)
}

/// A token description for `device`, or `None` when it is not a token.
fn classify(device: &nusb::DeviceInfo) -> Option<UsbToken> {
    let known = KNOWN_VENDORS
        .iter()
        .find(|vendor| vendor.vendor_id == device.vendor_id());
    let smart_card = device.class() == CLASS_SMART_CARD
        || device
            .interfaces()
            .any(|interface| interface.class() == CLASS_SMART_CARD);
    if known.is_none() && !smart_card {
        return None;
    }
    Some(UsbToken {
        vendor_id: device.vendor_id(),
        product_id: device.product_id(),
        maker: known
            .map(|vendor| vendor.maker.to_owned())
            .or_else(|| device.manufacturer_string().map(str::to_owned)),
        product: device.product_string().map(str::to_owned),
        family: known.map(|vendor| vendor.family),
        driver_families: known.map_or(&[], |vendor| vendor.driver_families),
        driver_page: known.and_then(|vendor| vendor.driver_page),
    })
}

/// What a plugged-in token still needs before it can sign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverState {
    /// A driver loaded and the token is visible.
    Ready,
    /// No driver for this token is installed.
    Missing,
    /// The only drivers found are built for the other processor.
    OtherArchitecture,
    /// A driver is installed but did not start.
    Failed(String),
    /// A driver started but does not see the token.
    NotSeen,
}

/// Decides [`DriverState`] for `token` from what the token thread found.
/// User-added drivers count for every token, since their family is unknown.
pub fn driver_state(token: &UsbToken, inventory: &Inventory) -> DriverState {
    let drivers: Vec<&ModuleStatus> = inventory
        .modules
        .iter()
        .filter(|status| {
            status.candidate.exists
                && (status.candidate.user_added
                    || token.driver_families.is_empty()
                    || token
                        .driver_families
                        .contains(&status.candidate.family.as_str()))
        })
        .collect();
    if drivers.is_empty() {
        return DriverState::Missing;
    }
    let loaded: Vec<&&ModuleStatus> = drivers.iter().filter(|status| status.loaded).collect();
    if !loaded.is_empty() {
        let visible = inventory.tokens.iter().any(|entry| {
            loaded
                .iter()
                .any(|status| status.candidate.path == entry.module)
        });
        return if visible {
            DriverState::Ready
        } else {
            DriverState::NotSeen
        };
    }
    if drivers
        .iter()
        .all(|status| status.candidate.arch == ArchSupport::Incompatible)
    {
        return DriverState::OtherArchitecture;
    }
    DriverState::Failed(
        drivers
            .iter()
            .find_map(|status| status.error.clone())
            .unwrap_or_else(|| "The driver did not start.".to_owned()),
    )
}

#[cfg(test)]
#[path = "usb_tests.rs"]
mod tests;
