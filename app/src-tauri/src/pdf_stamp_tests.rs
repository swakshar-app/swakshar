//! Regression coverage for the approved vector appearance and its PDF encoding.

use super::*;

/// PDF cross-reference entries must point to their corresponding object headers.
#[test]
fn vector_pdf_has_correct_offsets_and_stream_length() {
    let placement = PdfPlacement {
        page: 1,
        rect: [0, 0, 170, 70],
    };
    let bytes = render(
        "Example (Signer) \\ Name",
        &placement,
        "2026.11.27 12:00:00 UTC",
    )
    .unwrap();
    let pdf = String::from_utf8(bytes).unwrap();
    assert!(pdf.contains("/BaseFont /Helvetica"));
    assert!(pdf.contains("(Signature valid) Tj"));
    assert!(pdf.contains("0.10 0.55 0.15 rg"));
    assert!(pdf.contains("\\(Signer\\)"));
    assert!(pdf.contains("\\\\"));
    let xref = pdf.rfind("\nxref\n").unwrap() + 1;
    assert!(pdf.contains(&format!("startxref\n{xref}\n")));
    for (index, line) in pdf[xref..].lines().skip(3).take(5).enumerate() {
        let offset: usize = line[..10].parse().unwrap();
        assert!(pdf[offset..].starts_with(&format!("{} 0 obj", index + 1)));
    }
    let stream = pdf
        .split_once("stream\n")
        .unwrap()
        .1
        .split_once("endstream")
        .unwrap()
        .0;
    assert!(pdf.contains(&format!("/Length {} >>", stream.len())));
}

/// Small placements must fit the full heading, signer and date rather than clip.
#[test]
fn typography_fits_small_and_normal_boxes() {
    let style: Style = serde_json::from_str(STYLE).unwrap();
    assert_eq!(style.widths.len(), 95);
    for (w, h) in [(12.0, 12.0), (170.0, 70.0), (100.0, 24.0)] {
        let (scale, lines) = layout(
            &style,
            w,
            h,
            "Example Long Certificate Holder Name",
            "2026.11.27 12:00:00 UTC",
        );
        let needed =
            (2.0 * style.padding + style.head_size * 1.15 + lines.len() as f64 * style.leading)
                * scale;
        assert!(needed <= h + 0.01);
        assert!(
            lines
                .iter()
                .all(|line| width(&style, line, style.body_size * scale) <= w)
        );
    }
}

/// UTC is displayed explicitly and is independent of machine timezone settings.
#[test]
fn timestamp_is_explicit() {
    assert_eq!(date(OffsetDateTime::UNIX_EPOCH), "1970.01.01 00:00:00 UTC");
}
