pub mod evidence;
pub mod git;
pub mod lsp;
pub mod pipeline;
pub mod publish;
pub mod status;
pub mod target;
pub mod test;
pub mod trybuild;
pub mod ui;
pub mod workspace;

// Advanced nouns (feature-gated)
#[cfg(feature = "advanced")]
pub mod analyze;
