/// Adapters translate between external representations and the internal state model.
/// Each adapter is responsible for a single external source (git, cargo metadata, filesystem).
// New boundary adapters
pub mod cargo_metadata;
pub mod changed_file_detector;
pub mod cicd_toml_writer;
pub mod fs;
pub mod git_status;
pub mod manifest_parser;
pub mod target_scanner;
pub mod toolchain_detector;
pub mod trybuild_detector;

pub use cargo_metadata::CargoMetadataAdapter;
pub use changed_file_detector::ChangedFileDetector;
pub use cicd_toml_writer::CicdTomlWriter;
pub use git_status::GitStatusAdapter;
pub use manifest_parser::{
    parse_package_name, parse_workspace_members, parse_workspace_package_metadata,
};
pub use target_scanner::TargetScannerAdapter;
pub use toolchain_detector::ToolchainDetector;
pub use trybuild_detector::TrybuildDetector;

// Advanced capability integrations (feature-gated)
#[cfg(feature = "advanced")]
pub mod cached;
#[cfg(feature = "advanced")]
pub mod fingerprint;
#[cfg(feature = "advanced")]
pub mod governance_patterns;
#[cfg(feature = "advanced")]
pub mod state_snapshot;
