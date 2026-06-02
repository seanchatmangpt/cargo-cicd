use super::{CicdPolicy, PolicyMode, PolicyResult};
use crate::adapters::ToolchainDetector;

pub struct ToolchainMismatchPolicy;

impl CicdPolicy for ToolchainMismatchPolicy {
    fn name(&self) -> &'static str { "toolchain_mismatch" }
    fn enabled(&self) -> bool { true }
    fn mode(&self) -> PolicyMode { PolicyMode::Suggest }
    fn evaluate(&self) -> PolicyResult {
        let active = ToolchainDetector::active_toolchain();
        let required = if std::path::Path::new("rust-toolchain.toml").exists() {
            std::fs::read_to_string("rust-toolchain.toml").ok()
                .and_then(|s| s.lines().find(|l| l.contains("channel")).map(|l| l.split('"').nth(1).unwrap_or("stable").to_string()))
        } else { None };

        let (verdict, rec) = match &required {
            Some(req) if !active.contains(req.as_str()) => (
                "warn",
                Some(format!("active toolchain '{}' does not match required '{}' — run 'rustup override set {}'", active, req, req))
            ),
            _ => ("pass", None),
        };
        PolicyResult { name: self.name().into(), enabled: true, mode: "suggest".into(), verdict: verdict.into(), recommendation: rec, event_kind: "toolchain_mismatch".into() }
    }
}
