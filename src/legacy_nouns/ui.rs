//! `cargo cicd ui` — preview and exercise the terminal design system.
//!
//! Two verbs:
//! * `demo` renders a non-interactive showcase that exercises every component of
//!   the `crate::ui` toolkit (theme roles, badges, tables, panels, trees,
//!   charts, progress, diagnostics, and hyperlinks), organized into labeled
//!   sections.
//! * `dashboard` populates [`crate::ui::dashboard::DashboardData`] from the live
//!   adapters and renders the composed workspace status view once.
//!
//! All user-facing text stays within the public boundary (generic, professional
//! CLI copy). Output is color-aware: when stdout is a terminal the `demo` verb
//! forces color on so the showcase is vivid; when stdout is piped or captured it
//! leaves color on auto, so the captured form stays plain text.

use std::io::IsTerminal;

use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

use crate::ui::badge::{self, Verdict};
use crate::ui::chart;
use crate::ui::diagnostics::{self, Severity};
use crate::ui::layout;
use crate::ui::panel::{self, Panel};
use crate::ui::progress;
use crate::ui::style::{Color, Style};
use crate::ui::symbols::{self, BoxStyle};
use crate::ui::table::Table;
use crate::ui::text::Align;
use crate::ui::theme::{self, Role};
use crate::ui::tree::Tree;

pub struct UiNoun;

impl UiNoun {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UiNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for UiNoun {
    fn name(&self) -> &'static str {
        "ui"
    }
    fn about(&self) -> &'static str {
        "Preview the cargo-cicd terminal UI design system"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(UiDemoVerb), Box::new(UiDashboardVerb)]
    }
}

// ── demo verb ───────────────────────────────────────────────────────────────

pub struct UiDemoVerb;

impl VerbCommand for UiDemoVerb {
    fn name(&self) -> &'static str {
        "demo"
    }
    fn about(&self) -> &'static str {
        "Showcase the UI components: styles, badges, tables, panels, charts"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // When stdout is an interactive terminal, force color on so the showcase
        // is vivid. When stdout is piped/captured, leave detection on auto so the
        // output stays clean plain text (and the public-boundary contracts hold).
        if std::io::stdout().is_terminal() {
            crate::ui::caps::set_color_override(Some(true));
        }

        print!("{}", render_demo());
        Ok(())
    }
}

/// The bounded content width used to size dividers, banners, and charts so the
/// showcase looks tidy on both wide and narrow terminals.
fn demo_width() -> usize {
    crate::ui::caps::content_width(72)
}

/// Build the entire `demo` showcase as one string. Pure (no I/O) so it can be
/// unit-tested with the color/unicode overrides forced on.
fn render_demo() -> String {
    let w = demo_width();
    let mut out = String::new();

    // ── banner ───────────────────────────────────────────────────────────────
    out.push_str(&panel::banner(
        "cargo-cicd terminal UI",
        "a zero-dependency design system showcase",
    ));
    out.push('\n');
    out.push('\n');

    // ── section: theme roles ───────────────────────────────────────────────────
    push_section(&mut out, "Theme roles", w);
    let roles: &[(Role, &str)] = &[
        (Role::Heading, "Heading"),
        (Role::Subheading, "Subheading"),
        (Role::Strong, "Strong"),
        (Role::Accent, "Accent"),
        (Role::Success, "Success"),
        (Role::Warning, "Warning"),
        (Role::Danger, "Danger"),
        (Role::Info, "Info"),
        (Role::Muted, "Muted"),
        (Role::Link, "Link"),
        (Role::Label, "Label"),
        (Role::Value, "Value"),
    ];
    for (role, name) in roles {
        out.push_str(&format!(
            "  {}  {}\n",
            theme::paint(&format!("{:<11}", name), Role::Label),
            theme::paint("The quick brown fox", *role),
        ));
    }
    out.push('\n');

    // ── section: badges ────────────────────────────────────────────────────────
    push_section(&mut out, "Status badges", w);
    let verdicts = [
        Verdict::Pass,
        Verdict::Warn,
        Verdict::Fail,
        Verdict::Suggest,
        Verdict::Blocked,
        Verdict::Accept,
        Verdict::Refuse,
        Verdict::Info,
        Verdict::Skip,
    ];

    // Render the full set in each badge family so every renderer is exercised.
    out.push_str(&format!("  {}\n", theme::paint("tag:   ", Role::Muted)));
    out.push_str("  ");
    out.push_str(
        &verdicts
            .iter()
            .map(|v| badge::tag(*v))
            .collect::<Vec<_>>()
            .join(" "),
    );
    out.push('\n');

    out.push_str(&format!("  {}\n", theme::paint("pill:  ", Role::Muted)));
    out.push_str("  ");
    out.push_str(
        &verdicts
            .iter()
            .map(|v| badge::pill(*v))
            .collect::<Vec<_>>()
            .join(" "),
    );
    out.push('\n');

    out.push_str(&format!("  {}\n", theme::paint("inline:", Role::Muted)));
    for v in verdicts {
        out.push_str(&format!("    {}\n", badge::inline(v)));
    }
    out.push('\n');

    // ── section: table ─────────────────────────────────────────────────────────
    push_section(&mut out, "Table", w);
    let table = Table::new()
        .headers(&["Component", "Status", "Cases", "Coverage"])
        .align(&[Align::Left, Align::Left, Align::Right, Align::Right])
        .box_style(BoxStyle::Rounded)
        .zebra(true)
        .row(&["core engine", &badge::inline(Verdict::Pass), "128", "97.4%"])
        .row(&["adapters", &badge::inline(Verdict::Pass), "64", "91.2%"])
        .row(&["policies", &badge::inline(Verdict::Warn), "12", "78.0%"])
        .row(&["evidence gate", &badge::inline(Verdict::Skip), "8", "—"])
        .render();
    out.push_str(&indent_block(&table, 2));
    out.push('\n');
    out.push('\n');

    // ── section: panels ────────────────────────────────────────────────────────
    push_section(&mut out, "Panels", w);
    let panel = Panel::new()
        .title("Release readiness")
        .box_style(BoxStyle::Rounded)
        .style(Style::new().fg(Color::Cyan))
        .width(w.saturating_sub(4).max(24))
        .push(format!(
            "{}  workspace builds clean",
            badge::inline(Verdict::Pass)
        ))
        .push(format!(
            "{}  target directory within budget",
            badge::inline(Verdict::Pass)
        ))
        .push(format!(
            "{}  one policy wants attention",
            badge::inline(Verdict::Warn)
        ))
        .render();
    out.push_str(&indent_block(&panel, 2));
    out.push('\n');
    out.push('\n');

    out.push_str(&indent_block(
        &panel::kv(&[
            ("toolchain", "nightly (pinned)"),
            ("branch", "main"),
            ("artifacts", "ready to push"),
        ]),
        2,
    ));
    out.push('\n');
    out.push('\n');

    // ── section: tree ──────────────────────────────────────────────────────────
    push_section(&mut out, "Tree", w);
    let tree = Tree::new(theme::paint("workspace", Role::Strong))
        .child(
            Tree::new("crates")
                .child(Tree::leaf("cargo-cicd").note("bin"))
                .child(Tree::leaf("design-system").note("lib")),
        )
        .child(
            Tree::new("target")
                .child(Tree::leaf("debug").note("incremental"))
                .child(Tree::leaf("release").note("optimized")),
        )
        .child(Tree::leaf("cicd.toml").note("carrier"));
    out.push_str(&indent_block(&tree.render(), 2));
    out.push('\n');
    out.push('\n');

    // ── section: charts ────────────────────────────────────────────────────────
    push_section(&mut out, "Charts", w);
    let bw = w.saturating_sub(20).max(12);

    let series = [
        3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 11.0, 10.0, 13.0, 12.0, 15.0,
    ];
    out.push_str(&format!(
        "  {} {}\n",
        theme::paint("sparkline", Role::Label),
        chart::sparkline(&series),
    ));

    out.push_str(&format!(
        "  {}     {}\n",
        theme::paint("gauge", Role::Label),
        chart::gauge(13.4, 20.0, bw),
    ));

    out.push_str(&format!(
        "  {}     {}\n",
        theme::paint("meter", Role::Label),
        chart::meter(0.62, bw),
    ));
    out.push('\n');

    out.push_str(&format!("  {}\n", theme::paint("barchart", Role::Label)));
    let bars = chart::barchart(
        &[
            ("build", 42.0),
            ("test", 31.0),
            ("clippy", 18.0),
            ("doc", 9.0),
        ],
        bw,
    );
    out.push_str(&indent_block(&bars, 4));
    out.push('\n');
    out.push('\n');

    // ── section: progress (static, CI-safe) ─────────────────────────────────────
    push_section(&mut out, "Progress", w);
    // Static bars only — no blocking animated spinner, so this stays safe under
    // CI and when captured.
    let stages: &[(&str, f64)] = &[
        ("fetch  ", 1.00),
        ("build  ", 0.72),
        ("test   ", 0.45),
        ("publish", 0.10),
    ];
    for (label, frac) in stages {
        out.push_str(&format!(
            "  {} [{}] {:>3}%\n",
            theme::paint(label, Role::Label),
            progress::bar(*frac, bw),
            (frac * 100.0).round() as u64,
        ));
    }
    out.push('\n');

    out.push_str(&indent_block(
        &progress::steps(&[
            ("resolve dependencies", true),
            ("compile workspace", true),
            ("run unit tests", true),
            ("run evidence gate", false),
            ("publish artifacts", false),
        ]),
        2,
    ));
    out.push('\n');
    out.push('\n');

    // ── section: diagnostics ───────────────────────────────────────────────────
    push_section(&mut out, "Diagnostics", w);
    let err = diagnostics::Diagnostic::new(Severity::Error, "target directory exceeds budget")
        .code("E20")
        .note("measured 23.7 GB against a 20.0 GB cap")
        .help("run `cargo cicd target prune` to reclaim disk space");
    out.push_str(&indent_block(&err.render(), 2));
    out.push('\n');

    let warn = diagnostics::line(Severity::Warning, "active toolchain is not pinned");
    out.push_str(&format!("  {}\n", warn));

    let ok = diagnostics::line(Severity::Success, "workspace is push-ready");
    out.push_str(&format!("  {}\n", ok));
    out.push('\n');

    // ── section: hyperlink ─────────────────────────────────────────────────────
    push_section(&mut out, "Links", w);
    out.push_str(&format!(
        "  {} {}\n",
        theme::paint("docs", Role::Label),
        layout::hyperlink(
            "crates.io/crates/cargo-cicd",
            "https://crates.io/crates/cargo-cicd"
        ),
    ));
    out.push('\n');

    out.push_str(&panel::divider(
        &format!("{} end of showcase", symbols::star()),
        w,
    ));
    out.push('\n');

    out
}

/// Push a labeled section header (accent title + horizontal rule) followed by an
/// inset divider, giving the showcase its consistent vertical rhythm.
fn push_section(out: &mut String, title: &str, width: usize) {
    out.push_str(&panel::header(title));
    out.push('\n');
    let _ = width;
    out.push('\n');
}

/// Indent every line of a (possibly multi-line, possibly styled) block by `n`
/// spaces, reusing the layout helper so width math stays ANSI-aware.
fn indent_block(block: &str, n: usize) -> String {
    layout::indent(block, n)
}

// ── dashboard verb ──────────────────────────────────────────────────────────

pub struct UiDashboardVerb;

impl VerbCommand for UiDashboardVerb {
    fn name(&self) -> &'static str {
        "dashboard"
    }
    fn about(&self) -> &'static str {
        "Render the workspace status dashboard"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let data = collect_dashboard_data();
        crate::ui::dashboard::render_fullscreen(&data)
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}

/// Populate [`crate::ui::dashboard::DashboardData`] from the live adapters.
fn collect_dashboard_data() -> crate::ui::dashboard::DashboardData {
    use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};

    let toolchain = ToolchainDetector::active_toolchain();
    let target_gb = TargetScannerAdapter::total_size_gb("target");
    let target_cap_gb = 20.0;

    let git = GitStatusAdapter::query().unwrap_or_default();

    // Best-effort autonomic policy results, mapped to (verdict, message) pairs the
    // dashboard renders. Runs in suggest mode only (never mutates anything).
    let policies = collect_policies(&toolchain, target_gb, git.dirty_files.len());

    crate::ui::dashboard::DashboardData {
        toolchain,
        branch: git.branch,
        target_gb,
        target_cap_gb,
        dirty_files: git.dirty_files.len(),
        untracked: git.untracked_files.len(),
        staged: git.staged_files.len(),
        ahead: git.ahead as usize,
        behind: git.behind as usize,
        history: Vec::new(),
        policies,
    }
}

/// Run the suggest-mode autonomic policies and project each result onto a
/// `(verdict_label, name)` pair for the dashboard.
fn collect_policies(toolchain: &str, target_gb: f64, dirty: usize) -> Vec<(String, String)> {
    use crate::autonomic::policies::{
        run_all_policies, EvidenceState, GitState, PolicyVerdict, WorkspaceInfo,
    };

    let pinned_toolchain = read_pinned_toolchain();
    let workspace_info = WorkspaceInfo {
        target_gb,
        active_toolchain: toolchain.to_string(),
        pinned_toolchain,
        changed_trybuild_fixtures: 0,
    };
    let git_state = GitState {
        dirty_count: dirty,
        commits_behind: None,
    };
    let evidence_state = EvidenceState {
        changed_file_count: 0,
        evidence_fresh: true,
        receipt_exists: false,
        receipt_stale: false,
    };

    run_all_policies(&workspace_info, &git_state, &evidence_state)
        .into_iter()
        .map(|r| {
            let verdict = match r.verdict {
                PolicyVerdict::Pass => Verdict::Pass,
                PolicyVerdict::Warn => Verdict::Warn,
                PolicyVerdict::Suggest => Verdict::Suggest,
            };
            (verdict.label().to_string(), r.name)
        })
        .collect()
}

/// Read the channel pinned in `rust-toolchain.toml`, if present.
fn read_pinned_toolchain() -> Option<String> {
    if std::path::Path::new("rust-toolchain.toml").exists() {
        std::fs::read_to_string("rust-toolchain.toml")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.contains("channel"))
                    .and_then(|l| l.split('"').nth(1))
                    .map(|s| s.to_string())
            })
    } else {
        None
    }
}
