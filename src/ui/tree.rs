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

/// An intermediate node used while folding a flat path list into a real tree.
struct PathNode {
    name: String,
    children: Vec<PathNode>,
}

impl PathNode {
    fn root() -> Self {
        Self {
            name: String::new(),
            children: Vec::new(),
        }
    }

    /// Find or create a direct child named `name`.
    fn entry(&mut self, name: &str) -> &mut PathNode {
        // Linear search keeps merging order-independent; small sibling sets.
        if let Some(idx) = self.children.iter().position(|c| c.name == name) {
            return &mut self.children[idx];
        }
        self.children.push(PathNode {
            name: name.to_string(),
            children: Vec::new(),
        });
        let last = self.children.len() - 1;
        &mut self.children[last]
    }

    /// Sort this subtree's children lexicographically, recursively.
    fn sort(&mut self) {
        self.children.sort_by(|a, b| a.name.cmp(&b.name));
        for c in &mut self.children {
            c.sort();
        }
    }

    /// Convert into a renderable [`Tree`] with the given label.
    fn into_tree(self, label: String) -> Tree {
        let mut t = Tree::new(label);
        for c in self.children {
            let name = c.name.clone();
            t.children.push(c.into_tree(name));
        }
        t
    }
}

/// Render a flat list of slash-separated paths as a directory tree.
///
/// Paths are split on `/`, common prefixes are merged into shared parent nodes,
/// siblings are sorted, and the result is drawn with connector glyphs. A path
/// that is a prefix of another simply becomes a parent node.
pub fn from_paths(paths: &[&str]) -> String {
    let mut root = PathNode::root();
    for path in paths {
        let mut cursor = &mut root;
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            cursor = cursor.entry(seg);
        }
    }
    root.sort();
    root.into_tree(".".to_string()).render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps::set_unicode_override;
    use crate::ui::text::strip_ansi;

    #[test]
    fn two_level_render_has_connectors_and_labels() {
        set_unicode_override(Some(true));

        let tree = Tree::new("root")
            .child(Tree::leaf("alpha"))
            .child(Tree::leaf("omega"));
        let out = tree.render();

        // Root label first, then both children.
        assert!(out.starts_with("root"));
        assert!(out.contains("alpha"));
        assert!(out.contains("omega"));
        // At least one connector char is present (tee or corner).
        assert!(
            out.contains('\u{251c}') || out.contains('\u{2514}'),
            "expected a connector glyph in:\n{out}"
        );
        // The last child uses the corner, earlier children the tee.
        assert!(out.contains('\u{2514}'), "expected corner for last child");
        assert!(out.contains('\u{251c}'), "expected tee for first child");

        set_unicode_override(None);
    }

    #[test]
    fn from_paths_groups_common_prefix() {
        set_unicode_override(Some(true));

        let out = from_paths(&["a/b", "a/c"]);
        // `a` appears once as a parent of both `b` and `c`.
        assert!(out.contains('a'), "missing parent node in:\n{out}");
        assert!(out.contains('b'));
        assert!(out.contains('c'));
        assert_eq!(out.matches("a").count(), 1, "prefix `a` should be merged:\n{out}");
        // `b` then `c` (sorted): tee before corner.
        let bpos = out.find('b').unwrap();
        let cpos = out.find('c').unwrap();
        assert!(bpos < cpos, "siblings should be sorted in:\n{out}");

        set_unicode_override(None);
    }

    #[test]
    fn note_renders_dim_after_label() {
        crate::ui::caps::set_color_override(Some(true));
        set_unicode_override(Some(true));

        let tree = Tree::new("root").child(Tree::leaf("item").note("ok"));
        let out = tree.render();
        // The note text appears, after the label, and is ANSI-styled (dim).
        assert!(strip_ansi(&out).contains("item   ok"), "note layout wrong:\n{out}");
        assert!(out.contains("\u{1b}["), "note should be styled when color is on");

        set_unicode_override(None);
        crate::ui::caps::set_color_override(None);
    }

    #[test]
    fn ascii_fallback_uses_plain_connectors() {
        set_unicode_override(Some(false));

        let out = Tree::new("root")
            .child(Tree::leaf("one"))
            .child(Tree::leaf("two"))
            .render();
        // ASCII connectors only — no box-drawing glyphs.
        assert!(out.contains("+-- ") || out.contains("\\-- "), "expected ASCII tee/corner:\n{out}");
        assert!(!out.contains('\u{251c}'));
        assert!(!out.contains('\u{2514}'));

        set_unicode_override(None);
    }

    #[test]
    fn nested_continuation_lanes_align() {
        set_unicode_override(Some(true));

        // root
        // ├── a
        // │   └── deep
        // └── b
        let out = Tree::new("root")
            .child(Tree::new("a").child(Tree::leaf("deep")))
            .child(Tree::leaf("b"))
            .render();
        // Under non-last `a`, the deep child line carries a vertical lane.
        assert!(out.contains('\u{2502}'), "expected vertical continuation lane:\n{out}");
        assert!(out.contains("deep"));

        set_unicode_override(None);
    }
}
