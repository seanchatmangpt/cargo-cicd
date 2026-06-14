//! The composed workspace status dashboard.
//!
//! [`DashboardData`] is a plain data carrier populated by callers from adapters
//! / engine state. [`render`] composes panels, tables, charts, and badges into
//! a single string; [`render_fullscreen`] paints it to the alternate screen.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent composes the rich
//! layout using the other `ui` modules; the [`DashboardData`] fields and the
//! function signatures must not change (callers depend on them).

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

/// Render the dashboard to a string.
pub fn render(data: &DashboardData) -> String {
    // STUB: linear key/values; agent composes panels/tables/charts/badges.
    let mut out = String::new();
    out.push_str("cargo-cicd dashboard\n");
    out.push_str(&format!("toolchain: {}\n", data.toolchain));
    out.push_str(&format!("branch:    {}\n", data.branch));
    out.push_str(&format!(
        "target:    {:.2}/{:.2} GB\n",
        data.target_gb, data.target_cap_gb
    ));
    out.push_str(&format!(
        "git:       {} dirty, {} untracked, {} staged\n",
        data.dirty_files, data.untracked, data.staged
    ));
    for (verdict, msg) in &data.policies {
        out.push_str(&format!("[{verdict}] {msg}\n"));
    }
    out
}

/// Render the dashboard into the terminal's alternate screen buffer.
pub fn render_fullscreen(data: &DashboardData) -> std::io::Result<()> {
    // STUB: print inline; agent uses alt-screen + clears on exit.
    print!("{}", render(data));
    Ok(())
}
