//! Tree rendering with connector glyphs.
//!
//! Draws proper directory-style trees using box-drawing connectors
//! (`├──`, `└──`, `│`) with automatic ASCII fallback (`+`, `\`, `|`) when the
//! terminal is not Unicode-capable. Labels may contain ANSI styling and pass
//! through unchanged; nodes may carry a dim `note` for status annotations.

use crate::ui::caps;
use crate::ui::style::{Color, Style};
use crate::ui::symbols::{box_chars, BoxStyle};

/// Style applied to a node's optional trailing note (rendered dim/muted).
const NOTE_STYLE: Style = Style::new().dim().fg(Color::BrightBlack);

/// The glyphs used to draw one level of tree connectors, resolved against the
/// current Unicode capability.
struct Connectors {
    /// Branch into a child that has following siblings (e.g. `├── `).
    tee: String,
    /// Branch into the final child (e.g. `└── `).
    corner: String,
    /// Vertical continuation under a node with following siblings (e.g. `│   `).
    vbar: String,
    /// Blank continuation under the final child (`    `).
    space: String,
}

impl Connectors {
    fn resolve() -> Self {
        if caps::unicode_enabled() {
            // Derive from the shared Light box set so glyphs stay consistent
            // with the rest of the design system.
            let bc = box_chars(BoxStyle::Light);
            let h2 = format!("{}{}", bc.h, bc.h); // "──"
            Self {
                tee: format!("{}{} ", bc.tee_right, h2), // "├── "
                corner: format!("{}{} ", bc.bl, h2),     // "└── "
                vbar: format!("{}   ", bc.v),            // "│   "
                space: "    ".to_string(),
            }
        } else {
            // ASCII fallback: `|`, `+`, `\`, `-`.
            Self {
                tee: "+-- ".to_string(),
                corner: "\\-- ".to_string(),
                vbar: "|   ".to_string(),
                space: "    ".to_string(),
            }
        }
    }
}

/// A render-tree node.
pub struct Tree {
    pub label: String,
    pub children: Vec<Tree>,
    /// Optional dim annotation rendered after the label (e.g. a status hint).
    pub note: Option<String>,
}

impl Tree {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            note: None,
        }
    }
    /// Alias for [`Tree::new`] — a node with no children.
    pub fn leaf(label: impl Into<String>) -> Self {
        Self::new(label)
    }
    /// Builder-style child append.
    pub fn child(mut self, c: Tree) -> Self {
        self.children.push(c);
        self
    }
    /// Mutable child append.
    #[allow(dead_code, reason = "builder-completeness sibling of child(), which is used")]
    pub fn push(&mut self, c: Tree) {
        self.children.push(c);
    }
    /// Builder-style note: a dim annotation rendered after the label.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Render this node's label plus its optional note, the note dimmed.
    fn render_label(&self) -> String {
        match &self.note {
            Some(n) if !n.is_empty() => format!("{}   {}", self.label, NOTE_STYLE.paint(n)),
            _ => self.label.clone(),
        }
    }

    /// Render the tree to a multi-line string with connector glyphs. The root
    /// prints its own label first (no connector); descendants are prefixed with
    /// `├── `/`└── ` and continuation lanes `│   `/`    `.
    pub fn render(&self) -> String {
        let g = Connectors::resolve();
        let mut out = self.render_label();
        let n = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            let last = i + 1 == n;
            child.render_into(&mut out, "", last, &g);
        }
        out
    }

    /// Recursively render `self` as a child, given the accumulated `prefix` of
    /// ancestor continuation lanes and whether `self` is its parent's last child.
    fn render_into(&self, out: &mut String, prefix: &str, last: bool, g: &Connectors) {
        out.push('\n');
        out.push_str(prefix);
        out.push_str(if last { &g.corner } else { &g.tee });
        out.push_str(&self.render_label());

        // Continuation lane for this node's own descendants.
        let child_prefix = format!("{}{}", prefix, if last { &g.space } else { &g.vbar });
        let n = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            let child_last = i + 1 == n;
            child.render_into(out, &child_prefix, child_last, g);
        }
    }
}
