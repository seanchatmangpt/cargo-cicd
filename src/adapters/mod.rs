/// Adapters translate between external representations and the internal state model.
/// Each adapter is responsible for a single external source (git, cargo metadata, filesystem).

// Existing functional adapters (kept)
pub mod git;
pub mod cargo_meta;
pub mod fs;

// New boundary adapters
pub mod cargo_metadata;
pub mod git_status;
pub mod target_scanner;
pub mod changed_file_detector;
pub mod trybuild_detector;
pub mod toolchain_detector;
pub mod cicd_toml_writer;

pub use cargo_metadata::CargoMetadataAdapter;
pub use git_status::GitStatusAdapter;
pub use target_scanner::TargetScannerAdapter;
pub use changed_file_detector::ChangedFileDetector;
pub use trybuild_detector::TrybuildDetector;
pub use toolchain_detector::ToolchainDetector;
pub use cicd_toml_writer::CicdTomlWriter;
