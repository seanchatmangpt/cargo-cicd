//! SHELL_OUT adapter for the `wpm` CLI binary.
//!
//! ## Scan finding (2026-06-02, wasm4pm commit 65169e62)
//!
//! Selected path: SHELL_OUT — 7 confirmed working commands via the wpm binary.
//! Library coupling deferred (see wasm4pm_current.rs).
//!
//! ## Confirmed working commands
//!
//! | Command                        | Purpose                              |
//! |--------------------------------|--------------------------------------|
//! | wpm audit <input.xes>          | Vision 2030 token-replay conformance |
//! | wpm receipt doctor <file>      | Receipt forensic audit               |
//! | wpm lean                       | Lean Six Sigma waste audit           |
//! | wpm spc status                 | Statistical Process Control          |
//! | wpm doctor                     | System health check                  |
//! | wpm telco status               | Telco routing status                 |
//! | wpm autoprocess                | AutoProcess pipeline                 |
//!
//! ## Blockers (do not use these)
//!
//! - wpm oracle check: confirmed stub — AndonPull detection not implemented
//! - wpm mining conformance: stubs model loading to DFG::new() — always meaningless
//! - wpm doctor: reports FAIL for Cargo.toml/src/ when run outside wasm4pm tree (non-blocking)
//!
//! ## Usage
//!
//! ```no_run
//! use cargo_cicd::integrations::Wasm4pmShell;
//!
//! if let Some(wpm) = Wasm4pmShell::detect() {
//!     let result = wpm.audit("target/cargo-cicd/evidence/events.xes")?;
//!     println!("audit: {}", result);
//! } else {
//!     println!("wpm not found — skipping process audit");
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Discover the wpm binary using a priority-ordered candidate list.
///
/// Discovery order:
/// 1. `WPM_BIN` environment variable
/// 2. `WPM_PATH` environment variable
/// 3. Each path in `WPM_SEARCH_PATHS` (colon-separated on Unix)
/// 4. `.bin/wpm` relative to working directory
/// 5. `bin/wpm` relative to working directory
/// 6. `which wpm` PATH lookup
///
/// Returns `None` if no candidate resolves to an existing file.
pub fn discover_wpm_binary() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    for var in &["WPM_BIN", "WPM_PATH"] {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                candidates.push(PathBuf::from(val));
            }
        }
    }

    if let Ok(search_paths) = std::env::var("WPM_SEARCH_PATHS") {
        for path in search_paths.split(':') {
            if !path.is_empty() {
                candidates.push(PathBuf::from(path));
            }
        }
    }

    candidates.push(PathBuf::from(".bin/wpm"));
    candidates.push(PathBuf::from("bin/wpm"));

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Ok(output) = Command::new("which").arg("wpm").output() {
        let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !p.is_empty() {
            let pb = PathBuf::from(&p);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }

    None
}

/// Shell-out adapter for the confirmed wpm CLI commands.
/// Detected at runtime — if wpm is absent, all methods return graceful PARTIAL.
pub struct Wasm4pmShell {
    binary: String,
}

/// Result of a wpm shell-out invocation.
#[derive(Debug, Clone)]
pub struct WpmResult {
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub verdict: WpmVerdict,
}

impl std::fmt::Display for WpmResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} — {}",
            self.verdict,
            self.command,
            if self.success { "ok" } else { "fail" }
        )
    }
}

/// Shell-operation verdict from a wpm invocation.
///
/// Represents the outcome of a shell command (pass/warn/fail), distinct from
/// `cargo_cicd_core::wpm::verdict::WpmVerdict` which carries the structured
/// JSON court assessment with fitness scores.
#[derive(Debug, Clone, PartialEq)]
pub enum WpmVerdict {
    Pass,
    Warn,
    Fail,
    /// A fitness-scored partial pass; `infer_audit_verdict` doesn't classify
    /// any wpm output as `Partial` yet, but `WpmEvidenceOracle::audit_xes`
    /// already groups it with `Pass`/`Warn` as `Accept`.
    #[allow(dead_code, reason = "meaningful in evidence.rs's match; no current wpm output maps to it")]
    Partial,
    /// wpm binary absent; exercised by tests/wasm4pm_shell.rs's Display test.
    #[allow(dead_code, reason = "exercised by tests/wasm4pm_shell.rs, not constructed by main()-reachable code")]
    NotAvailable,
}

impl std::fmt::Display for WpmVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "pass"),
            Self::Warn => write!(f, "warn"),
            Self::Fail => write!(f, "fail"),
            Self::Partial => write!(f, "partial"),
            Self::NotAvailable => write!(f, "not_available"),
        }
    }
}

impl Wasm4pmShell {
    /// Detect the wpm binary. Returns None if not found.
    /// Uses `discover_wpm_binary()` for multi-source resolution.
    pub fn detect() -> Option<Self> {
        if let Some(pb) = discover_wpm_binary() {
            return Some(Self {
                binary: pb.to_string_lossy().into_owned(),
            });
        }
        // Legacy PATH lookup fallback (kept for environments that set PATH but not env vars)
        if let Ok(output) = Command::new("which").arg("wpm").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(Self { binary: path });
                }
            }
        }
        None
    }

    /// Returns the binary path.
    pub fn binary_path(&self) -> &str {
        &self.binary
    }

    /// Run `wpm audit <path>` — Vision 2030 token-replay conformance audit.
    ///
    /// Accepts an XES event log (`events.xes`). The current `wpm audit` CLI
    /// only supports XES input — it bails early with an actionable message
    /// for OCEL 2.0 JSON files (see `wasm4pm-cli/src/commands/audit.rs`,
    /// `is_ocel_log` check). Callers with an OCEL log must flatten it first
    /// (`wpm run --algorithm dfg --format json`) or use the TypeScript CLI.
    ///
    /// cargo-cicd must emit the evidence file before calling this.
    ///
    /// Unlike the other shell-outs in this file, the verdict here is parsed
    /// from the command's specific three-tier vocabulary (`TRUTHFUL` /
    /// `VARIANCE` / `DECEPTIVE`) rather than the generic pass/warn/fail
    /// keyword heuristic — `wpm audit` always exits 0 and never prints
    /// "fail"/"error"/"warn" in its report, so the generic [`infer_verdict`]
    /// heuristic previously classified every audit result (including
    /// `DECEPTIVE`) as `Pass`, silently blinding the evidence gate to
    /// non-conforming traces.
    pub fn audit(&self, path: &str) -> Result<WpmResult> {
        if !Path::new(path).exists() {
            bail!("wpm audit: evidence file not found at {}", path);
        }
        let output = Command::new(&self.binary).args(["audit", path]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let verdict = infer_audit_verdict(&stdout, output.status.success());
        Ok(WpmResult {
            command: "wpm audit".to_string(),
            success: output.status.success(),
            stdout,
            stderr,
            verdict,
        })
    }

    /// Run `wpm lean` — Lean Six Sigma process waste and efficiency audit.
    ///
    /// One of the "7 confirmed working commands" cataloged in this module's
    /// header; exercised by tests/wasm4pm_shell.rs, not yet called by a noun.
    #[allow(dead_code, reason = "confirmed-working wpm shell-out, exercised by tests/wasm4pm_shell.rs, not yet called by a noun")]
    pub fn lean(&self) -> Result<WpmResult> {
        self.invoke(&["lean"], "lean")
    }

    /// Run `wpm receipt doctor <receipt_path>` — forensic receipt audit.
    ///
    /// One of the "7 confirmed working commands" cataloged in this module's
    /// header; not yet called by a noun.
    #[allow(dead_code, reason = "confirmed-working wpm shell-out, not yet called by a noun")]
    pub fn receipt_doctor(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(&["receipt", "doctor", receipt_path], "receipt doctor")
    }

    /// Run `wpm spc status` — Statistical Process Control status.
    #[allow(dead_code, reason = "confirmed-working wpm shell-out, not yet called by a noun")]
    pub fn spc_status(&self) -> Result<WpmResult> {
        self.invoke(&["spc", "status"], "spc status")
    }

    /// Run `wpm doctor` — system health check.
    ///
    /// Note: reports FAIL for Cargo.toml/src/ when run outside wasm4pm source tree.
    /// This is expected and non-blocking for cargo-cicd projects.
    #[allow(dead_code, reason = "confirmed-working wpm shell-out, not yet called by a noun")]
    pub fn doctor(&self) -> Result<WpmResult> {
        self.invoke(&["doctor"], "doctor")
    }

    /// Run `wpm receipt verify-ocel2 <receipt_path>` — OCEL 2.0 receipt verification.
    pub fn receipt_verify_ocel2(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(
            &["receipt", "verify-ocel2", receipt_path],
            "receipt verify-ocel2",
        )
    }

    /// Run `wpm receipt canonicalize-ocel2 <receipt_path>` — OCEL 2.0 canonicalization.
    #[allow(dead_code, reason = "wpm receipt shell-out sibling of receipt_verify_ocel2(), which is used; not yet called by a noun")]
    pub fn receipt_canonicalize_ocel2(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(
            &["receipt", "canonicalize-ocel2", receipt_path],
            "receipt canonicalize-ocel2",
        )
    }

    /// Run `wpm receipt detect-fixture-mutation <receipt_path>` — mutation detection.
    #[allow(dead_code, reason = "wpm receipt shell-out sibling of receipt_verify_ocel2(), which is used; not yet called by a noun")]
    pub fn receipt_detect_fixture_mutation(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(
            &["receipt", "detect-fixture-mutation", receipt_path],
            "receipt detect-fixture-mutation",
        )
    }

    /// Run `wpm receipt verify-boundary-evidence <receipt_path>` — boundary evidence check.
    #[allow(dead_code, reason = "wpm receipt shell-out sibling of receipt_verify_ocel2(), which is used; not yet called by a noun")]
    pub fn receipt_verify_boundary_evidence(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(
            &["receipt", "verify-boundary-evidence", receipt_path],
            "receipt verify-boundary-evidence",
        )
    }

    /// Run `wpm receipt verify-proof-class <receipt_path>` — proof-class verification.
    #[allow(dead_code, reason = "wpm receipt shell-out sibling of receipt_verify_ocel2(), which is used; not yet called by a noun")]
    pub fn receipt_verify_proof_class(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(
            &["receipt", "verify-proof-class", receipt_path],
            "receipt verify-proof-class",
        )
    }

    /// Run `wpm autoprocess --format json` — AutoProcess pipeline with JSON output.
    ///
    /// One of the "7 confirmed working commands" cataloged in this module's
    /// header; not yet called by a noun.
    #[allow(dead_code, reason = "confirmed-working wpm shell-out, not yet called by a noun")]
    pub fn autoprocess(&self) -> Result<WpmResult> {
        self.invoke(&["autoprocess", "--format", "json"], "autoprocess")
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn invoke(&self, args: &[&str], label: &str) -> Result<WpmResult> {
        let output = Command::new(&self.binary).args(args).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let verdict = infer_verdict(&stdout, &stderr, output.status.success());
        Ok(WpmResult {
            command: format!("wpm {}", label),
            success: output.status.success(),
            stdout,
            stderr,
            verdict,
        })
    }
}

fn infer_verdict(stdout: &str, _stderr: &str, exit_ok: bool) -> WpmVerdict {
    if !exit_ok {
        return WpmVerdict::Fail;
    }
    let lower = stdout.to_lowercase();
    if lower.contains("fail") || lower.contains("error") || lower.contains("warn") {
        WpmVerdict::Warn
    } else {
        WpmVerdict::Pass // exit 0 with neutral output = pass
    }
}

/// Parse the verdict of a `wpm audit` invocation.
///
/// `wpm audit` reports one of three fitness-derived bands on its own
/// "Audit Verdict:" line — `TRUTHFUL` (fitness >= 0.95), `VARIANCE`
/// (0.70–0.95), or `DECEPTIVE` (< 0.70) — and always exits 0 regardless of
/// which band the trace lands in (see `wasm4pm-cli/src/commands/audit.rs`,
/// `print_audit_report`). None of those words overlap with the generic
/// `fail`/`error`/`warn` substrings [`infer_verdict`] looks for, so that
/// heuristic previously mapped every audit result — including `DECEPTIVE`
/// (non-conforming) traces — to `Pass`.
///
/// Mapping: `TRUTHFUL` → `Pass`, `VARIANCE` → `Warn` (matches cargo-cicd's
/// own documented expectation that ambient `live_workspace` traces show
/// honest variance, not full conformance), `DECEPTIVE` → `Fail`.
fn infer_audit_verdict(stdout: &str, exit_ok: bool) -> WpmVerdict {
    if !exit_ok {
        return WpmVerdict::Fail;
    }
    let lower = stdout.to_lowercase();
    if lower.contains("deceptive") {
        WpmVerdict::Fail
    } else if lower.contains("variance") {
        WpmVerdict::Warn
    } else if lower.contains("truthful") {
        WpmVerdict::Pass
    } else {
        // Unrecognised report shape (wpm version drift) — fall back to the
        // generic keyword heuristic rather than silently defaulting to Pass.
        infer_verdict(stdout, "", exit_ok)
    }
}

/// Returns a capability scan summary for use in cicd.toml events.
#[allow(dead_code, reason = "exercised by tests/wasm4pm_shell.rs, not yet called from a live CLI path")]
pub fn capability_summary() -> &'static str {
    "wasm4pm v26.5.29 — 112 capabilities scanned, SHELL_OUT selected, \
     7 confirmed commands: audit, receipt doctor, lean, spc status, doctor, telco status, autoprocess. \
     Library coupling deferred to v26.6.3 (FILE_EXCHANGE path)."
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: `wpm audit` always exits 0 and reports one of
    // TRUTHFUL/VARIANCE/DECEPTIVE — none of which contain "fail"/"error"/"warn".
    // The generic `infer_verdict` heuristic therefore classified a DECEPTIVE
    // (non-conforming) audit result as `Pass`, silently blinding the evidence
    // gate. `infer_audit_verdict` must read the report's own vocabulary.

    #[test]
    fn deceptive_report_maps_to_fail_not_pass() {
        let stdout = "Audit Verdict:            DECEPTIVE\nFitness Score:             0.6457\n";
        assert_eq!(infer_audit_verdict(stdout, true), WpmVerdict::Fail);
    }

    #[test]
    fn variance_report_maps_to_warn() {
        let stdout = "Audit Verdict:            VARIANCE\nFitness Score:             0.80\n";
        assert_eq!(infer_audit_verdict(stdout, true), WpmVerdict::Warn);
    }

    #[test]
    fn truthful_report_maps_to_pass() {
        let stdout = "Audit Verdict:            TRUTHFUL\nFitness Score:             1.0000\n";
        assert_eq!(infer_audit_verdict(stdout, true), WpmVerdict::Pass);
    }

    #[test]
    fn nonzero_exit_is_always_fail_regardless_of_report_text() {
        let stdout = "Audit Verdict:            TRUTHFUL\n";
        assert_eq!(infer_audit_verdict(stdout, false), WpmVerdict::Fail);
    }

    #[test]
    fn unrecognised_report_shape_falls_back_to_generic_heuristic() {
        let stdout = "some future wpm version's unrecognised report shape\n";
        assert_eq!(infer_audit_verdict(stdout, true), WpmVerdict::Pass);
    }
}
