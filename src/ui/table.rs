//! Tables: aligned columns with optional headers and box-drawn borders.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent adds box borders,
//! header separators, and zebra striping; signatures must not change.

use crate::ui::symbols::BoxStyle;
use crate::ui::text::{self, Align};

/// A simple table builder.
pub struct Table {
    headers: Vec<String>,
    aligns: Vec<Align>,
    rows: Vec<Vec<String>>,
    box_style: BoxStyle,
}

impl Table {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            aligns: Vec::new(),
            rows: Vec::new(),
            box_style: BoxStyle::Light,
        }
    }
    pub fn headers(mut self, cols: &[&str]) -> Self {
        self.headers = cols.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn align(mut self, aligns: &[Align]) -> Self {
        self.aligns = aligns.to_vec();
        self
    }
    pub fn box_style(mut self, s: BoxStyle) -> Self {
        self.box_style = s;
        self
    }
    pub fn row(mut self, cells: &[&str]) -> Self {
        self.rows.push(cells.iter().map(|s| s.to_string()).collect());
        self
    }
    pub fn push_row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }
    pub fn render(&self) -> String {
        // STUB: whitespace-aligned columns; agent adds real borders.
        let _ = self.box_style;
        let ncol = self
            .headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if ncol == 0 {
            return String::new();
        }
        let mut widths = vec![0usize; ncol];
        let mut all: Vec<Vec<String>> = Vec::new();
        if !self.headers.is_empty() {
            all.push(self.headers.clone());
        }
        all.extend(self.rows.iter().cloned());
        for r in &all {
            for (i, c) in r.iter().enumerate() {
                widths[i] = widths[i].max(text::display_width(c));
            }
        }
        let align_of = |i: usize| self.aligns.get(i).copied().unwrap_or(Align::Left);
        all.iter()
            .map(|r| {
                r.iter()
                    .enumerate()
                    .map(|(i, c)| text::pad(c, widths[i], align_of(i)))
                    .collect::<Vec<_>>()
                    .join("  ")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
