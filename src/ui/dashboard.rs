//! The composed workspace status dashboard.
//!
//! [`DashboardData`] is a plain data carrier populated by callers from adapters
//! / engine state. [`render`] composes panels, tables, charts, and badges into
//! a single string; [`render_fullscreen`] paints it to the alternate screen.
//!
//! The layout is assembled entirely from the sibling `crate::ui::*` components
//! so it inherits their graceful degradation: color auto-disables off-TTY and
//! glyphs fall back to ASCII. The [`DashboardData`] fields and the function
//! signatures are frozen — callers depend on them.

use std::io::Write as _;

use crate::ui::badge::{self, Verdict};
use crate::ui::chart;
use crate::ui::layout;
use crate::ui::panel::{self, Panel};
use crate::ui::style::{Color, Style};
use crate::ui::symbols::BoxStyle;
use crate::ui::table::Table;
use crate::ui::text::{self, Align};
use crate::ui::theme::{self, Role};
use crate::ui::{caps, symbols};

/// Everything the dashboard needs to render, decoupled from data sources.
#[derive(Clone, Debug, Default)]
pub struct DashboardData {
    pub toolchain: String,
    pub branch: String,
    pub target_gb: f64,
    pub target_cap_gb: f64,
    pub dirty_files: usize,
    pub untracked: usize,
    pub staged: usize,
    pub ahead: usize,
    pub behind: usize,
    /// Historical target-size samples for a sparkline (oldest → newest).
    pub history: Vec<f64>,
    /// `(verdict, message)` policy results.
    pub policies: Vec<(String, String)>,
}

/// Inner content width of the dashboard, bounded for readable layouts.
fn dash_width() -> usize {
    caps::content_width(100)
}

/// Render the dashboard to a string.
///
/// Composes a banner, a row of side-by-side panels (toolchain / git / target),
/// a policy table, and a dim footer hint. Safe on empty/default data and never
/// panics; in plain mode every section degrades to clean aligned text. The
/// literal `cargo-cicd` is always present as a contiguous substring.
pub fn render(data: &DashboardData) -> String {
    let width = dash_width();
    let mut out = String::new();

    // ── banner ───────────────────────────────────────────────────────────────
    out.push_str(&panel::banner("cargo-cicd", "workspace status"));
    out.push('\n');
    out.push('\n');

    // ── side-by-side status panels ─────────────────────────────────────────────
    // Three roughly-equal columns within the available width, leaving room for
    // two single-space gaps and the panel borders.
    let gap = 1usize;
    let col_w = panel_column_width(width, 3, gap);

    let toolchain_panel = render_toolchain_panel(data, col_w);
    let git_panel = render_git_panel(data, col_w);
    let target_panel = render_target_panel(data, col_w);

    let cols = layout::columns(
        &[
            toolchain_panel.as_str(),
            git_panel.as_str(),
            target_panel.as_str(),
        ],
        gap,
    );
    out.push_str(&cols);
    out.push('\n');

    // ── policies section ────────────────────────────────────────────────────────
    out.push('\n');
    out.push_str(&render_policies(data, width));
    out.push('\n');

    // ── footer hint ─────────────────────────────────────────────────────────────
    out.push('\n');
    out.push_str(&render_footer(width));

    out
}

/// Width budget for one of `n` equal panels laid out with `gap` columns between
/// them. Each panel adds two border columns, so account for those too.
fn panel_column_width(total: usize, n: usize, gap: usize) -> usize {
    let n = n.max(1);
    let gaps = gap.saturating_mul(n.saturating_sub(1));
    let usable = total.saturating_sub(gaps);
    (usable / n).max(16)
}

/// The "Toolchain" panel: the active toolchain string, accented.
fn render_toolchain_panel(data: &DashboardData, width: usize) -> String {
    let toolchain = if data.toolchain.is_empty() {
        theme::paint("unknown", Role::Muted)
    } else {
        Style::new().fg(Color::Cyan).bold().paint(&data.toolchain)
    };

    let label = theme::paint("active", Role::Muted);
    Panel::new()
        .title("Toolchain")
        .box_style(BoxStyle::Rounded)
        .width(width)
        .push(format!("{} {}", symbols::bolt(), toolchain))
        .push(label)
        .render()
}

/// The "Git" panel: the branch plus dirty/untracked/staged/ahead/behind state,
/// summarized with a clean/dirty verdict badge and per-metric counts.
fn render_git_panel(data: &DashboardData, width: usize) -> String {
    let branch = if data.branch.is_empty() {
        theme::paint("(detached)", Role::Muted)
    } else {
        Style::new().fg(Color::Magenta).bold().paint(&data.branch)
    };

    // Overall cleanliness drives the headline badge.
    let clean = data.dirty_files == 0 && data.untracked == 0 && data.staged == 0;
    let headline = if clean {
        badge::tag(Verdict::Pass)
    } else {
        badge::tag(Verdict::Warn)
    };

    let mut p = Panel::new()
        .title("Git")
        .box_style(BoxStyle::Rounded)
        .width(width)
        .push(format!("{} {}", symbols::chevron(), branch))
        .push(headline);

    // Per-metric counts: a colored value only when non-zero, otherwise dim zero.
    for (name, count, warn) in [
        ("dirty", data.dirty_files, true),
        ("untracked", data.untracked, true),
        ("staged", data.staged, false),
    ] {
        p = p.push(count_line(name, count, width, warn));
    }
    if data.ahead > 0 || data.behind > 0 {
        let sync = format!(
            "{}{}  {}{}",
            symbols::arrow_small(),
            data.ahead,
            "v",
            data.behind
        );
        p = p.push(format!(
            "{}  {}",
            text::pad("sync", 9, Align::Left),
            Style::new().fg(Color::Blue).paint(sync)
        ));
    }
    p.render()
}

/// One `label  value` metric line; the count is colored by severity when set.
fn count_line(label: &str, count: usize, _width: usize, warn_when_set: bool) -> String {
    let value = if count == 0 {
        theme::paint("0", Role::Muted)
    } else if warn_when_set {
        Style::new()
            .fg(Color::Yellow)
            .bold()
            .paint(count.to_string())
    } else {
        Style::new()
            .fg(Color::Green)
            .bold()
            .paint(count.to_string())
    };
    format!("{}  {}", text::pad(label, 9, Align::Left), value)
}

/// The "Target" panel: a colored gauge of `target/cap` GB plus a sparkline of
/// recent history when available.
fn render_target_panel(data: &DashboardData, width: usize) -> String {
    // Leave room for the panel borders and the `value/max` readout the gauge
    // appends, so the meter itself doesn't overflow the column.
    let inner = width.saturating_sub(2);
    let meter_w = inner.saturating_sub(14).max(6);

    let frac = if data.target_cap_gb <= 0.0 {
        0.0
    } else {
        (data.target_gb / data.target_cap_gb).clamp(0.0, 1.0)
    };

    let gauge = chart::gauge(data.target_gb, data.target_cap_gb, meter_w);
    let gauge_line = colorize_by_threshold(&gauge, frac);

    let mut p = Panel::new()
        .title("Target")
        .box_style(BoxStyle::Rounded)
        .width(width)
        .push(gauge_line)
        .push(format!(
            "{}  {}",
            theme::paint("usage", Role::Muted),
            pct_badge(frac)
        ));

    if !data.history.is_empty() {
        let spark = chart::sparkline(&data.history);
        let spark = Style::new().fg(Color::Cyan).paint(spark);
        p = p.push(format!("{} {}", theme::paint("trend", Role::Muted), spark));
    }

    p.render()
}

/// Apply a green/yellow/red accent to an already-rendered meter string based on
/// fill fraction. Color is dropped automatically when disabled.
fn colorize_by_threshold(s: &str, frac: f64) -> String {
    let color = threshold_color(frac);
    Style::new().fg(color).paint(s)
}

/// A small `NN%` badge whose verdict tracks the same thresholds as the gauge.
fn pct_badge(frac: f64) -> String {
    let pct = (frac * 100.0).round() as i64;
    let verdict = if frac >= 0.9 {
        Verdict::Fail
    } else if frac >= 0.75 {
        Verdict::Warn
    } else {
        Verdict::Pass
    };
    badge::style_for(verdict).paint(format!("{pct}%"))
}

/// Threshold palette: green under 75%, yellow under 90%, red at/above.
fn threshold_color(frac: f64) -> Color {
    if frac >= 0.9 {
        Color::Red
    } else if frac >= 0.75 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// The "Policies" section: a bordered table of `(badge, verdict, message)` rows,
/// one per policy result. Renders a friendly placeholder when there are none.
fn render_policies(data: &DashboardData, width: usize) -> String {
    let mut out = String::new();
    out.push_str(&panel::divider("Policies", width));
    out.push('\n');

    if data.policies.is_empty() {
        out.push_str(&theme::paint("no policy results", Role::Muted));
        return out;
    }

    let mut table = Table::new()
        .headers(&["", "Verdict", "Message"])
        .align(&[Align::Left, Align::Left, Align::Left])
        .box_style(BoxStyle::Light);

    for (verdict, message) in &data.policies {
        let v = Verdict::from_tag(verdict);
        // Dot carries the verdict color; the label stays readable. Message is
        // truncated to keep the table within the available width.
        let mark = badge::dot(v);
        let label = badge::style_for(v).paint(v.label());
        let msg_budget = width.saturating_sub(28).max(12);
        let msg = text::truncate(message, msg_budget, symbols::ellipsis());
        table.push_row(vec![mark, label, msg]);
    }

    out.push_str(&table.render());
    out
}

/// A dim footer hint pointing at the detailed doctor command.
fn render_footer(width: usize) -> String {
    let hint = format!(
        "{} run `cargo cicd workspace doctor` for details",
        symbols::arrow()
    );
    let line = theme::paint(&hint, Role::Muted);
    // Left-align within the width without padding past it; pad is a no-op when
    // the (visible) hint already meets or exceeds `width`.
    text::pad(&line, width, Align::Left)
}

/// Render the dashboard into the terminal's alternate screen buffer.
///
/// When stdout is an interactive terminal *and* color is enabled, the dashboard
/// is painted on the alternate screen with the cursor hidden, then the terminal
/// state is restored on return (even on a write error). Otherwise the dashboard
/// is printed inline. This never reads input and never blocks waiting for a key.
pub fn render_fullscreen(data: &DashboardData) -> std::io::Result<()> {
    use std::io::{stdout, IsTerminal};

    let body = render(data);
    let mut out = stdout();

    let interactive = out.is_terminal() && caps::color_enabled();
    if !interactive {
        write!(out, "{body}")?;
        out.flush()?;
        return Ok(());
    }

    // Enter the alternate screen, hide the cursor, clear, and home.
    let enter = write!(out, "\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H{body}").and_then(|()| out.flush());

    // Always restore terminal state regardless of how the paint went.
    let _ = write!(out, "\x1b[?25h\x1b[?1049l");
    let _ = out.flush();

    enter
}
