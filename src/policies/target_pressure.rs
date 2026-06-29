use super::{CicdPolicy, PolicyMode, PolicyResult};

pub struct TargetPressurePolicy {
    pub max_gb: f64,
}

impl Default for TargetPressurePolicy {
    fn default() -> Self {
        Self { max_gb: 20.0 }
    }
}

impl CicdPolicy for TargetPressurePolicy {
    fn name(&self) -> &'static str {
        "target_pressure"
    }
    fn enabled(&self) -> bool {
        true
    }
    fn mode(&self) -> PolicyMode {
        PolicyMode::Suggest
    }
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let size_gb = state.target.total_size_bytes as f64 / 1_000_000_000.0;
        let (verdict, rec) = if size_gb >= self.max_gb {
            (
                "alert",
                Some(format!(
                    "target/ is {:.1}GB (limit {}GB) — run 'cargo cicd target prune'",
                    size_gb, self.max_gb
                )),
            )
        } else if size_gb >= self.max_gb * 0.7 {
            (
                "warn",
                Some(format!(
                    "target/ is {:.1}GB ({:.0}% of limit) — consider pruning soon",
                    size_gb,
                    size_gb / self.max_gb * 100.0
                )),
            )
        } else {
            ("pass", None)
        };
        PolicyResult {
            name: self.name().into(),
            enabled: true,
            mode: "suggest".into(),
            verdict: verdict.into(),
            recommendation: rec,
            event_kind: "target_pressure".into(),
        }
    }
}
