//! Lightweight data visualization: sparklines, bar charts, gauges, and meters.
//!
//! A small, zero-dependency charting toolkit. Every primitive degrades
//! gracefully: color is applied only through [`Style::paint`] (so it vanishes
//! off-TTY) and every glyph comes from [`symbols`] (so ASCII terminals get a
//! readable fallback). In plain mode the output is still tidy, aligned text of
//! the same display width.
//!
//! Public surface (frozen): [`sparkline`], [`barchart`], [`gauge`], [`meter`],
//! plus [`histogram`] and [`trend`].

use crate::ui::style::{Color, Style};
use crate::ui::symbols;
use crate::ui::text::{self, Align};

// ── color thresholds ─────────────────────────────────────────────────────────
//
// A single, shared notion of "how healthy is this fraction" so meters, gauges,
// and readouts all agree on the green / amber / red boundaries.

const OK_LIMIT: f64 = 0.6;
const WARN_LIMIT: f64 = 0.85;

/// Foreground color for a fill fraction in `0.0..=1.0`.
///
/// `< 0.6` reads as healthy (green), `< 0.85` as cautionary (amber/yellow),
/// and anything higher as saturated (red).
fn level_color(fraction: f64) -> Color {
    if fraction < OK_LIMIT {
        Color::Green
    } else if fraction < WARN_LIMIT {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// The style used to paint a fill at `fraction`.
fn level_style(fraction: f64) -> Style {
    Style::new().fg(level_color(fraction))
}

/// A six-stop cool-to-hot gradient (green → cyan → blue → yellow → orange →
/// red) indexed by a height bucket in `0..buckets`. Used to tint sparkline
/// cells so peaks stand out from troughs.
fn gradient_style(bucket: usize, buckets: usize) -> Style {
    // Map the bucket onto a perceptual-ish ramp using 256-color palette indices,
    // which render on far more terminals than truecolor while still looking
    // smooth. Low values stay calm/green; high values warm to red.
    const RAMP: [u8; 6] = [
        46,  // bright green
        51,  // cyan
        45,  // sky blue
        226, // yellow
        208, // orange
        196, // red
    ];
    let last = buckets.saturating_sub(1).max(1);
    let idx = (bucket * (RAMP.len() - 1) + last / 2) / last;
    let idx = idx.min(RAMP.len() - 1);
    Style::new().fg(Color::Fixed(RAMP[idx]))
}

/// Normalize a series into `0.0..=1.0` against its own min/max. A flat series
/// maps to all-zeros (a calm baseline rather than a misleading full bar).
fn normalize(data: &[f64]) -> Vec<f64> {
    let (min, max) = data
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });
    let span = max - min;
    data.iter()
        .map(|&v| {
            if !v.is_finite() || span.abs() < f64::EPSILON {
                0.0
            } else {
                ((v - min) / span).clamp(0.0, 1.0)
            }
        })
        .collect()
}

// ── sparklines ───────────────────────────────────────────────────────────────

/// A single-line sparkline from a data series.
///
/// Each value becomes one cell from the 8-level [`symbols::spark_ramp`],
/// normalized against the series' own range. When color is enabled, cells are
/// tinted along a cool→hot gradient by height so peaks pop; in plain mode the
/// ramp characters are emitted uncolored. The result is always exactly
/// `data.len()` display columns wide.
pub fn sparkline(data: &[f64]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let ramp = symbols::spark_ramp();
    let levels = ramp.len();
    let mut out = String::with_capacity(data.len() * 4);
    for t in normalize(data) {
        let idx = ((t * (levels - 1) as f64).round() as usize).min(levels - 1);
        out.push_str(&gradient_style(idx, levels).paint(ramp[idx]));
    }
    out
}

// ── meters ───────────────────────────────────────────────────────────────────

/// A bare meter of `width` columns for `fraction` in `0.0..=1.0`.
///
/// The fill uses sub-cell resolution: full columns are solid blocks and the
/// boundary column is an eighth-block from [`symbols::hblocks`], giving 8×
/// finer granularity than whole cells. The filled portion is colored by level
/// (green `< 0.6`, amber `< 0.85`, red otherwise); the empty track is rendered
/// with [`symbols::gauge_empty`]. Output is always exactly `width` columns.
pub fn meter(fraction: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let f = fraction.clamp(0.0, 1.0);
    let blocks = symbols::hblocks(); // 9 stops: empty .. full
    let steps_per_cell = blocks.len() - 1; // 8 eighths per column

    // Total fill expressed in eighth-of-a-column steps.
    let total_steps = (f * (width * steps_per_cell) as f64).round() as usize;
    let full = total_steps / steps_per_cell;
    let partial = total_steps % steps_per_cell;

    let style = level_style(f);
    let mut bar = String::with_capacity(width * 4);

    // Solid filled columns.
    let full = full.min(width);
    if full > 0 {
        bar.push_str(&style.paint(blocks[steps_per_cell].repeat(full)));
    }

    // One partial boundary column, if there's room and a remainder.
    let mut used = full;
    if used < width && partial > 0 {
        bar.push_str(&style.paint(blocks[partial]));
        used += 1;
    }

    // Empty track for the remainder.
    let empty = width - used;
    if empty > 0 {
        bar.push_str(&symbols::gauge_empty().repeat(empty));
    }
    bar
}

// ── gauges ───────────────────────────────────────────────────────────────────

/// A labeled gauge: a [`meter`] plus a right-aligned `value/max` readout and a
/// percentage.
///
/// The readout and percent are tinted to match the meter's fill level, so the
/// numeric figure and the bar tell the same story at a glance. The `value/max`
/// field is right-aligned within a stable width derived from `max` so stacked
/// gauges line up.
pub fn gauge(value: f64, max: f64, width: usize) -> String {
    let frac = if max <= 0.0 {
        0.0
    } else {
        (value / max).clamp(0.0, 1.0)
    };
    let style = level_style(frac);

    // Right-align the "value/max" field to a width based on the larger
    // magnitude so a column of gauges stays aligned.
    let num_w = format!("{:.1}", value.abs().max(max.abs())).len();
    let readout = format!("{:>w$}/{:.1}", format!("{:.1}", value), max, w = num_w);
    let pct = format!("{:>4}", format!("{:.0}%", frac * 100.0));

    format!(
        "{} {} {}",
        meter(frac, width),
        style.paint(readout),
        style.paint(pct)
    )
}

// ── bar charts ───────────────────────────────────────────────────────────────

/// A horizontal bar chart: `(label, value)` rows scaled to `width` columns.
///
/// Labels are left-padded to a common width (via [`text::pad`]) so the bars
/// start at the same column; each bar is a level-colored [`meter`] scaled to
/// the largest value in the set; and the raw value is printed right-aligned
/// after the bar. Negative and non-finite values are treated as zero-length.
pub fn barchart(rows: &[(&str, f64)], width: usize) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let max = rows
        .iter()
        .map(|(_, v)| if v.is_finite() { *v } else { 0.0 })
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    let labw = rows
        .iter()
        .map(|(l, _)| text::display_width(l))
        .max()
        .unwrap_or(0);
    // Stable, right-aligned value column.
    let valw = rows
        .iter()
        .map(|(_, v)| format!("{:.1}", v).len())
        .max()
        .unwrap_or(0);

    rows.iter()
        .map(|(l, v)| {
            let val = if v.is_finite() { *v } else { 0.0 };
            let frac = (val / max).clamp(0.0, 1.0);
            format!(
                "{}  {}  {}",
                text::pad(l, labw, Align::Left),
                meter(frac, width),
                text::pad(&format!("{:.1}", v), valw, Align::Right),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── histogram ────────────────────────────────────────────────────────────────

/// A frequency histogram: distribute `data` into `buckets` equal-width bins
/// over its `[min, max]` range and render the per-bin counts as a [`barchart`].
///
/// Each row is labeled with its bin's lower bound. Returns an empty string when
/// there is no data or `buckets == 0`. Non-finite samples are ignored.
pub fn histogram(data: &[f64], buckets: usize, width: usize) -> String {
    if buckets == 0 {
        return String::new();
    }
    let finite: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return String::new();
    }
    let (min, max) = finite
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let span = max - min;

    let mut counts = vec![0usize; buckets];
    for &v in &finite {
        let idx = if span.abs() < f64::EPSILON {
            0
        } else {
            (((v - min) / span) * buckets as f64) as usize
        };
        counts[idx.min(buckets - 1)] += 1;
    }

    // Build owned labels (bin lower-bound), then borrow them for the barchart.
    let step = if span.abs() < f64::EPSILON {
        0.0
    } else {
        span / buckets as f64
    };
    let labels: Vec<String> = (0..buckets)
        .map(|i| format!("{:.2}", min + step * i as f64))
        .collect();
    let rows: Vec<(&str, f64)> = labels
        .iter()
        .zip(counts.iter())
        .map(|(l, &c)| (l.as_str(), c as f64))
        .collect();

    barchart(&rows, width)
}

// ── trend ────────────────────────────────────────────────────────────────────

/// A compact trend indicator: a [`sparkline`] of `data` followed by a colored
/// direction arrow and the signed delta between the last and first samples.
///
/// Rising series show an up arrow in green, falling series a down arrow in red,
/// and flat series a neutral dot. The delta is the absolute first→last change.
pub fn trend(data: &[f64]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let spark = sparkline(data);

    let first = data.iter().copied().find(|v| v.is_finite());
    let last = data.iter().rev().copied().find(|v| v.is_finite());
    let (first, last) = match (first, last) {
        (Some(a), Some(b)) => (a, b),
        _ => return spark, // no finite samples to compare
    };
    let delta = last - first;

    // Choose a direction glyph from the existing ramp/arrow vocabulary so the
    // ASCII fallback stays coherent with everything else.
    let (glyph, style) = if delta > f64::EPSILON {
        (symbols::arrow(), Style::new().fg(Color::Green))
    } else if delta < -f64::EPSILON {
        // A down arrow built from the same glyph family; ASCII falls back below.
        (down_arrow(), Style::new().fg(Color::Red))
    } else {
        (symbols::dot(), Style::new().fg(Color::BrightBlack))
    };

    format!(
        "{} {} {}",
        spark,
        style.paint(glyph),
        style.paint(format!("{:+.2}", delta))
    )
}

/// Down-pointing arrow with ASCII fallback, mirroring [`symbols::arrow`].
fn down_arrow() -> &'static str {
    if crate::ui::caps::unicode_enabled() {
        "\u{2193}"
    } else {
        "v"
    }
}

