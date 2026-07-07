use super::{CicdPolicy, PolicyResult};

pub struct ToolchainMismatchPolicy;

impl CicdPolicy for ToolchainMismatchPolicy {
    fn name(&self) -> &'static str {
        "toolchain_mismatch"
    }
    fn enabled(&self) -> bool {
        true
    }
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let active = &state.toolchain.active;
        let required = if std::path::Path::new("rust-toolchain.toml").exists() {
            std::fs::read_to_string("rust-toolchain.toml")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.contains("channel"))
                        .map(|l| l.split('"').nth(1).unwrap_or("stable").to_string())
                })
        } else {
            None
        };

        let (verdict, rec) = match &required {
            Some(req) if !active.contains(req.as_str()) => (
                "warn",
                Some(format!("active toolchain '{}' does not match required '{}' — run 'rustup override set {}'", active, req, req))
            ),
            _ => ("pass", None),
        };
        PolicyResult {
            verdict: verdict.into(),
            recommendation: rec,
        }
    }
}
