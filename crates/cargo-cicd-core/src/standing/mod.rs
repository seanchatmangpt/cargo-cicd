//! Standing schema v1 (`praxis-standing.v1`): artifact readiness tracking.
pub mod model;

pub use model::{
    compute_ladder_level, ArtifactKind, EvidenceRef, LadderLevel, StandingArtifact,
    StandingDocument, StandingError, StandingStatus,
};
