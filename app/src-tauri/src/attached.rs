//! Tokens plugged in over USB and what each still needs, for guided setup.

use serde::Serialize;
use swakshar_token::{DriverState, Inventory, UsbToken, attached_tokens, driver_state};
use tauri::async_runtime::JoinHandle;

/// A token seen on USB.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttachedView {
    /// Maker, when known.
    pub(crate) maker: Option<String>,
    /// Product name the token reports.
    pub(crate) product: Option<String>,
    /// Driver family to install, when the maker is known.
    pub(crate) family: Option<String>,
    /// `ready`, `missing`, `other-architecture`, `failed` or `not-seen`.
    pub(crate) state: &'static str,
    /// The driver's own error, for `failed`.
    pub(crate) detail: Option<String>,
}

/// Starts listing USB tokens on a blocking thread, so it runs while the
/// token thread reads the drivers.
pub(crate) fn scan() -> JoinHandle<Vec<UsbToken>> {
    tauri::async_runtime::spawn_blocking(attached_tokens)
}

/// One view per USB token, judged against what the drivers found.
pub(crate) fn attached_views(usb: &[UsbToken], inventory: &Inventory) -> Vec<AttachedView> {
    usb.iter()
        .map(|token| {
            let (state, detail) = match driver_state(token, inventory) {
                DriverState::Ready => ("ready", None),
                DriverState::Missing => ("missing", None),
                DriverState::OtherArchitecture => ("other-architecture", None),
                DriverState::Failed(error) => ("failed", Some(error)),
                DriverState::NotSeen => ("not-seen", None),
            };
            AttachedView {
                maker: token.maker.clone(),
                product: token.product.clone(),
                family: token.family.map(str::to_owned),
                state,
                detail,
            }
        })
        .collect()
}
