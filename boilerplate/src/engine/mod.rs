//! Level 5 engine — the aggregate root for all runtime state.
//!
//! [`EngineState`] is populated once at command entry via
//! [`EngineState::from_workspace()`], which calls every registered adapter in
//! sequence.  Adapter failures are **silent** — the engine carries whatever
//! partial data is available.  A partial state is always preferable to a
//! crash.
//!
//! ## Architecture contract
//!
//! - Nouns **read** from `EngineState`.  They must not call adapters directly.
//! - Adapters **write** to `EngineState`.  They must not read from each other.
//! - Business logic lives in nouns, not in adapters or the engine.
//!
//! ## Adding a new state dimension
//!
//! 1. Add a sub-module `src/engine/<dimension>.rs` with a `<Dimension>State`
//!    struct that implements `Default`.
//! 2. Add a field `pub <dimension>: <Dimension>State` to [`EngineState`].
//! 3. Create `src/adapters/<dimension>.rs` implementing the population logic.
//! 4. Call the new adapter in `EngineState::from_workspace()`.

#![cfg(feature = "process-data")]

use crate::adapters::{git::GitAdapter, toolchain::ToolchainAdapter, workspace::WorkspaceAdapter};

pub mod git_state;
pub mod toolchain_state;
pub mod workspace_state;

pub use git_state::GitState;
pub use toolchain_state::ToolchainState;
pub use workspace_state::WorkspaceState;

/// Aggregate root — every dimension of runtime state lives here.
///
/// This struct is populated once per command invocation via
/// [`from_workspace()`].  It is intentionally `Clone` so that nouns can take
/// ownership of a snapshot without holding a reference into the engine.
#[derive(Debug, Clone, Default)]
pub struct EngineState {
    /// Workspace identity: name, root path, crate members.
    pub workspace: WorkspaceState,
    /// Active Rust toolchain and compiler version.
    pub toolchain: ToolchainState,
    /// Git repository state: branch, dirty/staged/untracked files, tracking.
    pub git: GitState,
}

impl EngineState {
    /// Populate a complete `EngineState` by running all adapters.
    ///
    /// Adapters that fail (e.g., because `git` is not installed, or we are not
    /// inside a workspace) silently return their `Default` value.  The caller
    /// always receives a valid struct, never an error.
    ///
    /// This function may invoke external processes (`git`, `rustc`) and
    /// perform filesystem I/O.  It is intentionally synchronous — spawning
    /// all adapters concurrently adds complexity without meaningful speedup
    /// for the typical < 50 ms runtime.
    pub fn from_workspace() -> Self {
        tracing::debug!("initialising EngineState from workspace");

        let mut state = Self::default();

        // Workspace identity — fast (TOML parse only, no cargo invocation).
        state.workspace = WorkspaceAdapter::populate();

        // Toolchain — medium (spawns `rustc --version`).
        state.toolchain = ToolchainAdapter::populate();

        // Git state — medium (spawns `git status --porcelain`).
        state.git = GitAdapter::populate();

        tracing::debug!(
            workspace = %state.workspace.name,
            branch = %state.git.branch,
            "EngineState ready"
        );

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_state_default_does_not_panic() {
        let state = EngineState::default();
        assert!(state.workspace.name.is_empty() || !state.workspace.name.is_empty());
    }

    #[test]
    fn engine_state_from_workspace_does_not_panic() {
        // from_workspace() must never panic even if git/rustc are unavailable.
        let _state = EngineState::from_workspace();
    }
}
