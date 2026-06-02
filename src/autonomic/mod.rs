/// Autonomic layer: policy evaluation and automatic remediation suggestions.
/// Only active when the `autonomic` feature is enabled.

pub mod policy_engine;
pub mod signals;
pub mod policies;

pub use policies::{AutomicPolicy, PolicyMode, PolicyVerdict, run_all_policies};
