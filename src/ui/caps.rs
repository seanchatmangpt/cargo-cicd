//! Terminal capability detection (zero-dependency, std-only).
//!
//! Decisions about color and Unicode are made once per call from the
//! environment and an optional override.
//!
//! The override is **thread-local**: a command sets it and renders on the same
//! thread (e.g. `ui demo` forces color on, then prints). Keeping it per-thread
//! means the parallel test runner — which schedules each `#[test]` on its own
//! thread — never sees one test's forced override leak into another's
//! auto-detection, so color/unicode assertions stay deterministic without a
//! cross-module lock.

use std::cell::Cell;
use std::io::IsTerminal;

thread_local! {
    // 0 = auto, 1 = force on, 2 = force off.
    static COLOR_MODE: Cell<u8> = const { Cell::new(0) };
    static UNICODE_MODE: Cell<u8> = const { Cell::new(0) };
}

fn encode(mode: Option<bool>) -> u8 {
    match mode {
        None => 0,
        Some(true) => 1,
        Some(false) => 2,
    }
}

/// Force color on (`Some(true)`), off (`Some(false)`), or restore auto-detection
/// (`None`) for the current thread. Intended to be wired to `--color` /
/// `--no-color` CLI flags.
pub fn set_color_override(mode: Option<bool>) {
    COLOR_MODE.with(|c| c.set(encode(mode)));
}

/// Whether ANSI styling should be emitted to stdout right now.
///
/// Precedence: explicit override → `NO_COLOR` → `CLICOLOR_FORCE` → `CLICOLOR`
/// → stdout-is-a-terminal.
pub fn color_enabled() -> bool {
    match COLOR_MODE.with(|c| c.get()) {
        1 => return true,
        2 => return false,
        _ => {}
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Ok(v) = std::env::var("CLICOLOR_FORCE") {
        if v != "0" && !v.is_empty() {
            return true;
        }
    }
    if let Ok(v) = std::env::var("CLICOLOR") {
        if v == "0" {
            return false;
        }
    }
    std::io::stdout().is_terminal()
}

/// Whether Unicode glyphs should be used instead of ASCII fallbacks.
pub fn unicode_enabled() -> bool {
    match UNICODE_MODE.with(|c| c.get()) {
        1 => return true,
        2 => return false,
        _ => {}
    }
    if std::env::var_os("CICD_ASCII").is_some() {
        return false;
    }
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_CTYPE"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    if locale.is_empty() {
        return true; // assume a modern UTF-8 terminal
    }
    let up = locale.to_uppercase();
    up.contains("UTF-8") || up.contains("UTF8")
}

/// Terminal width in columns. Honors `COLUMNS`, defaulting to 80.
pub fn width() -> usize {
    if let Ok(v) = std::env::var("COLUMNS") {
        if let Ok(w) = v.parse::<usize>() {
            if w > 0 {
                return w;
            }
        }
    }
    80
}

/// Terminal width clamped to `[20, max]` for readable, bounded layouts.
pub fn content_width(max: usize) -> usize {
    width().min(max).max(20)
}
