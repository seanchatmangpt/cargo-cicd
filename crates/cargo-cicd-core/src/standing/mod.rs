//! Standing schema v1 (`praxis-standing.v1`): artifact readiness tracking.
pub mod emit;
pub mod glob;
pub mod model;
pub mod score;
pub mod sources;

pub use model::{
    compute_ladder_level, ArtifactKind, EvidenceRef, LadderLevel, StandingArtifact,
    StandingDocument, StandingError, StandingStatus,
};
pub use score::{score_all, score_ladder};
