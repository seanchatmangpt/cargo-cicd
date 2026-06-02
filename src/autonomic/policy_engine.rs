use crate::state::policy::{PolicyMode, PolicyState, PolicyVerdict};

/// Evaluate a single policy against a set of signals.
pub fn evaluate_policy(name: &str, mode: PolicyMode, signals: Vec<String>) -> PolicyState {
    let (verdict, recommendation) = match mode {
        PolicyMode::Disabled => (PolicyVerdict::Pass, "policy disabled".to_string()),
        PolicyMode::Suggest | PolicyMode::Apply => {
            if signals.is_empty() {
                (PolicyVerdict::Pass, "no signals — clean".to_string())
            } else {
                let rec = format!(
                    "address {} signal(s): {}",
                    signals.len(),
                    signals.join(", ")
                );
                (PolicyVerdict::Warn, rec)
            }
        }
    };

    PolicyState {
        name: name.to_string(),
        mode,
        signals,
        recommendation,
        verdict,
    }
}
