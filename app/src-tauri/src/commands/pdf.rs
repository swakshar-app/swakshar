//! Native pickers and explicit signing actions for the main window.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use secrecy::SecretString;
use tauri::{AppHandle, Manager as _};
use tauri_plugin_dialog::DialogExt as _;
use tokio::sync::oneshot;

use crate::error::{CommandError, CommandResult};
use crate::pdf_engine;
use crate::pdf_files::{PdfWorkspace, read_pdf};
use crate::pdf_types::{PdfDocument, PdfSignRequest, PdfState, SelectedPdf};
use crate::state::{AppState, lock, unix_now};

/// Picks one local PDF and stores the exact bytes that will be previewed and signed.
#[tauri::command]
pub(crate) async fn choose_pdf(app: AppHandle) -> CommandResult<Option<PdfDocument>> {
    let (sender, receiver) = oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose a PDF to sign")
        .add_filter("PDF", &["pdf"])
        .pick_file(move |picked| {
            let _ = sender.send(picked);
        });
    let Some(picked) = receiver.await.ok().flatten() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|error| CommandError::Message(error.to_string()))?;
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = read_pdf(&path)?;
        let name = path
            .file_name()
            .ok_or_else(|| CommandError::Message("Choose a PDF file.".to_owned()))?
            .to_string_lossy()
            .into_owned();
        let id = app.state::<AppState>().next_id();
        let size = bytes.len();
        *lock(&app.state::<PdfState>().selected) = Some(SelectedPdf {
            id,
            name: name.clone(),
            bytes: Arc::new(bytes),
        });
        Ok(Some(PdfDocument { id, name, size }))
    })
    .await
    .map_err(|error| CommandError::Message(error.to_string()))?
}

/// Transfers only the picker-selected snapshot as binary, avoiding JSON byte arrays.
#[tauri::command]
pub(crate) async fn get_pdf_bytes(
    app: AppHandle,
    document_id: u64,
) -> CommandResult<tauri::ipc::Response> {
    let selected = lock(&app.state::<PdfState>().selected)
        .as_ref()
        .filter(|selected| selected.id == document_id)
        .cloned()
        .ok_or_else(|| {
            CommandError::Message("Choose the PDF again before previewing.".to_owned())
        })?;
    Ok(tauri::ipc::Response::new(selected.bytes.as_ref().clone()))
}

/// Holds the existing single-signature slot through dialogs, crypto and publication.
struct SigningSlot(AppHandle);

impl SigningSlot {
    /// Prevents GST and PDF signatures from running together.
    fn take(app: &AppHandle, document_id: u64) -> CommandResult<Self> {
        app.state::<AppState>()
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                CommandError::Message(
                    "Another signing request is active. Finish it first.".to_owned(),
                )
            })?;
        let state = app.state::<PdfState>();
        let _child = lock(&state.child);
        state
            .active_document_id
            .store(document_id, Ordering::Release);
        state.cancelled.store(false, Ordering::Release);
        Ok(Self(app.clone()))
    }
}

impl Drop for SigningSlot {
    /// Releases the slot on cancellation, failure and success.
    fn drop(&mut self) {
        let state = self.0.state::<PdfState>();
        let _child = lock(&state.child);
        state.active_document_id.store(0, Ordering::Release);
        self.0
            .state::<AppState>()
            .busy
            .store(false, Ordering::Release);
    }
}

/// Signs the immutable preview into a new, owner-selected output file.
#[tauri::command]
pub(crate) async fn sign_pdf(
    app: AppHandle,
    request: PdfSignRequest,
    pin: Option<String>,
) -> CommandResult<Option<String>> {
    let pin = pin.map(SecretString::from);
    if let Some(pin) = &pin {
        pdf_engine::validate_pin(pin)?;
    }
    request.placement.validate()?;
    let slot = SigningSlot::take(&app, request.document_id)?;
    let selected = lock(&app.state::<PdfState>().selected)
        .as_ref()
        .filter(|selected| selected.id == request.document_id)
        .cloned()
        .ok_or_else(|| CommandError::Message("Choose the PDF again before signing.".to_owned()))?;
    let (sender, receiver) = oneshot::channel();
    let suggested = format!(
        "{}-signed.pdf",
        selected.name.strip_suffix(".pdf").unwrap_or(&selected.name)
    );
    app.dialog()
        .file()
        .set_title("Save signed PDF as a new file")
        .set_file_name(suggested)
        .add_filter("PDF", &["pdf"])
        .save_file(move |picked| {
            let _ = sender.send(picked);
        });
    let Some(picked) = receiver.await.ok().flatten() else {
        return Ok(None);
    };
    let destination = picked
        .into_path()
        .map_err(|error| CommandError::Message(error.to_string()))?;
    let (mut profile, signer) = signing_profile(&app, &request, pin.is_some()).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let _slot = slot;
        let workspace = PdfWorkspace::create(&destination, app.state::<AppState>().next_id())?;
        workspace.write("input.pdf", &selected.bytes)?;
        let timestamp = crate::pdf_stamp::date(time::OffsetDateTime::now_utc());
        let stamp = crate::pdf_stamp::render(&signer, &request.placement, &timestamp)?;
        let background = workspace.write("stamp.pdf", &stamp)?;
        profile["stamp-styles"]["swakshar"]["background"] = serde_json::json!(background);
        workspace.write("profile.json", profile.to_string().as_bytes())?;
        pdf_engine::sign(
            &app,
            &workspace,
            &request.placement,
            app.state::<AppState>().next_id(),
            pin,
        )?;
        if app.state::<PdfState>().cancelled.load(Ordering::Acquire) {
            return Err(CommandError::Message("PDF signing cancelled.".to_owned()));
        }
        workspace.publish(&destination)?;
        Ok(Some(destination.display().to_string()))
    })
    .await
    .map_err(|error| CommandError::Message(error.to_string()))?
}

/// Resolves fresh token flags after the save dialog and before any PIN is submitted.
async fn signing_profile(
    app: &AppHandle,
    request: &PdfSignRequest,
    has_pin: bool,
) -> CommandResult<(serde_json::Value, String)> {
    let state = app.state::<AppState>();
    let inventory = state
        .read_inventory(state.settings().module_paths())
        .await?;
    let (token, certificate) = inventory
        .tokens
        .iter()
        .flat_map(|token| {
            token
                .certificates
                .iter()
                .map(move |certificate| (token, certificate))
        })
        .find(|(token, certificate)| {
            pdf_engine::certificate_id(token, certificate) == request.fingerprint
        })
        .ok_or_else(|| {
            CommandError::Message(
                "The selected certificate is no longer connected. Refresh certificates.".to_owned(),
            )
        })?;
    if token.pin.locked
        || !certificate.summary.valid_at(unix_now())
        || !certificate.summary.signing
        || certificate.id.is_empty()
    {
        return Err(CommandError::Message(
            "This certificate is unavailable or the PIN is locked. Refresh certificates."
                .to_owned(),
        ));
    }
    if token.pin.final_try && !request.final_try_acknowledged {
        return Err(CommandError::Message("One more wrong PIN will lock this token. Refresh certificates and read the warning before trying again.".to_owned()));
    }
    if !token.pin.protected_path && !has_pin {
        return Err(CommandError::Message("Enter the token PIN.".to_owned()));
    }
    Ok((
        pdf_engine::profile(token, certificate),
        certificate.summary.subject_cn.clone(),
    ))
}

/// Cancels the active child process without exposing a general process API.
#[tauri::command]
pub(crate) fn cancel_pdf_signing(app: AppHandle, document_id: u64) {
    pdf_engine::cancel_document(&app.state::<PdfState>(), document_id);
}
