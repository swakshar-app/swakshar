//! Vector appearance ported from the owner's approved DSC CLI stamp.

use serde::Deserialize;
use time::OffsetDateTime;

use crate::error::{CommandError, CommandResult};
use crate::pdf_types::PdfPlacement;

/// One source of geometry and Helvetica advances for native output and preview.
const STYLE: &str = include_str!("../../src/pdf/stamp-layout.json");

/// Approved typography and tick geometry, shared with the preview.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Style {
    heading: String,
    head_size: f64,
    body_size: f64,
    leading: f64,
    padding: f64,
    tick: Vec<[f64; 2]>,
    widths: Vec<u16>,
}

/// UTC is explicit and does not depend on a bundled runtime's timezone database.
pub(crate) fn date(now: OffsetDateTime) -> String {
    format!(
        "{:04}.{:02}.{:02} {:02}:{:02}:{:02} UTC",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Builds a tiny, uncompressed vector PDF without loading fonts or another process.
pub(crate) fn render(
    signer: &str,
    placement: &PdfPlacement,
    timestamp: &str,
) -> CommandResult<Vec<u8>> {
    let style: Style =
        serde_json::from_str(STYLE).map_err(|error| CommandError::Message(error.to_string()))?;
    let [left, bottom, right, top] = placement.rect;
    let w = f64::from(right - left);
    let h = f64::from(top - bottom);
    let (scale, lines) = layout(&style, w, h, signer, timestamp);
    let path = style
        .tick
        .iter()
        .enumerate()
        .map(|(index, [x, y])| {
            format!(
                "{:.2} {:.2} {}",
                w * (0.4 + x * 0.3),
                h * (0.1 + y * 0.72),
                if index == 0 { "m" } else { "l" }
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut content = format!("0 g\nq 0.10 0.55 0.15 rg 0 0 0 RG 1.4 w 1 j {path} h B Q\n");
    let top = h - (style.padding + style.head_size * 0.8) * scale;
    text(
        &mut content,
        &style.heading,
        style.head_size * scale,
        style.padding * scale,
        top,
    );
    let mut y = top - (style.head_size * 0.35 + style.leading) * scale;
    for line in lines {
        text(
            &mut content,
            &line,
            style.body_size * scale,
            style.padding * scale,
            y,
        );
        y -= style.leading * scale;
    }
    Ok(document(w, h, &content))
}

/// Reduces typography only when the approved fixed sizes would clip the box.
fn layout(style: &Style, w: f64, h: f64, signer: &str, timestamp: &str) -> (f64, Vec<String>) {
    let mut scale = 1_f64.min(w / 100.0).min(h / 60.0);
    let mut lines = Vec::new();
    for _ in 0..8 {
        let width = w - 2.0 * style.padding * scale;
        lines = wrap(
            style,
            &format!("Digitally signed by {signer}"),
            style.body_size * scale,
            width,
        );
        lines.extend(wrap(
            style,
            &format!("Date: {timestamp}"),
            style.body_size * scale,
            width,
        ));
        let needed =
            (2.0 * style.padding + style.head_size * 1.15 + lines.len() as f64 * style.leading)
                * scale;
        if needed <= h {
            break;
        }
        scale *= h / needed;
    }
    (scale, lines)
}

/// Measures the standard font without platform-specific font discovery.
fn width(style: &Style, text: &str, size: f64) -> f64 {
    text.chars()
        .map(|character| {
            let index = (character as usize).wrapping_sub(32);
            f64::from(style.widths.get(index).copied().unwrap_or(556))
        })
        .sum::<f64>()
        * size
        / 1000.0
}

/// Wraps long names by Helvetica advance, including oversized single words.
fn wrap(style: &Style, text: &str, size: f64, maximum: f64) -> Vec<String> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut lines = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if !current.is_empty() && width(style, &format!("{current}{character}"), size) > maximum {
            if let Some(space) = current.rfind(' ').filter(|space| *space > 0) {
                lines.push(current[..space].to_owned());
                current = current[space + 1..].to_owned();
            } else {
                lines.push(std::mem::take(&mut current));
            }
        }
        current.push(character);
    }
    if !current.trim().is_empty() {
        lines.push(current.trim().to_owned());
    }
    lines
}

/// Escapes PDF literal text and writes a standard-font text operation.
fn text(content: &mut String, value: &str, size: f64, x: f64, y: f64) {
    let escaped = value
        .chars()
        .map(|character| match character {
            '\\' => "\\\\".to_owned(),
            '(' => "\\(".to_owned(),
            ')' => "\\)".to_owned(),
            ' '..='~' => character.to_string(),
            _ => format!("\\{:03o}", win_ansi(character)),
        })
        .collect::<String>();
    content.push_str(&format!(
        "BT /F1 {size:.4} Tf {x:.4} {y:.4} Td ({escaped}) Tj ET\n"
    ));
}

/// Matches the approved Helvetica WinAnsi encoding for common Latin names.
fn win_ansi(character: char) -> u32 {
    match character {
        '\u{a0}'..='\u{ff}' => character as u32,
        '\u{2019}' => 146,
        '\u{2018}' => 145,
        '\u{20ac}' => 128,
        _ => u32::from(b'?'),
    }
}

/// Writes byte-accurate stream lengths and cross-reference offsets.
fn document(w: f64, h: f64, content: &str) -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{content}endstream",
            content.len()
        ),
    ];
    let mut out = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.push_str(&format!("{} 0 obj\n{object}\nendobj\n", index + 1));
    }
    let xref = out.len();
    out.push_str("xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets {
        out.push_str(&format!("{offset:010} 00000 n \n"));
    }
    out.push_str(&format!(
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    ));
    out.into_bytes()
}

#[cfg(test)]
#[path = "pdf_stamp_tests.rs"]
mod tests;
