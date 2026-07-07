//! Glyph sets with automatic ASCII fallbacks (zero-dependency).
//!
//! Every glyph accessor returns a Unicode symbol when [`caps::unicode_enabled`]
//! is true, and a plain-ASCII substitute otherwise.

use crate::ui::caps;

#[inline]
fn uni() -> bool {
    caps::unicode_enabled()
}

macro_rules! glyph {
    ($(#[$m:meta])* $name:ident, $u:expr, $a:expr) => {
        $(#[$m])*
        pub fn $name() -> &'static str {
            if uni() { $u } else { $a }
        }
    };
}

glyph!(/// Success check mark.
    success, "\u{2714}", "+");
glyph!(/// Failure cross.
    failure, "\u{2718}", "x");
glyph!(/// Warning triangle.
    warning, "\u{25b2}", "!");
glyph!(/// Information mark.
    info, "\u{2139}", "i");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller needs a bare question mark yet")]
    /// Question mark.
    question, "?", "?");
glyph!(/// Round bullet.
    bullet, "\u{2022}", "*");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller needs a middle dot yet")]
    /// Middle dot.
    dot, "\u{00b7}", ".");
glyph!(/// Right arrow.
    arrow, "\u{2192}", "->");
glyph!(/// Small right chevron.
    arrow_small, "\u{203a}", ">");
glyph!(/// Heavy right pointer.
    chevron, "\u{276f}", ">");
glyph!(/// Horizontal ellipsis.
    ellipsis, "\u{2026}", "...");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller renders a filled radio yet")]
    /// Filled radio.
    radio_on, "\u{25c9}", "(*)");
glyph!(/// Empty radio.
    radio_off, "\u{25ef}", "( )");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller renders a checked box yet")]
    /// Checked box.
    box_checked, "\u{2611}", "[x]");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller renders an unchecked box yet")]
    /// Unchecked box.
    box_unchecked, "\u{2610}", "[ ]");
glyph!(#[allow(dead_code, reason = "complete glyph set; no caller needs a pointer glyph yet")]
    /// Right-pointing triangle.
    pointer, "\u{25b8}", ">");
glyph!(/// Filled star.
    star, "\u{2605}", "*");
glyph!(/// Lightning bolt.
    bolt, "\u{26a1}", "!");
glyph!(#[allow(dead_code, reason = "complete glyph set; superseded by symbols::hblocks() for bars")]
    /// Full gauge cell.
    gauge_full, "\u{2588}", "#");
glyph!(/// Empty gauge cell.
    gauge_empty, "\u{2591}", "-");

/// Spinner animation frames.
///
/// Only consumed by `ui::progress`'s frozen-but-not-yet-wired spinner API.
#[allow(dead_code, reason = "only consumer is ui::progress's frozen-but-not-yet-wired spinner API")]
pub const SPINNER_UNICODE: &[&str] = &[
    "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}",
    "\u{2807}", "\u{280f}",
];
/// ASCII spinner frames.
#[allow(dead_code, reason = "only consumer is ui::progress's frozen-but-not-yet-wired spinner API")]
pub const SPINNER_ASCII: &[&str] = &["|", "/", "-", "\\"];

/// Active spinner frame set for the current terminal.
#[allow(dead_code, reason = "only consumer is ui::progress's frozen-but-not-yet-wired spinner API")]
pub fn spinner_frames() -> &'static [&'static str] {
    if uni() {
        SPINNER_UNICODE
    } else {
        SPINNER_ASCII
    }
}

/// Eight-level vertical ramp (low → high), for sparklines and bars.
pub const SPARK_UNICODE: [&str; 8] = [
    "\u{2581}", "\u{2582}", "\u{2583}", "\u{2584}", "\u{2585}", "\u{2586}", "\u{2587}", "\u{2588}",
];
/// ASCII vertical ramp.
pub const SPARK_ASCII: [&str; 8] = ["_", "_", ".", ".", "-", "-", "=", "#"];

/// Active 8-level ramp for the current terminal.
pub fn spark_ramp() -> [&'static str; 8] {
    if uni() {
        SPARK_UNICODE
    } else {
        SPARK_ASCII
    }
}

/// Nine-level horizontal eighth-blocks (empty → full), for fine-grained bars.
pub const HBLOCK_UNICODE: [&str; 9] = [
    " ", "\u{258f}", "\u{258e}", "\u{258d}", "\u{258c}", "\u{258b}", "\u{258a}", "\u{2589}",
    "\u{2588}",
];

/// Active horizontal eighth-blocks for the current terminal.
pub fn hblocks() -> [&'static str; 9] {
    if uni() {
        HBLOCK_UNICODE
    } else {
        [" ", " ", " ", " ", "=", "=", "=", "#", "#"]
    }
}

/// Selects a family of box-drawing characters.
///
/// Complete set of standard box-drawing families; `Double` isn't chosen by any
/// caller yet but `box_chars` already handles it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxStyle {
    Light,
    Heavy,
    #[allow(dead_code, reason = "complete box-drawing family set; not selected by any caller yet")]
    Double,
    Rounded,
    Ascii,
}

/// A complete set of box-drawing characters for borders and grids.
#[derive(Clone, Copy, Debug)]
pub struct BoxChars {
    pub h: &'static str,
    pub v: &'static str,
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub cross: &'static str,
    pub tee_down: &'static str,
    pub tee_up: &'static str,
    pub tee_left: &'static str,
    pub tee_right: &'static str,
}

const ASCII_BOX: BoxChars = BoxChars {
    h: "-",
    v: "|",
    tl: "+",
    tr: "+",
    bl: "+",
    br: "+",
    cross: "+",
    tee_down: "+",
    tee_up: "+",
    tee_left: "+",
    tee_right: "+",
};

/// Box characters for `style`, downgraded to ASCII when Unicode is disabled or
/// [`BoxStyle::Ascii`] is requested.
pub fn box_chars(style: BoxStyle) -> BoxChars {
    if !uni() || style == BoxStyle::Ascii {
        return ASCII_BOX;
    }
    match style {
        BoxStyle::Light => BoxChars {
            h: "\u{2500}",
            v: "\u{2502}",
            tl: "\u{250c}",
            tr: "\u{2510}",
            bl: "\u{2514}",
            br: "\u{2518}",
            cross: "\u{253c}",
            tee_down: "\u{252c}",
            tee_up: "\u{2534}",
            tee_left: "\u{2524}",
            tee_right: "\u{251c}",
        },
        BoxStyle::Heavy => BoxChars {
            h: "\u{2501}",
            v: "\u{2503}",
            tl: "\u{250f}",
            tr: "\u{2513}",
            bl: "\u{2517}",
            br: "\u{251b}",
            cross: "\u{254b}",
            tee_down: "\u{2533}",
            tee_up: "\u{253b}",
            tee_left: "\u{252b}",
            tee_right: "\u{2523}",
        },
        BoxStyle::Double => BoxChars {
            h: "\u{2550}",
            v: "\u{2551}",
            tl: "\u{2554}",
            tr: "\u{2557}",
            bl: "\u{255a}",
            br: "\u{255d}",
            cross: "\u{256c}",
            tee_down: "\u{2566}",
            tee_up: "\u{2569}",
            tee_left: "\u{2563}",
            tee_right: "\u{2560}",
        },
        BoxStyle::Rounded => BoxChars {
            h: "\u{2500}",
            v: "\u{2502}",
            tl: "\u{256d}",
            tr: "\u{256e}",
            bl: "\u{2570}",
            br: "\u{256f}",
            cross: "\u{253c}",
            tee_down: "\u{252c}",
            tee_up: "\u{2534}",
            tee_left: "\u{2524}",
            tee_right: "\u{251c}",
        },
        BoxStyle::Ascii => ASCII_BOX,
    }
}
