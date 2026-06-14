//! Lightweight data visualization: sparklines, bar charts, gauges, and meters.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent adds color
//! thresholds and finer (sub-cell) resolution; signatures must not change.

use crate::ui::symbols;
use crate::ui::text::{self, Align};

/// A single-line sparkline from a data series.
pub fn sparkline(data: &[f64]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let ramp = symbols::spark_ramp();
    let (min, max) = data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let span = max - min;
    data.iter()
        .map(|&v| {
            let t = if span.abs() < f64::EPSILON {
                0.0
            } else {
                (v - min) / span
            };
            let idx = ((t * (ramp.len() - 1) as f64).round() as usize).min(ramp.len() - 1);
            ramp[idx]
        })
        .collect::<String>()
}

/// A horizontal bar chart: `(label, value)` rows scaled to `width` columns.
pub fn barchart(rows: &[(&str, f64)], width: usize) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let max = rows.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max).max(f64::EPSILON);
    let labw = rows.iter().map(|(l, _)| text::display_width(l)).max().unwrap_or(0);
    rows.iter()
        .map(|(l, v)| {
            let frac = (v / max).clamp(0.0, 1.0);
            format!("{}  {}", text::pad(l, labw, Align::Left), meter(frac, width))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A labeled gauge: a meter plus a `value/max` readout.
pub fn gauge(value: f64, max: f64, width: usize) -> String {
    let frac = if max <= 0.0 {
        0.0
    } else {
        (value / max).clamp(0.0, 1.0)
    };
    format!("{} {:.1}/{:.1}", meter(frac, width), value, max)
}

/// A bare meter of `width` columns for `fraction` in `0.0..=1.0`.
pub fn meter(fraction: f64, width: usize) -> String {
    let f = fraction.clamp(0.0, 1.0);
    let filled = (f * width as f64).round() as usize;
    format!(
        "{}{}",
        symbols::gauge_full().repeat(filled),
        symbols::gauge_empty().repeat(width.saturating_sub(filled))
    )
}
