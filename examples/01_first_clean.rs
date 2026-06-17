//! Tutorial anchor for `docs/tutorials/01-first-clean-workspace.md`.
//!
//! Run:
//!   cargo run --example 01_first_clean
//!
//! What you will see: workspace name, git branch, and a CLEAN / DIRTY verdict.
//! No feature flags required.

use cargo_cicd::EngineState;

fn main() {
    let state = EngineState::from_workspace();

    println!("cargo-cicd — workspace snapshot");
    println!("  workspace : {}", state.workspace.name);
    println!("  branch    : {}", state.git_phase.branch);
    println!(
        "  toolchain : {}",
        if state.toolchain.rust_version.is_empty() {
            "unknown"
        } else {
            &state.toolchain.rust_version
        }
    );

    let dirty = state.git_phase.dirty_files.len();
    let staged = state.git_phase.staged_files.len();
    let untracked = state.git_phase.untracked_files.len();

    println!();
    if dirty == 0 && staged == 0 && untracked == 0 {
        println!("status: CLEAN — workspace is push-ready");
    } else {
        println!(
            "status: DIRTY — {} dirty, {} staged, {} untracked",
            dirty, staged, untracked
        );
        for f in &state.git_phase.dirty_files {
            println!("    M {f}");
        }
        for f in &state.git_phase.staged_files {
            println!("    S {f}");
        }
        for f in &state.git_phase.untracked_files {
            println!("    ? {f}");
        }
    }
}
