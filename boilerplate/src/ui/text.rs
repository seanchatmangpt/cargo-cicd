//! String utilities for terminal layout.
//!
//! Always use [`display_width`] rather than `.len()` when computing column
//! widths for table alignment — `.len()` counts bytes, not display columns.

/// Compute the display width of a string in terminal columns.
///
/// Multi-byte Unicode characters may occupy 1 or 2 display columns.
/// ANSI escape sequences contribute zero display columns.
/// ASCII characters each contribute exactly 1 column.
///
/// This is a best-effort implementation without external dependencies.
/// For full Unicode width support, consider the `unicode-width` crate.
pub fn display_width(s: &str) -> usize {
    let mut width = 0usize;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ANSI escape sequence: ESC [ ... final-byte
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for inner in chars.by_ref() {
                    // Final byte is in 0x40–0x7E range.
                    if inner.is_ascii_alphabetic() || inner == 'm' {
                        break;
                    }
                }
            }
            continue;
        }

        // Rough CJK/wide character detection: code points in common wide ranges.
        let cp = c as u32;
        let is_wide = matches!(cp,
            0x1100..=0x115F  // Hangul Jamo
            | 0x2E80..=0x2EFF  // CJK Radicals
            | 0x2F00..=0x2FDF
            | 0x2FF0..=0x2FFF
            | 0x3000..=0x303F  // CJK Symbols
            | 0x3040..=0x309F  // Hiragana
            | 0x30A0..=0x30FF  // Katakana
            | 0x3100..=0x312F
            | 0x3130..=0x318F
            | 0x3190..=0x319F
            | 0x31A0..=0x31BF
            | 0x31C0..=0x31EF
            | 0x31F0..=0x31FF
            | 0x3200..=0x32FF
            | 0x3300..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF  // CJK Unified Ideographs
            | 0xA000..=0xA48F
            | 0xA490..=0xA4CF
            | 0xAC00..=0xD7AF  // Hangul Syllables
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE1F
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60  // Fullwidth Forms
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1F9FF  // Emoji
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2CEAF
        );

        width += if is_wide { 2 } else { 1 };
    }

    width
}

/// Truncate a string to at most `max_columns` display columns.
///
/// If truncation occurs, `…` is appended (consuming one column).
/// If `max_columns` is 0, returns an empty string.
pub fn truncate(s: &str, max_columns: usize) -> String {
    if max_columns == 0 {
        return String::new();
    }

    let w = display_width(s);
    if w <= max_columns {
        return s.to_owned();
    }

    // Find the cut point.
    let ellipsis = "…";
    let target = max_columns.saturating_sub(1);
    let mut cols = 0usize;
    let mut byte_pos = 0;

    for c in s.chars() {
        let cw = if (c as u32) > 0x7F { 2 } else { 1 };
        if cols + cw > target {
            break;
        }
        cols += cw;
        byte_pos += c.len_utf8();
    }

    format!("{}{ellipsis}", &s[..byte_pos])
}

/// Pad a string on the right with spaces to reach `width` display columns.
pub fn pad_right(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_owned();
    }
    format!("{}{}", s, " ".repeat(width - w))
}

/// Pad a string on the left with spaces to reach `width` display columns.
pub fn pad_left(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_owned();
    }
    format!("{}{}", " ".repeat(width - w), s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width_equals_char_count() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn ansi_escapes_are_zero_width() {
        let with_color = "\x1b[32mhello\x1b[0m";
        assert_eq!(display_width(with_color), 5);
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_adds_ellipsis() {
        let result = truncate("hello world", 7);
        assert!(result.ends_with('…'));
        assert!(display_width(&result) <= 7);
    }

    #[test]
    fn pad_right_adds_spaces() {
        let result = pad_right("hi", 6);
        assert_eq!(result, "hi    ");
        assert_eq!(display_width(&result), 6);
    }

    #[test]
    fn pad_left_adds_spaces() {
        let result = pad_left("hi", 6);
        assert_eq!(result, "    hi");
        assert_eq!(display_width(&result), 6);
    }

    #[test]
    fn pad_longer_than_width_unchanged() {
        assert_eq!(pad_right("hello world", 3), "hello world");
    }
}
