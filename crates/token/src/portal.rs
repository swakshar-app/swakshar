//! Glue between a portal request and the token: filters in, reply frame out.

use swakshar_cms::to_portal_base64;
use swakshar_protocol::{
    SignRequest, SuccessReply, format_date_utc, format_datetime_ist, render_success,
};

use crate::select::Criteria;
use crate::types::SignedOutput;

/// Certificate filters taken from a portal request.
pub fn criteria_for(request: &SignRequest, now: i64) -> Criteria<'_> {
    Criteria {
        pan: request.pan.as_deref(),
        expiry_check: request.expiry_check,
        classes: &request.cert_classes,
        issuer_name: request.issuer_name.as_deref(),
        now,
    }
}

/// The success frame for a finished signature. `signing_time` is the same
/// Unix second written into the CMS.
pub fn portal_reply(output: &SignedOutput, request: &SignRequest, signing_time: i64) -> String {
    let signature = to_portal_base64(&output.cms_der);
    let summary = &output.summary;
    render_success(&SuccessReply {
        signature: &signature,
        serial_number: &summary.serial_decimal,
        common_name: &summary.subject_cn,
        issuer_name: &summary.issuer_cn,
        issued_date: &format_date_utc(summary.not_before),
        expiry_date: &format_date_utc(summary.not_after),
        cert_class: summary.class_label(),
        signing_time: &format_datetime_ist(signing_time),
        unique_id: request.unique_id.as_deref().unwrap_or_default(),
    })
}
