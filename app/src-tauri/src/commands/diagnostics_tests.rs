//! The report's privacy rules and log trimming.

use super::{redact, tail_lines};

/// Every occurrence of the home folder becomes `~`, hiding the user name.
#[test]
fn hides_the_home_folder() {
    let text = "driver /Users/someone/lib/x.dylib\nlog /Users/someone/Library/Logs";
    assert_eq!(
        redact(text, Some("/Users/someone")),
        "driver ~/lib/x.dylib\nlog ~/Library/Logs"
    );
}

/// Without a home folder, or with an empty one, the text is unchanged.
#[test]
fn leaves_text_alone_without_a_home() {
    assert_eq!(redact("/Users/x", None), "/Users/x");
    assert_eq!(redact("/Users/x", Some("")), "/Users/x");
}

/// Only the last lines are kept, each indented.
#[test]
fn keeps_the_last_lines() {
    assert_eq!(tail_lines("a\nb\nc\nd", 2), "  c\n  d\n");
    assert_eq!(tail_lines("a", 5), "  a\n");
    assert_eq!(tail_lines("", 5), "");
}
