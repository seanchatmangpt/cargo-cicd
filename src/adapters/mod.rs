pub mod cargo_meta;
pub mod fs;
/// Adapters translate between external representations and the internal state model.
/// Each adapter is responsible for a single external source (git, cargo metadata, filesystem).
// Existing functional adapters (kept)
pub mod git;

// New boundary adapters
pub mod cargo_metadata;
pub mod changed_file_detector;
pub mod cicd_toml_writer;
pub mod git_status;
pub mod target_scanner;
pub mod toolchain_detector;
pub mod trybuild_detector;

// Named API surfaces (required by external callers)
pub mod changed_files;
pub mod target;
pub mod trybuild;

pub use cargo_metadata::CargoMetadataAdapter;
pub use changed_file_detector::ChangedFileDetector;
pub use cicd_toml_writer::CicdTomlWriter;
pub use git_status::GitStatusAdapter;
pub use target_scanner::TargetScannerAdapter;
pub use toolchain_detector::ToolchainDetector;
pub use trybuild_detector::TrybuildDetector;
