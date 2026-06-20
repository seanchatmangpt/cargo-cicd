//! Toolchain state dimension.

/// Active Rust toolchain information.
#[derive(Debug, Clone, Default)]
pub struct ToolchainState {
    /// Full version string returned by `rustc --version` (e.g. `rustc 1.86.0 (05f9846f8 2025-03-31)`).
    pub rust_version: String,
    /// Toolchain channel: `stable`, `beta`, `nightly`, or a date string.
    pub channel: String,
    /// Host triple (e.g. `x86_64-unknown-linux-gnu`).
    pub host: String,
}
