//! Ladder scoring: derive an artifact's `ladder_level` from its `standing`
//! set and `scope`, using the single source of truth in `model.rs`.

use crate::standing::model::{compute_ladder_level, StandingArtifact};

/// Recompute and set `artifact.ladder_level` in place from its current
/// `standing` and `scope`.
pub fn score_ladder(artifact: &mut StandingArtifact) {
    artifact.ladder_level = compute_ladder_level(&artifact.standing, artifact.scope.as_deref());
}

/// Score every artifact in a slice in place.
pub fn score_all(artifacts: &mut [StandingArtifact]) {
    for artifact in artifacts.iter_mut() {
        score_ladder(artifact);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standing::model::{ArtifactKind, StandingStatus};

    fn artifact(standing: Vec<StandingStatus>, scope: Option<&str>) -> StandingArtifact {
        StandingArtifact {
            id: "x".to_string(),
            kind: ArtifactKind::RustCrate,
            path: "x".to_string(),
            standing,
            scope: scope.map(|s| s.to_string()),
            ladder_level: 255, // deliberately wrong, to prove score_ladder overwrites it
            evidence: vec![],
            external_operator_side_effects: vec![],
        }
    }

    #[test]
    fn score_ladder_overwrites_stale_level() {
        let mut a = artifact(vec![StandingStatus::Tested], None);
        score_ladder(&mut a);
        assert_eq!(a.ladder_level, 2);
    }

    #[test]
    fn score_all_updates_every_artifact() {
        let mut artifacts = vec![
            artifact(vec![StandingStatus::Discovered], None),
            artifact(vec![StandingStatus::Wasm4pmProven], None),
        ];
        score_all(&mut artifacts);
        assert_eq!(artifacts[0].ladder_level, 0);
        assert_eq!(artifacts[1].ladder_level, 5);
    }
}
