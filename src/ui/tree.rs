//! Tree rendering with connector glyphs.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent draws real
//! `├──`/`└──`/`│` connectors; signatures must not change.

/// A render-tree node.
pub struct Tree {
    pub label: String,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
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
    pub fn render(&self) -> String {
        // STUB: indented list; agent draws connector glyphs.
        fn walk(node: &Tree, depth: usize, out: &mut String) {
            for c in &node.children {
                out.push('\n');
                out.push_str(&"  ".repeat(depth + 1));
                out.push_str(&c.label);
                walk(c, depth + 1, out);
            }
        }
        let mut out = self.label.clone();
        walk(self, 0, &mut out);
        out
    }
}

/// Render a flat list of slash-separated paths as a directory tree.
pub fn from_paths(paths: &[&str]) -> String {
    // STUB: newline-joined; agent builds a real nested tree.
    paths.join("\n")
}
