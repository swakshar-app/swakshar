//! What the approval window is shown: masked PANs, and the document
//! fingerprint in full so the user can compare it.

use swakshar_protocol::parse_request;
use swakshar_token::Inventory;

use super::pending_view;
use crate::state::mask_serial;

/// A GST page's origin.
const ORIGIN: &str = "https://services.gst.gov.in";
/// A document fingerprint.
const DIGEST: &str = "3b5d5c3712955042212316173ccf37be800a7fec8bd38c48a1dc0d0a5c1e4f33";

/// A registration signs the PAN itself; the window shows it masked.
#[test]
fn registration_masks_the_pan() {
    let request = parse_request(
        "action=sign\ntobesigned=ABCDE1234F\npanNo=ABCDE1234F\nsigntype=1\nexpirycheck=true\ncertclass=2|3",
    )
    .unwrap();
    let view = pending_view(7, ORIGIN, &request, &Inventory::default(), &[], 100);
    assert_eq!(view.kind, "registration");
    assert_eq!(view.content, "ABCDE****F");
    assert_eq!(view.pan_masked.as_deref(), Some("ABCDE****F"));
    assert_eq!((view.id, view.expires_at), (7, 100));
}

/// A document keeps its full fingerprint and still masks the PAN.
#[test]
fn documents_keep_the_fingerprint() {
    let request = parse_request(&format!(
        "action=sign\ntobesigned={DIGEST}\npanNo=ABCDE1234F\nsigntype=1\nexpirycheck=true\ncertclass=2|3"
    ))
    .unwrap();
    let view = pending_view(8, ORIGIN, &request, &Inventory::default(), &[], 100);
    assert_eq!(view.kind, "document");
    assert_eq!(view.content, DIGEST);
    assert_eq!(view.pan_masked.as_deref(), Some("ABCDE****F"));
}

/// Token serials show only their last four characters.
#[test]
fn serials_keep_the_last_four() {
    assert_eq!(mask_serial("1234567890"), "****7890");
    assert_eq!(mask_serial("12"), "****12");
}
