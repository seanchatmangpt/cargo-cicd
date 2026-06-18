//! SHELL_OUT adapter for the `affi` CLI — the **affidavit** cryptographic
//! provenance engine (<https://github.com/seanchatmangpt/affidavit>).
//!
//! affidavit "assembles, seals, and certifies provenance receipts — append-only,
//! content-addressed BLAKE3 chains of operation-events." Its doctrine,
//! *"Certify, Don't Decide"*, is the same principle as cargo-cicd's invariant
//! **E1**: the engine never grades itself; only an external witness issues a
//! verdict. affidavit therefore slots in as a **second external oracle**
//! alongside wasm4pm — where wasm4pm scores process *conformance*, affidavit
//! proves evidence *integrity* with a rolling BLAKE3 chain.
//!
//! ## Why shell-out (not a library dependency)
//!
//! The affidavit crate's mandatory `core` feature pulls in `wasm4pm-compat`,
//! which requires unstable rustc features (`generic_const_exprs`,
//! `unsized_const_params`, …). cargo-cicd is a **stable-toolchain** tool
//! (`rust-version = 1.86`), so linking affidavit in-process would force the
//! whole project — and every downstream user — onto nightly. Instead we invoke
//! the installed `affi` binary at runtime, exactly as [`Wasm4pmShell`] invokes
//! `wpm`. When `affi` is absent the integration degrades gracefully
//! ([`AffidavitVerdict::Blocked`]).
//!
//! ## Confirmed CLI contract (affidavit `examples/golden_run.sh`)
//!
//! | Command                                                         | Purpose                       |
//! |-----------------------------------------------------------------|-------------------------------|
//! | `affi receipt emit --type T --object ID:TYPE:QUAL --payload F`  | record one operation-event    |
//! | `affi receipt assemble --out receipt.json`                      | seal accumulated events       |
//! | `affi receipt verify receipt.json`                              | certify (exit 0 = ACCEPT)     |
//! | `affi receipt show receipt.json`                                | display a sealed receipt      |
//!
//! The certify verdict is conveyed by **exit code**: `0` = ACCEPT, non-zero =
//! REJECT (a single bit-flip anywhere in the chain flips it to REJECT).
//!
//! [`Wasm4pmShell`]: super::Wasm4pmShell

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::evidence::ProcessEvent;

/// Outcome verdict from an `affi receipt verify` invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum AffidavitVerdict {
    /// `affi receipt verify` exited 0 — the receipt was certified.
    Accept,
    /// `affi receipt verify` exited non-zero — certification rejected.
    Reject,
    /// The `affi` binary is not installed / could not be run.
    Blocked,
}

impl std::fmt::Display for AffidavitVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "ACCEPT"),
            Self::Reject => write!(f, "REJECT"),
            Self::Blocked => write!(f, "BLOCKED"),
        }
    }
}

/// Result of an `affi` shell-out.
#[derive(Debug, Clone)]
pub struct AffidavitResult {
    /// Human-readable command label, e.g. `"affi receipt verify"`.
    pub command: String,
    /// Whether the process exited with status 0.
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    /// Verdict derived from the exit status.
    pub verdict: AffidavitVerdict,
}

/// Shell-out adapter for the `affi` provenance CLI, detected at runtime.
pub struct AffidavitShell {
    binary: String,
}

impl AffidavitShell {
    /// Detect the `affi` binary. Returns `None` if not found.
    /// Tries: (1) `$AFFI_PATH` env var, (2) `which affi` on `PATH`.
    pub fn detect() -> Option<Self> {
        if let Ok(path) = std::env::var("AFFI_PATH") {
            if Path::new(&path).exists() {
                return Some(Self { binary: path });
            }
        }
        if let Ok(output) = Command::new("which").arg("affi").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(Self { binary: path });
                }
            }
        }
        None
    }

    /// Returns the resolved binary path.
    pub fn binary_path(&self) -> &str {
        &self.binary
    }

    /// `affi --version` — readiness probe. Returns the trimmed version string.
    pub fn version(&self) -> Option<String> {
        let out = Command::new(&self.binary).arg("--version").output().ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// `affi receipt emit --type <event_type> --object <object> --payload <file>`.
    ///
    /// Run inside `work_dir` so affi's accumulating working state stays scoped to
    /// the receipt directory rather than leaking into the current directory.
    pub fn emit(
        &self,
        work_dir: &Path,
        event_type: &str,
        object: &str,
        payload_file: &Path,
    ) -> std::io::Result<AffidavitResult> {
        let payload = payload_file.to_string_lossy();
        self.invoke(
            work_dir,
            &[
                "receipt", "emit", "--type", event_type, "--object", object, "--payload", &payload,
            ],
            "receipt emit",
        )
    }

    /// `affi receipt assemble --out <out_file>` — seal accumulated events.
    pub fn assemble(&self, work_dir: &Path, out_file: &Path) -> std::io::Result<AffidavitResult> {
        let out = out_file.to_string_lossy();
        self.invoke(
            work_dir,
            &["receipt", "assemble", "--out", &out],
            "receipt assemble",
        )
    }

    /// `affi receipt verify <receipt_file>` — certify (exit 0 = ACCEPT).
    pub fn verify(&self, receipt_file: &Path) -> std::io::Result<AffidavitResult> {
        let work_dir = receipt_file.parent().unwrap_or_else(|| Path::new("."));
        let receipt = receipt_file.to_string_lossy();
        self.invoke(work_dir, &["receipt", "verify", &receipt], "receipt verify")
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn invoke(
        &self,
        work_dir: &Path,
        args: &[&str],
        label: &str,
    ) -> std::io::Result<AffidavitResult> {
        let output = Command::new(&self.binary)
            .args(args)
            .current_dir(work_dir)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let verdict = if output.status.success() {
            AffidavitVerdict::Accept
        } else {
            AffidavitVerdict::Reject
        };
        Ok(AffidavitResult {
            command: format!("affi {label}"),
            success: output.status.success(),
            stdout,
            stderr,
            verdict,
        })
    }
}

// ── Pure mapping helpers (cargo-cicd ProcessEvent → affi emit args) ──────────

/// Directory affidavit receipts and working state live in: `<evidence_dir>/affidavit`.
pub fn affidavit_receipt_dir(evidence_dir: &Path) -> PathBuf {
    evidence_dir.join("affidavit")
}

/// Strip characters that would break affi's `--object ID:TYPE:QUAL` parsing.
fn sanitize_token(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| if c.is_whitespace() || c == ':' { '_' } else { c })
        .collect();
    if cleaned.is_empty() {
        "_".to_string()
    } else {
        cleaned
    }
}

/// Derive the `--type` value for an affi operation-event from a cargo-cicd
/// command + lifecycle, e.g. `("status show", "complete")` → `"status:show:complete"`.
pub fn event_type_for(command: &str, lifecycle: &str) -> String {
    let base = command.split_whitespace().collect::<Vec<_>>().join(":");
    let base = if base.is_empty() {
        "event".to_string()
    } else {
        base
    };
    if lifecycle.is_empty() {
        base
    } else {
        format!("{base}:{lifecycle}")
    }
}

/// Derive the `--object` token (`ID:TYPE:QUALIFIER`) for an affi operation-event
/// from a cargo-cicd [`ProcessEvent`]. The workspace is the touched object and
/// the claimed verdict is carried as the qualifier.
pub fn object_ref_for(event: &ProcessEvent) -> String {
    format!(
        "{}:workspace:{}",
        sanitize_token(&event.workspace_id),
        sanitize_token(&event.verdict_claimed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_dir_is_under_evidence() {
        let p = affidavit_receipt_dir(Path::new("target/cargo-cicd/evidence"));
        assert!(p.ends_with("affidavit"));
    }

    #[test]
    fn event_type_joins_command_and_lifecycle() {
        assert_eq!(event_type_for("status show", "complete"), "status:show:complete");
        assert_eq!(event_type_for("publish run", ""), "publish:run");
        assert_eq!(event_type_for("", "start"), "event:start");
    }

    #[test]
    fn object_ref_sanitizes_separators() {
        let mut ev = ProcessEvent::new("status show", "PASS");
        ev.workspace_id = "my ws:1".to_string();
        let obj = object_ref_for(&ev);
        // Exactly two ':' separators — components must not introduce more.
        assert_eq!(obj.matches(':').count(), 2, "object ref: {obj}");
        assert!(obj.ends_with(":PASS"));
    }

    #[test]
    fn verdict_display() {
        assert_eq!(AffidavitVerdict::Accept.to_string(), "ACCEPT");
        assert_eq!(AffidavitVerdict::Reject.to_string(), "REJECT");
        assert_eq!(AffidavitVerdict::Blocked.to_string(), "BLOCKED");
    }

    #[test]
    fn detect_is_graceful_when_absent() {
        // In an environment without `affi`, detection returns None rather than
        // panicking. If `affi` happens to be installed, version() must parse.
        if let Some(shell) = AffidavitShell::detect() {
            let _ = shell.version();
        }
    }
}
