//! Terminal capability detection.
//!
//! Results are cached so the detection logic runs at most once per process.

use std::sync::OnceLock;

/// Detected terminal capabilities for the current process.
#[derive(Debug, Clone, Copy)]
pub struct Caps {
    /// Whether stdout is connected to an interactive terminal.
    pub is_tty: bool,
    /// Whether the terminal is likely to support ANSI colour escape codes.
    pub has_color: bool,
    /// Whether the terminal is likely to support Unicode glyphs.
    pub has_unicode: bool,
}

static CAPS: OnceLock<Caps> = OnceLock::new();

impl Caps {
    /// Detect capabilities for the current process (cached after first call).
    pub fn detect() -> &'static Self {
        CAPS.get_or_init(|| {
            let is_tty = Self::stdout_is_tty();
            let has_color = is_tty && Self::env_allows_color();
            let has_unicode = is_tty && Self::locale_supports_unicode();
            Self { is_tty, has_color, has_unicode }
        })
    }

    /// Override capabilities for testing.  Only effective if called before the
    /// first `detect()` call.
    #[cfg(test)]
    pub fn set_for_test(caps: Caps) {
        let _ = CAPS.set(caps);
    }

    fn stdout_is_tty() -> bool {
        // Use the `TERM` env var and the `NO_COLOR` convention as a heuristic
        // when we cannot call `isatty()` without a libc dependency.
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        // CI environments typically set one of these.
        if std::env::var_os("CI").is_some() {
            return false;
        }
        // If TERM is "dumb" or unset, assume no TTY.
        match std::env::var("TERM").as_deref() {
            Ok("dumb") | Err(_) => false,
            _ => true,
        }
    }

    fn env_allows_color() -> bool {
        std::env::var_os("NO_COLOR").is_none()
            && std::env::var("TERM").as_deref() != Ok("dumb")
    }

    fn locale_supports_unicode() -> bool {
        // Check LANG / LC_ALL for UTF-8 indication.
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                if val.to_ascii_uppercase().contains("UTF") {
                    return true;
                }
            }
        }
        // On macOS and most Linux systems, UTF-8 is the default.
        cfg!(any(target_os = "macos", target_os = "linux"))
    }
}

/// Plain-mode caps: no TTY, no colour, no Unicode.
pub const PLAIN: Caps = Caps { is_tty: false, has_color: false, has_unicode: false };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_caps_all_false() {
        assert!(!PLAIN.is_tty);
        assert!(!PLAIN.has_color);
        assert!(!PLAIN.has_unicode);
    }

    #[test]
    fn detect_returns_consistent_value() {
        let a = Caps::detect();
        let b = Caps::detect();
        // Same pointer — cached.
        assert!(std::ptr::eq(a, b));
    }
}
