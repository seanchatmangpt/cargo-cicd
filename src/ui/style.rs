//! Zero-dependency ANSI styling.
//!
//! [`Style`] is a small, `Copy`, const-constructible builder. [`Style::paint`]
//! wraps text in SGR escape codes only when [`caps::color_enabled`] is true;
//! otherwise it returns the text unchanged, so captured/piped output is plain.

use crate::ui::caps;

/// A terminal color: the 16 ANSI colors, a 256-palette index, or 24-bit RGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// 256-color palette index.
    Fixed(u8),
    /// 24-bit truecolor.
    Rgb(u8, u8, u8),
}

impl Color {
    fn fg_params(self) -> String {
        use Color::*;
        match self {
            Black => "30".into(),
            Red => "31".into(),
            Green => "32".into(),
            Yellow => "33".into(),
            Blue => "34".into(),
            Magenta => "35".into(),
            Cyan => "36".into(),
            White => "37".into(),
            BrightBlack => "90".into(),
            BrightRed => "91".into(),
            BrightGreen => "92".into(),
            BrightYellow => "93".into(),
            BrightBlue => "94".into(),
            BrightMagenta => "95".into(),
            BrightCyan => "96".into(),
            BrightWhite => "97".into(),
            Fixed(n) => format!("38;5;{n}"),
            Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }

    fn bg_params(self) -> String {
        use Color::*;
        match self {
            Black => "40".into(),
            Red => "41".into(),
            Green => "42".into(),
            Yellow => "43".into(),
            Blue => "44".into(),
            Magenta => "45".into(),
            Cyan => "46".into(),
            White => "47".into(),
            BrightBlack => "100".into(),
            BrightRed => "101".into(),
            BrightGreen => "102".into(),
            BrightYellow => "103".into(),
            BrightBlue => "104".into(),
            BrightMagenta => "105".into(),
            BrightCyan => "106".into(),
            BrightWhite => "107".into(),
            Fixed(n) => format!("48;5;{n}"),
            Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
        }
    }
}

/// An immutable, `Copy` style: optional fg/bg colors plus text attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// True when this style would emit no escape codes.
    pub fn is_plain(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
    }

    fn sgr(&self) -> String {
        let mut p: Vec<String> = Vec::new();
        if self.bold {
            p.push("1".into());
        }
        if self.dim {
            p.push("2".into());
        }
        if self.italic {
            p.push("3".into());
        }
        if self.underline {
            p.push("4".into());
        }
        if let Some(fg) = self.fg {
            p.push(fg.fg_params());
        }
        if let Some(bg) = self.bg {
            p.push(bg.bg_params());
        }
        p.join(";")
    }

    /// Wrap `text` in this style's ANSI codes, honoring [`caps::color_enabled`].
    /// Returns `text` unchanged when color is disabled or the style is plain.
    pub fn paint(&self, text: impl AsRef<str>) -> String {
        let text = text.as_ref();
        if self.is_plain() || !caps::color_enabled() {
            return text.to_string();
        }
        format!("\u{1b}[{}m{}\u{1b}[0m", self.sgr(), text)
    }
}

/// Free-function form: `paint("hi", Style::new().bold())`.
pub fn paint(text: impl AsRef<str>, style: Style) -> String {
    style.paint(text)
}
