//! Layout helpers: indentation, centering, rules, word-wrapping, multi-column
//! composition, and terminal hyperlinks.
//!
//! All width math goes through [`text::display_width`] / [`text::pad`] so that
//! styled cells (containing ANSI escapes) align by their *visible* width.

use crate::ui::caps;
use crate::ui::text::{self, Align};

/// Indent every line of `text` by `n` spaces.
pub fn indent(text: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    text.lines()
        .map(|l| format!("{pad}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Center a single line within `width` columns.
pub fn center(text: &str, width: usize) -> String {
    text::pad(text, width, Align::Center)
}

/// A horizontal rule of `width` columns built from `ch`.
pub fn rule(width: usize, ch: &str) -> String {
    text::fill(ch, width)
}

/// Greedy word-wrap `text` to at most `width` display columns.
///
/// Existing newlines are treated as hard breaks (each paragraph is wrapped
/// independently). Words longer than `width` are placed on their own line
/// rather than split mid-word. Width is measured with [`text::display_width`],
/// so already-styled input wraps by visible width. Returns the original text
/// unchanged when `width` is `0`.
pub fn wrap(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }
    let mut out: Vec<String> = Vec::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            // Preserve the explicit hard break between source lines.
        }
        let mut cur = String::new();
        let mut cur_w = 0usize;
        let mut wrapped_any = false;
        for word in line.split_whitespace() {
            let ww = text::display_width(word);
            if cur.is_empty() {
                cur.push_str(word);
                cur_w = ww;
            } else if cur_w + 1 + ww <= width {
                cur.push(' ');
                cur.push_str(word);
                cur_w += 1 + ww;
            } else {
                out.push(std::mem::take(&mut cur));
                cur.push_str(word);
                cur_w = ww;
                wrapped_any = true;
            }
        }
        // Always emit the trailing buffer; for an empty source line this yields
        // an empty string, preserving blank lines / paragraph spacing.
        let _ = wrapped_any;
        out.push(cur);
    }
    out.join("\n")
}

/// Join multi-line `blocks` side by side with `gap` spaces between them.
///
/// Each block is laid out as a column: every line is left-padded to that
/// block's widest line (measured by visible width), then columns are zipped
/// row-by-row. Shorter columns are padded with blank (full-width) rows so the
/// grid stays rectangular. The result is the combined multi-line string.
pub fn columns(blocks: &[&str], gap: usize) -> String {
    let blocks: Vec<&&str> = blocks.iter().filter(|b| !b.is_empty()).collect();
    if blocks.is_empty() {
        return String::new();
    }

    // Split each block into lines and record its column width.
    let mut cols: Vec<Vec<&str>> = Vec::with_capacity(blocks.len());
    let mut widths: Vec<usize> = Vec::with_capacity(blocks.len());
    let mut height = 0usize;
    for b in &blocks {
        let lines: Vec<&str> = b.split('\n').collect();
        let w = lines
            .iter()
            .map(|l| text::display_width(l))
            .max()
            .unwrap_or(0);
        height = height.max(lines.len());
        cols.push(lines);
        widths.push(w);
    }

    let sep = " ".repeat(gap);
    let mut rows: Vec<String> = Vec::with_capacity(height);
    for row in 0..height {
        let mut cells: Vec<String> = Vec::with_capacity(cols.len());
        let last = cols.len() - 1;
        for (ci, col) in cols.iter().enumerate() {
            let raw = col.get(row).copied().unwrap_or("");
            // Don't pad the final column's trailing edge — avoids stray
            // trailing whitespace while keeping interior columns aligned.
            if ci == last {
                cells.push(raw.to_string());
            } else {
                cells.push(text::pad(raw, widths[ci], Align::Left));
            }
        }
        rows.push(cells.join(&sep));
    }
    rows.join("\n")
}

/// A terminal hyperlink (OSC-8) with visible `label`, falling back to the label
/// alone where unsupported.
///
/// When [`caps::color_enabled`] is true (a capable, interactive terminal), an
/// OSC-8 sequence is emitted with `ST` (`ESC \`) terminators:
/// `ESC ] 8 ; ; URL ST LABEL ESC ] 8 ; ; ST`. Otherwise the raw `label` is
/// returned so piped/captured output stays plain.
pub fn hyperlink(label: &str, url: &str) -> String {
    if caps::color_enabled() {
        format!("\u{1b}]8;;{url}\u{1b}\\{label}\u{1b}]8;;\u{1b}\\")
    } else {
        label.to_string()
    }
}
