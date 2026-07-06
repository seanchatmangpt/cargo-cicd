//! Standing schema v1 (`cicd-standing.v1`, alias `praxis-standing.v1`): artifact readiness tracking.
pub mod emit;
pub mod glob;
pub mod model;
pub mod score;
pub mod sources;

pub use model::{
    compute_ladder_level, is_recognized_schema_id, ArtifactKind, EvidenceRef, LadderLevel,
    StandingArtifact, StandingDocument, StandingError, StandingStatus, STANDING_SCHEMA_ID,
    STANDING_SCHEMA_ID_ALIAS_PRAXIS,
};
pub use score::{score_all, score_ladder};
