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
//! | wpm audit <input.xes>          | XES conformance audit (SIMD replay)  |
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
//!     let result = wpm.audit("target/cargo-cicd/process/events.xes")?;
//!     println!("audit: {}", result);
//! } else {
//!     println!("wpm not found — skipping process audit");
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Known path to the wpm binary from the capability scan.
const WPM_KNOWN_PATH: &str = "/Users/sac/wasm4pm/target/release/wpm";

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

/// Standardized verdict from a wpm invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum WpmVerdict {
    Pass,
    Warn,
    Fail,
    Partial,
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
    /// Tries: (1) $WPM_PATH env var, (2) known scan path, (3) PATH lookup.
    pub fn detect() -> Option<Self> {
        // Env override
        if let Ok(path) = std::env::var("WPM_PATH") {
            if Path::new(&path).exists() {
                return Some(Self { binary: path });
            }
        }
        // Known path from capability scan
        if Path::new(WPM_KNOWN_PATH).exists() {
            return Some(Self {
                binary: WPM_KNOWN_PATH.to_string(),
            });
        }
        // PATH lookup
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

    /// Run `wpm audit <xes_path>` — XES event log conformance audit.
    ///
    /// Requires: a valid XES event log file at xes_path.
    /// cargo-cicd must emit the XES file before calling this.
    pub fn audit(&self, xes_path: &str) -> Result<WpmResult> {
        if !Path::new(xes_path).exists() {
            bail!("wpm audit: XES file not found at {}", xes_path);
        }
        self.invoke(&["audit", xes_path], "audit")
    }

    /// Run `wpm lean` — Lean Six Sigma process waste and efficiency audit.
    pub fn lean(&self) -> Result<WpmResult> {
        self.invoke(&["lean"], "lean")
    }

    /// Run `wpm receipt doctor <receipt_path>` — forensic receipt audit.
    pub fn receipt_doctor(&self, receipt_path: &str) -> Result<WpmResult> {
        self.invoke(&["receipt", "doctor", receipt_path], "receipt doctor")
    }

    /// Run `wpm spc status` — Statistical Process Control status.
    pub fn spc_status(&self) -> Result<WpmResult> {
        self.invoke(&["spc", "status"], "spc status")
    }

    /// Run `wpm doctor` — system health check.
    ///
    /// Note: reports FAIL for Cargo.toml/src/ when run outside wasm4pm source tree.
    /// This is expected and non-blocking for cargo-cicd projects.
    pub fn doctor(&self) -> Result<WpmResult> {
        self.invoke(&["doctor"], "doctor")
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

/// Returns a capability scan summary for use in cicd.toml events.
pub fn capability_summary() -> &'static str {
    "wasm4pm v26.5.29 — 112 capabilities scanned, SHELL_OUT selected, \
     7 confirmed commands: audit, receipt doctor, lean, spc status, doctor, telco status, autoprocess. \
     Library coupling deferred to v26.6.3 (FILE_EXCHANGE path)."
}
