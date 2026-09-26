//! Commands the webview may invoke. `build.rs` lists every name, and each
//! window's capability allows only its own subset.

pub(crate) mod approve;
pub(crate) mod doctor;
pub(crate) mod drivers;
pub(crate) mod history;
pub(crate) mod overview;
pub(crate) mod settings;
pub(crate) mod trust;
