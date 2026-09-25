//! PAN validation and masking.

/// Returns true when `pan` has the shape of an Indian PAN: five uppercase
/// letters, four digits, one uppercase letter.
pub fn is_valid_pan(pan: &str) -> bool {
    pan.len() == 10
        && pan.char_indices().all(|(index, ch)| match index {
            0..=4 | 9 => ch.is_ascii_uppercase(),
            _ => ch.is_ascii_digit(),
        })
}

/// Masks a PAN for display and logs: `ABCDE1234F` becomes `ABCDE****F`.
/// Anything that is not a valid PAN is masked completely.
pub fn mask_pan(pan: &str) -> String {
    if !is_valid_pan(pan) {
        return "*".repeat(pan.chars().count().min(10));
    }
    let head = pan.get(..5).unwrap_or_default();
    let tail = pan.get(9..).unwrap_or_default();
    format!("{head}****{tail}")
}

#[cfg(test)]
mod tests {
    use super::{is_valid_pan, mask_pan};

    /// Accepts well-formed PANs and rejects near misses.
    #[test]
    fn validates_pan_shape() {
        assert!(is_valid_pan("ABCDE1234F"));
        assert!(!is_valid_pan("abcde1234f"));
        assert!(!is_valid_pan("ABCD01234F"));
        assert!(!is_valid_pan("ABCDE12345"));
        assert!(!is_valid_pan("ABCDE1234"));
        assert!(!is_valid_pan(""));
    }

    /// Keeps the first five and the last character of a valid PAN.
    #[test]
    fn masks_pan() {
        assert_eq!(mask_pan("ABCDE1234F"), "ABCDE****F");
        assert_eq!(mask_pan("oops"), "****");
    }
}
