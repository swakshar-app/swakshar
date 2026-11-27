//! Guardrails for PDF placement and private workspace publication.

use super::pdf_files::PdfWorkspace;
use super::pdf_files::{MAX_PDF_BYTES, validate_pdf};
use super::pdf_types::PdfPlacement;

/// An old editor's delayed cancellation cannot stop a newer document's job.
#[test]
fn cancellation_checks_document_ownership() {
    use std::sync::atomic::Ordering;
    let state = super::pdf_types::PdfState::default();
    state.active_document_id.store(2, Ordering::Release);
    super::pdf_engine::cancel_document(&state, 1);
    assert!(!state.cancelled.load(Ordering::Acquire));
    super::pdf_engine::cancel_document(&state, 2);
    assert!(state.cancelled.load(Ordering::Acquire));
}

/// Rejecting upstream-trimmed PINs prevents a silent wrong-PIN token attempt.
#[test]
fn pdf_pin_preserves_exact_input() {
    for pin in ["", " 1234", "1234\n", "\u{001c}1234", "1234\u{00a0}"] {
        assert!(super::pdf_engine::validate_pin(&secrecy::SecretString::from(pin)).is_err());
    }
    assert!(super::pdf_engine::validate_pin(&secrecy::SecretString::from("12 34")).is_ok());
}

/// A destination appearing after the save dialog is never overwritten.
#[test]
fn publication_preserves_a_concurrent_file() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("swakshar-pdf-test-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let destination = root.join("signed.pdf");
    let workspace = PdfWorkspace::create(&destination, 1).unwrap();
    let staging = workspace.directory.clone();
    workspace
        .write("signed.pdf", b"%PDF-1.7\nnew signed bytes")
        .unwrap();
    std::fs::write(&destination, b"existing document").unwrap();
    assert!(workspace.publish(&destination).is_err());
    assert_eq!(std::fs::read(&destination).unwrap(), b"existing document");
    drop(workspace);
    assert!(!staging.exists());
    std::fs::remove_dir_all(root).unwrap();
}

/// A regular page and rectangle are accepted, including offset page coordinates.
#[test]
fn placement_accepts_offset_coordinates() {
    let placement = PdfPlacement {
        page: 2,
        rect: [-100, 10, 70, 80],
    };
    assert!(placement.validate().is_ok());
}

/// A bad rectangle or page is rejected before any token is accessed.
#[test]
fn invalid_placements_are_rejected() {
    for (page, rect) in [
        (0, [0, 0, 170, 70]),
        (1, [10, 0, 0, 70]),
        (1, [0, 0, 4, 70]),
        (1, [i32::MIN, 0, i32::MAX, 70]),
    ] {
        assert!(PdfPlacement { page, rect }.validate().is_err());
    }
}

/// Non-PDF files and oversized inputs cannot reach either PDF engine.
#[test]
fn pdf_input_is_bounded() {
    assert!(validate_pdf(b"%PDF-1.7\n", 9).is_ok());
    assert!(validate_pdf(b"not a PDF", 9).is_err());
    assert!(validate_pdf(b"%PDF-1.7\n", MAX_PDF_BYTES + 1).is_err());
}
