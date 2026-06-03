//! RuntimeCourtAnalyzer — raises CICD-WPM-001 through CICD-WPM-003.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;
use cargo_cicd_core::wpm::WpmCapabilityCache;

use super::CicdAnalyzer;

/// Verifies wpm capability cache, binary availability, and runtime court invocation.
pub struct RuntimeCourtAnalyzer;

impl CicdAnalyzer for RuntimeCourtAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        // Check capability cache on disk.
        let cache_path = snapshot
            .root
            .join("target")
            .join("cargo-cicd")
            .join("wpm-capability-cache.json");

        let cache_exists = cache_path.exists();

        // CICD-WPM-002: capability scan cache missing.
        if !cache_exists {
            findings.push(CicdFinding::new(
                CicdCode::WpmCommandUnavailable,
                cache_path.to_string_lossy().as_ref(),
                "cargo cicd wpm scan",
                vec!["cargo cicd wpm scan".to_string()],
                "wpm capability scan cache is missing. Run `cargo cicd wpm scan` to populate it.",
            ));
        }

        // Rebuild a fresh detection (cheap — only checks path existence).
        let cap = WpmCapabilityCache::detect();

        // CICD-WPM-001: wpm binary not confirmed available.
        if !cap.is_available {
            findings.push(CicdFinding::new(
                CicdCode::WpmUnconfirmedReceiptCourt,
                "PATH / WPM_BIN",
                "cargo cicd evidence doctor",
                vec!["cargo cicd evidence doctor".to_string()],
                "wpm binary not confirmed available. \
                 Set WPM_BIN or build wasm4pm. No release may claim ALIVE without wpm court adjudication.",
            ));
        }

        // CICD-WPM-003: runtime court has never been invoked (no audit event in evidence).
        // We detect this by looking for a wpm-audit event marker in the JSONL evidence log.
        let evidence_dir = snapshot
            .root
            .join("target")
            .join("cargo-cicd")
            .join("evidence");
        let jsonl_path = evidence_dir.join("events.jsonl");

        let court_invoked = if jsonl_path.exists() {
            std::fs::read_to_string(&jsonl_path)
                .map(|content| {
                    content.contains("\"wpm_audit\"")
                        || content.contains("\"wpm:audit\"")
                        || content.contains("\"court:adjudicate\"")
                        || content.contains("wpm_court_invoked")
                })
                .unwrap_or(false)
        } else {
            false
        };

        if !court_invoked {
            findings.push(CicdFinding::new(
                CicdCode::WpmRuntimeCourtNotInvoked,
                jsonl_path.to_string_lossy().as_ref(),
                "wpm audit",
                vec![
                    "wpm audit target/cargo-cicd/evidence/".to_string(),
                    "cargo cicd evidence doctor".to_string(),
                ],
                "Runtime court has never been invoked — no audit event found in process evidence. \
                 No release may claim ALIVE solely from internal tests.",
            ));
        }

        findings
    }

    fn name(&self) -> &'static str {
        "RuntimeCourtAnalyzer"
    }
}
