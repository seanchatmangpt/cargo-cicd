//! Zero-dependency text measurement and alignment.
//!
//! All widths are measured in display columns, ignoring ANSI SGR escape
//! sequences so that styled and unstyled text align identically. Width is
//! approximated as one column per `char` — correct for ASCII, Latin, and
//! box-drawing glyphs; East-Asian wide characters are not special-cased.

/// Horizontal alignment for padded cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Remove ANSI SGR escape sequences (`ESC [ ... <final>`) from `s`.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // Consume parameter/intermediate bytes until a final byte (0x40..=0x7e).
                for d in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&d) {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Display width of `s` in columns, ignoring ANSI escapes.
pub fn display_width(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

/// Truncate `s` to at most `max` display columns, appending `ellipsis` when cut.
///
/// ANSI-naive: escapes are stripped before measuring, so the result is plain
/// text. Style spans first, then truncate, if you need both.
pub fn truncate(s: &str, max: usize, ellipsis: &str) -> String {
    if display_width(s) <= max {
        return s.to_string();
    }
    let ell_w = display_width(ellipsis);
    if max <= ell_w {
        return ellipsis.chars().take(max).collect();
    }
    let keep = max - ell_w;
    let plain = strip_ansi(s);
    let mut out: String = plain.chars().take(keep).collect();
    out.push_str(ellipsis);
    out
}

/// Pad `s` to `width` display columns using `align`. Returns `s` unchanged when
/// it is already at least `width` columns wide. ANSI escapes in `s` are
/// preserved; only the visible width is padded.
pub fn pad(s: &str, width: usize, align: Align) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_string();
    }
    let pad = width - w;
    match align {
        Align::Left => format!("{}{}", s, " ".repeat(pad)),
        Align::Right => format!("{}{}", " ".repeat(pad), s),
        Align::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
        }
    }
}

/// Repeat single-column `unit` to fill exactly `width` columns.
pub fn fill(unit: &str, width: usize) -> String {
    if unit.is_empty() || width == 0 {
        return String::new();
    }
    let uw = display_width(unit).max(1);
    let n = width / uw;
    let mut out = unit.repeat(n);
    let rem = width - n * uw;
    if rem > 0 {
        out.push_str(&" ".repeat(rem));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_and_width() {
        let s = "\u{1b}[1;31mhi\u{1b}[0m";
        assert_eq!(strip_ansi(s), "hi");
        assert_eq!(display_width(s), 2);
    }

    #[test]
    fn pad_alignment() {
        assert_eq!(pad("ab", 5, Align::Left), "ab   ");
        assert_eq!(pad("ab", 5, Align::Right), "   ab");
        assert_eq!(pad("ab", 5, Align::Center), " ab  ");
    }

    #[test]
    fn truncate_cuts() {
        assert_eq!(truncate("hello world", 8, "…"), "hello w…");
        assert_eq!(truncate("hi", 8, "…"), "hi");
    }
}
