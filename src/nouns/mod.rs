//! Noun-verb CLI grammar for cargo-cicd.
//!
//! Each module in this directory corresponds to one *noun* — a domain area
//! of the workspace (e.g. `target`, `git`, `evidence`). Within each noun,
//! *verbs* express specific operations (`show`, `prune`, `doctor`).
//!
//! The grammar is manufactured from an RDF ontology via `ggen` and dispatched
//! by `clap-noun-verb`. Adding a new noun requires:
//! 1. Defining it in `ontology/cargo-cicd-capabilities.ttl`
//! 2. Running `ggen` to regenerate scaffolding
//! 3. Implementing the verb handlers here
//!
//! Bare nouns inject default verbs automatically (e.g. `status` → `status show`).

pub mod evidence;
pub mod evidence_helpers;
pub mod git;
pub mod lsp;
pub mod pipeline;
pub mod process_helpers;
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
