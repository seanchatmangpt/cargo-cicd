/// Adapters translate between external representations and the internal state model.
/// Each adapter is responsible for a single external source (git, cargo metadata, filesystem).
// New boundary adapters
pub mod cargo_metadata;
pub mod changed_file_detector;
pub mod git_status;
pub mod target_scanner;
pub mod toolchain_detector;
pub mod trybuild_detector;

pub use cargo_metadata::CargoMetadataAdapter;
pub use changed_file_detector::ChangedFileDetector;
pub use git_status::GitStatusAdapter;
pub use target_scanner::TargetScannerAdapter;
pub use toolchain_detector::ToolchainDetector;
pub use trybuild_detector::TrybuildDetector;

// Note: the former `cached`, `fingerprint`, `governance_patterns`, and
// `state_snapshot` adapter shims were removed as orphaned scaffolding — they
// were never constructed by `EngineState::from_workspace()` or any noun, and
// their only exercise was self-contained unit tests validating their own
// logic in isolation. The underlying capabilities they wrapped
// (`advanced::cache`, `advanced::fingerprint`, `advanced::pattern`,
// `advanced::snapshot`) remain and are exercised directly by
// `examples/03_max_pipeline.rs`.
