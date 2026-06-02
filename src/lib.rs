//! cargo-cicd — Level 5 State Model for Rust CI/CD Pipelines

pub mod state;
pub mod cicd_toml;
pub mod adapters;
pub mod autonomic;

pub use cicd_toml::CicdToml;
pub use state::workspace::WorkspaceState;
