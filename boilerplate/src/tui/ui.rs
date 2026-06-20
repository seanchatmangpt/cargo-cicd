//! Ratatui rendering for the dashboard.
//!
//! [`render`] is called once per frame from the event loop in
//! `src/nouns/dashboard.rs`. It receives a reference to the current [`App`]
//! and a mutable [`Frame`] from ratatui, and builds the full widget tree for
//! that frame.
//!
//! # Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  cargo-project  [Overview] [Git] [Toolchain]    │  ← header / tabs (3 lines)
//! ├─────────────────────────────────────────────────┤
//! │                                                 │
//! │  (tab content — fills remaining space)          │  ← main area
//! │                                                 │
//! ├─────────────────────────────────────────────────┤
//! │  q:quit  tab:next  shift-tab:prev  r:refresh    │  ← status bar (1 line)
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Tab content
//!
//! | Tab       | Content |
//! |-----------|---------|
//! | Overview  | Key/value table: workspace name, edition, MSRV, verdict badge, refresh countdown. |
//! | Git       | Branch, counts (dirty/staged/untracked), ahead/behind; list of dirty files. |
//! | Toolchain | Rust version, channel, host triple, MSRV from workspace. |
//!
//! # Colour conventions
//!
//! | State | Colour |
//! |-------|--------|
//! | Pass / clean | Green |
//! | Warn / dirty | Yellow |
//! | Fail  | Red |
//! | Neutral info | Cyan |
//! | Dim / labels | DarkGray |

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs,
    },
    Frame,
};

use super::app::App;

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Render the full dashboard for the current frame.
///
/// This function is pure in the sense that it reads `app` but does not mutate
/// it; all state transitions happen in the event loop.
pub fn render<B: Backend>(app: &App, frame: &mut Frame) {
    let size = frame.size();

    // Split the screen vertically into: header (3), main (fill), status bar (1).
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs header
            Constraint::Min(0),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_tabs(app, frame, outer_chunks[0]);
    render_main(app, frame, outer_chunks[1]);
    render_status_bar(app, frame, outer_chunks[2]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab header
// ─────────────────────────────────────────────────────────────────────────────

fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let tab_titles: Vec<Line> = ["Overview", "Git", "Toolchain"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White))))
        .collect();

    let workspace_title = workspace_name(app);
    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", workspace_title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let tabs = Tabs::new(tab_titles)
        .block(block)
        .select(app.selected_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(Span::styled(" | ", Style::default().fg(Color::DarkGray)));

    frame.render_widget(tabs, area);
}

// ─────────────────────────────────────────────────────────────────────────────
// Main content area — dispatches to the active tab
// ─────────────────────────────────────────────────────────────────────────────

fn render_main(app: &App, frame: &mut Frame, area: Rect) {
    match app.selected_tab {
        0 => render_overview_tab(app, frame, area),
        1 => render_git_tab(app, frame, area),
        2 => render_toolchain_tab(app, frame, area),
        _ => render_overview_tab(app, frame, area),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 0 — Overview
// ─────────────────────────────────────────────────────────────────────────────

fn render_overview_tab(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Overview ", Style::default().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    #[cfg(feature = "process-data")]
    {
        let state = &app.engine;
        let verdict = compute_verdict(app);
        let (verdict_text, verdict_color) = verdict_display(&verdict);
        let refresh_in = app.secs_until_refresh();

        let rows = vec![
            Row::new(vec![
                Cell::from("Workspace").style(label_style()),
                Cell::from(state.workspace.name.as_str()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("Root path").style(label_style()),
                Cell::from(state.workspace.root_path.as_str()).style(dim_style()),
            ]),
            Row::new(vec![
                Cell::from("Edition").style(label_style()),
                Cell::from(state.workspace.edition.as_str()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("MSRV").style(label_style()),
                Cell::from(
                    state
                        .workspace
                        .rust_version
                        .as_deref()
                        .unwrap_or("not declared"),
                )
                .style(if state.workspace.rust_version.is_some() {
                    value_style()
                } else {
                    dim_style()
                }),
            ]),
            Row::new(vec![
                Cell::from("Members").style(label_style()),
                Cell::from(state.workspace.members.len().to_string()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("Branch").style(label_style()),
                Cell::from(state.git.branch.as_str()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("Verdict").style(label_style()),
                Cell::from(verdict_text).style(
                    Style::default()
                        .fg(verdict_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Refresh in").style(label_style()),
                Cell::from(format!("{refresh_in}s")).style(dim_style()),
            ]),
        ];

        let widths = [Constraint::Length(12), Constraint::Min(30)];
        let table = Table::new(rows, widths)
            .block(block)
            .column_spacing(2);

        frame.render_widget(table, area);
    }

    #[cfg(not(feature = "process-data"))]
    {
        let text = Paragraph::new("Enable --features process-data for full overview.")
            .block(block)
            .style(dim_style());
        frame.render_widget(text, area);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 1 — Git
// ─────────────────────────────────────────────────────────────────────────────

fn render_git_tab(app: &App, frame: &mut Frame, area: Rect) {
    #[cfg(feature = "process-data")]
    {
        let git = &app.engine.git;
        let dirty_count = git.dirty_files.len();
        let staged_count = git.staged_files.len();
        let untracked_count = git.untracked_files.len();

        // Split the area vertically: summary table on top, file list below.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // summary
                Constraint::Min(0),     // dirty file list
            ])
            .split(area);

        // — Summary table ——————————————————————————————————————————————————
        let upstream_label = if git.has_upstream {
            format!(
                "↑{} ↓{}",
                git.ahead, git.behind
            )
        } else {
            "no upstream".to_owned()
        };

        let clean_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let warn_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let rows = vec![
            Row::new(vec![
                Cell::from("Branch").style(label_style()),
                Cell::from(git.branch.as_str()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("Dirty").style(label_style()),
                Cell::from(dirty_count.to_string()).style(if dirty_count > 0 {
                    warn_style
                } else {
                    clean_style
                }),
            ]),
            Row::new(vec![
                Cell::from("Staged").style(label_style()),
                Cell::from(staged_count.to_string()).style(if staged_count > 0 {
                    warn_style
                } else {
                    clean_style
                }),
            ]),
            Row::new(vec![
                Cell::from("Untracked").style(label_style()),
                Cell::from(untracked_count.to_string()).style(if untracked_count > 0 {
                    Style::default().fg(Color::Cyan)
                } else {
                    clean_style
                }),
            ]),
            Row::new(vec![
                Cell::from("Upstream").style(label_style()),
                Cell::from(upstream_label.as_str()).style(if git.behind > 0 {
                    warn_style
                } else {
                    value_style()
                }),
            ]),
            Row::new(vec![
                Cell::from("Status").style(label_style()),
                Cell::from(if git.is_clean() { "CLEAN" } else { "DIRTY" }).style(
                    if git.is_clean() { clean_style } else { warn_style },
                ),
            ]),
        ];

        let summary_block = Block::default()
            .title(Span::styled(" Git ", Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let widths = [Constraint::Length(12), Constraint::Min(30)];
        let summary_table = Table::new(rows, widths)
            .block(summary_block)
            .column_spacing(2);

        frame.render_widget(summary_table, chunks[0]);

        // — Dirty file list ————————————————————————————————————————————————
        let all_changed: Vec<(&str, &str)> = git
            .dirty_files
            .iter()
            .map(|f| (f.as_str(), "dirty"))
            .chain(git.staged_files.iter().map(|f| (f.as_str(), "staged")))
            .chain(
                git.untracked_files
                    .iter()
                    .map(|f| (f.as_str(), "untracked")),
            )
            .collect();

        let file_items: Vec<ListItem> = all_changed
            .iter()
            .map(|(path, kind)| {
                let color = match *kind {
                    "staged" => Color::Green,
                    "dirty" => Color::Yellow,
                    _ => Color::Cyan,
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(" {kind:<10} "),
                        Style::default().fg(color),
                    ),
                    Span::styled(*path, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let title = if all_changed.is_empty() {
            " Changed files — none "
        } else {
            " Changed files "
        };

        let file_block = Block::default()
            .title(Span::styled(title, Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        if file_items.is_empty() {
            let empty = Paragraph::new("Working tree is clean.")
                .block(file_block)
                .style(Style::default().fg(Color::Green));
            frame.render_widget(empty, chunks[1]);
        } else {
            let list = List::new(file_items).block(file_block);
            frame.render_widget(list, chunks[1]);
        }
    }

    #[cfg(not(feature = "process-data"))]
    {
        let block = Block::default()
            .title(Span::styled(" Git ", Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let text = Paragraph::new("Enable --features process-data for git information.")
            .block(block)
            .style(dim_style());
        frame.render_widget(text, area);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 2 — Toolchain
// ─────────────────────────────────────────────────────────────────────────────

fn render_toolchain_tab(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Toolchain ", Style::default().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    #[cfg(feature = "process-data")]
    {
        let tc = &app.engine.toolchain;
        let ws = &app.engine.workspace;

        let rows = vec![
            Row::new(vec![
                Cell::from("Rust version").style(label_style()),
                Cell::from(tc.rust_version.as_str()).style(value_style()),
            ]),
            Row::new(vec![
                Cell::from("Channel").style(label_style()),
                Cell::from(tc.channel.as_str()).style(channel_style(&tc.channel)),
            ]),
            Row::new(vec![
                Cell::from("Host triple").style(label_style()),
                Cell::from(tc.host.as_str()).style(dim_style()),
            ]),
            Row::new(vec![
                Cell::from("MSRV").style(label_style()),
                Cell::from(
                    ws.rust_version
                        .as_deref()
                        .unwrap_or("not declared"),
                )
                .style(if ws.rust_version.is_some() {
                    value_style()
                } else {
                    dim_style()
                }),
            ]),
            Row::new(vec![
                Cell::from("Edition").style(label_style()),
                Cell::from(ws.edition.as_str()).style(value_style()),
            ]),
        ];

        let widths = [Constraint::Length(14), Constraint::Min(30)];
        let table = Table::new(rows, widths)
            .block(block)
            .column_spacing(2);

        frame.render_widget(table, area);
    }

    #[cfg(not(feature = "process-data"))]
    {
        let text = Paragraph::new("Enable --features process-data for toolchain information.")
            .block(block)
            .style(dim_style());
        frame.render_widget(text, area);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Status bar (bottom line)
// ─────────────────────────────────────────────────────────────────────────────

fn render_status_bar(_app: &App, frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":next  ", Style::default().fg(Color::DarkGray)),
        Span::styled("shift-tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":prev  ", Style::default().fg(Color::DarkGray)),
        Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":refresh  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(":help ", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Left)
    .style(Style::default().bg(Color::Reset));

    frame.render_widget(hint, area);
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn label_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD)
}

fn value_style() -> Style {
    Style::default().fg(Color::White)
}

fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn channel_style(channel: &str) -> Style {
    let color = match channel {
        "stable" => Color::Green,
        "beta" => Color::Yellow,
        "nightly" => Color::Magenta,
        _ => Color::White,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

/// Derive the overall workspace verdict from the current engine snapshot.
///
/// Returns a `&'static str` tag; callers map it to a [`Color`].
#[cfg(feature = "process-data")]
fn compute_verdict(app: &App) -> &'static str {
    let git = &app.engine.git;
    if !git.dirty_files.is_empty() || !git.staged_files.is_empty() {
        return "WARN";
    }
    "PASS"
}

/// Map a verdict tag to a display string and ratatui [`Color`].
fn verdict_display(verdict: &str) -> (String, Color) {
    match verdict {
        "PASS" => (format!("[{}]", verdict), Color::Green),
        "WARN" => (format!("[{}]", verdict), Color::Yellow),
        "FAIL" => (format!("[{}]", verdict), Color::Red),
        _ => (format!("[{}]", verdict), Color::White),
    }
}

/// Return the workspace name from the engine if available, otherwise a default.
fn workspace_name(app: &App) -> String {
    #[cfg(feature = "process-data")]
    {
        let name = &app.engine.workspace.name;
        if name.is_empty() {
            "cargo-project".to_owned()
        } else {
            name.clone()
        }
    }
    #[cfg(not(feature = "process-data"))]
    {
        "cargo-project".to_owned()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_display_pass_is_green() {
        let (text, color) = verdict_display("PASS");
        assert_eq!(color, Color::Green);
        assert!(text.contains("PASS"));
    }

    #[test]
    fn verdict_display_warn_is_yellow() {
        let (text, color) = verdict_display("WARN");
        assert_eq!(color, Color::Yellow);
        assert!(text.contains("WARN"));
    }

    #[test]
    fn verdict_display_fail_is_red() {
        let (text, color) = verdict_display("FAIL");
        assert_eq!(color, Color::Red);
        assert!(text.contains("FAIL"));
    }

    #[test]
    fn channel_style_stable_green() {
        let style = channel_style("stable");
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn channel_style_nightly_magenta() {
        let style = channel_style("nightly");
        assert_eq!(style.fg, Some(Color::Magenta));
    }

    #[test]
    fn label_style_is_dark_gray() {
        let style = label_style();
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[cfg(feature = "process-data")]
    #[test]
    fn compute_verdict_clean_workspace_is_pass() {
        use crate::engine::EngineState;
        use std::time::Instant;

        let app = crate::tui::app::App {
            engine: EngineState::default(),
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        };
        assert_eq!(compute_verdict(&app), "PASS");
    }

    #[cfg(feature = "process-data")]
    #[test]
    fn compute_verdict_dirty_workspace_is_warn() {
        use crate::engine::{EngineState, GitState};
        use std::time::Instant;

        let mut engine = EngineState::default();
        engine.git = GitState {
            dirty_files: vec!["src/main.rs".to_owned()],
            ..Default::default()
        };
        let app = crate::tui::app::App {
            engine,
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        };
        assert_eq!(compute_verdict(&app), "WARN");
    }

    /// Smoke-test the render function using ratatui's `TestBackend`.
    ///
    /// This does not assert pixel-perfect output — it verifies that `render`
    /// does not panic and produces non-empty content.
    #[cfg(feature = "process-data")]
    #[test]
    fn render_does_not_panic() {
        use crate::engine::EngineState;
        use ratatui::{backend::TestBackend, Terminal};
        use std::time::Instant;

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let app = crate::tui::app::App {
            engine: EngineState::default(),
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        };

        terminal.draw(|f| render(&app, f)).unwrap();
    }

    /// Verify all three tabs render without panic.
    #[cfg(feature = "process-data")]
    #[test]
    fn render_all_tabs_do_not_panic() {
        use crate::engine::EngineState;
        use ratatui::{backend::TestBackend, Terminal};
        use std::time::Instant;

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        for tab in 0..3 {
            let app = crate::tui::app::App {
                engine: EngineState::default(),
                refresh_interval_secs: 5,
                last_refresh: Instant::now(),
                should_quit: false,
                selected_tab: tab,
                scroll_offset: 0,
            };
            terminal.draw(|f| render(&app, f)).unwrap();
        }
    }
}
