//! PDF selections and validated placement data, independent of Tauri IPC.

use std::process::Child;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::error::{CommandError, CommandResult};

/// Smallest usable visible stamp, in PDF points.
const MIN_BOX_SIZE: i64 = 12;
/// Bound extreme coordinates before passing them to the PDF engine.
const MAX_COORDINATE: i64 = 100_000;

/// Selected input kept as an immutable snapshot, so signing matches the preview.
#[derive(Clone)]
pub(crate) struct SelectedPdf {
    /// Opaque selection id.
    pub(crate) id: u64,
    /// Basename only.
    pub(crate) name: String,
    /// Original bytes, never re-read from an editable path.
    pub(crate) bytes: Arc<Vec<u8>>,
}

/// Main-window selection and cancellable signing subprocess.
#[derive(Default)]
pub(crate) struct PdfState {
    /// Prevents a late editor-unmount message cancelling a newer document's job.
    pub(crate) active_document_id: AtomicU64,
    /// Covers cancellation during a save dialog as well as while the child runs.
    pub(crate) cancelled: AtomicBool,
    /// Only a native picker can replace the input snapshot.
    pub(crate) selected: Mutex<Option<SelectedPdf>>,
    /// Present only while the bundled engine is running.
    pub(crate) child: Mutex<Option<Child>>,
}

/// Preview response, with no arbitrary filesystem read command.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfDocument {
    /// Selection id required by the signing command.
    pub(crate) id: u64,
    /// Basename displayed by the UI.
    pub(crate) name: String,
    /// PDF bytes for the local preview.
    pub(crate) bytes: Vec<u8>,
}

/// One page and rectangle, in unrotated PDF coordinates.
#[derive(Clone, Deserialize)]
pub(crate) struct PdfPlacement {
    /// One-based page number, matching pyHanko's CLI.
    pub(crate) page: u32,
    /// Left, bottom, right, top.
    pub(crate) rect: [i32; 4],
}

impl PdfPlacement {
    /// Rejects empty, inverted and extreme coordinates before accessing a token.
    pub(crate) fn validate(&self) -> CommandResult<()> {
        let [left, bottom, right, top] = self.rect.map(i64::from);
        if self.page == 0
            || self.page > 100_000
            || right - left < MIN_BOX_SIZE
            || top - bottom < MIN_BOX_SIZE
            || [left, bottom, right, top]
                .iter()
                .any(|value| value.abs() > MAX_COORDINATE)
        {
            return Err(CommandError::Message(
                "Choose a page and draw a larger signature box inside it.".to_owned(),
            ));
        }
        Ok(())
    }

    /// A unique field name avoids replacing an existing signature field.
    pub(crate) fn field_spec(&self, id: u64) -> String {
        let [left, bottom, right, top] = self.rect;
        format!("{}/{left},{bottom},{right},{top}/Swakshar{id}", self.page)
    }
}

/// Signing parameters from an explicit main-window action.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfSignRequest {
    /// The user saw the fresh last-try warning before explicitly pressing Sign.
    pub(crate) final_try_acknowledged: bool,
    /// Must match the current native-picker selection.
    pub(crate) document_id: u64,
    /// Opaque certificate and token fingerprint, resolved against a fresh inventory.
    pub(crate) fingerprint: String,
    /// Visible signature placement.
    pub(crate) placement: PdfPlacement,
}
