# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces. It keeps your target
directory under control, surfaces changed tests before you push, and records workspace
state into `cicd.toml` so CI has a machine-readable picture of what happened locally.
It is built on [`clap-noun-verb`](https://crates.io/crates/clap-noun-verb), which enforces
a three-tier architecture separating CLI surface, integration wiring, and domain logic.

## Install

```sh
cargo install cargo-cicd
```

## Quick Start

```sh
cargo cicd status                  # workspace snapshot: toolchain, target size, git state
cargo cicd git status              # branch, staged, dirty, untracked counts
cargo cicd git close               # enforce clean tree before pushing
cargo cicd target show             # target directory size vs configured limits
cargo cicd target prune            # remove stale artifacts (--dry-run to preview)
cargo cicd test changed            # run only tests for files changed vs base branch
cargo cicd trybuild changed        # run only changed trybuild fixtures
cargo cicd workspace doctor        # health checks and autonomic policy suggestions
cargo cicd publish                 # write cicd.toml with current workspace state
```

## Architecture

`cargo-cicd` is structured in three tiers. Every command follows this pattern — no
exceptions. If you add a command, follow the same structure.

```
Tier 1 — PRESENTATION LAYER     NounCommand + VerbCommand traits
                                 CLI parsing and validation only
                                 No business logic here

Tier 2 — INTEGRATION LAYER      run() method on VerbCommand
                                 Calls into Tier 3, formats output, emits events
                                 May use adapters; must not contain domain rules

Tier 3 — DOMAIN LOGIC LAYER     Pure functions, no clap imports
                                 The actual workspace/git/target reasoning
                                 Independently testable
```

### How cargo-cicd's own commands use the three tiers

The `git status` command illustrates the separation:

**Tier 1 — NounCommand defines the namespace:**

```rust
pub struct GitNoun;

impl NounCommand for GitNoun {
    fn name(&self) -> &'static str { "git" }
    fn about(&self) -> &'static str { "Git phase management" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(GitStatusVerb), Box::new(GitCloseVerb)]
    }
}
```

`GitNoun` knows nothing about git. It owns the CLI namespace (`cargo cicd git …`) and
declares which verbs exist. That is all it does.

**Tier 1 — VerbCommand is pure validation:**

```rust
pub struct GitStatusVerb;

impl VerbCommand for GitStatusVerb {
    fn name(&self) -> &'static str { "status" }
    fn about(&self) -> &'static str { "Show git repository state" }
    fn run(&self, _args: &VerbArgs) -> Result<()> {
        // Tier 2: delegate to domain logic, then format output
        let status = git_status_query()?;
        print_git_status(&status);
        Ok(())
    }
}
```

The `run()` method is Tier 2. It calls a Tier 3 function and handles output. It does not
contain the git query logic itself.

**Tier 3 — Pure domain function (no CLI imports):**

```rust
// src/adapters/git_status.rs — no clap, no VerbCommand, no NounCommand

pub struct GitStatusAdapter;

impl GitStatusAdapter {
    pub fn query() -> Result<GitStatusResult> {
        // Runs `git status --porcelain`, parses output, returns a plain struct.
        // Independently testable. Called from multiple verbs if needed.
    }
}
```

`GitStatusAdapter::query()` has no knowledge of clap, no CLI argument parsing, and no
output formatting. It can be called from tests, from other verbs, or from `cicd.toml`
publication without touching the CLI layer.

### The wrong pattern — do not copy this

```rust
// BAD: a single run() that does everything
impl VerbCommand for GitStatusVerb {
    fn run(&self, args: &VerbArgs) -> Result<()> {
        // Parsing CLI args here
        // Running `git status --porcelain` here
        // Business logic deciding what "dirty" means here
        // Formatting output here
        // All in one method — untestable, uncacheable, unextendable
        Ok(())
    }
}
```

When logic lives in `run()`, it cannot be reused by `cicd.toml` publication, by policy
checks, or by other verbs. The three-tier split exists to prevent this.

## Process Data: cicd.toml

`cargo cicd publish` calls Tier 3 adapters (workspace scanner, git adapter, target
scanner, toolchain detector) and writes their outputs to `cicd.toml`. This file is the
machine-readable record of local workspace state — useful for CI pipelines that need
to know what happened before the push.

```toml
[workspace]
name = "my-crate"
toolchain = "stable-aarch64-apple-darwin"
target_dir = "target"

[state]
dirty = false
target_size_gb = 1.24

[target]
max_size_gb = 20
prune_after_days = 14
```

The `cicd.toml` values come entirely from Tier 3 functions. `publish` is a Tier 2
integration that assembles the outputs — it adds no logic of its own.

## Extending cargo-cicd

To add a new command, create three things:

1. **A `NounCommand` struct** — defines the CLI namespace and lists verbs. No logic.
2. **One or more `VerbCommand` structs** — each `run()` calls into Tier 3, then formats
   output. No domain reasoning in `run()`.
3. **A Tier 3 function or adapter** — pure logic, no clap imports, independently
   testable. Lives in `src/adapters/` or `src/engine/`.

Register the noun in `src/main.rs` via `CliBuilder`. That is the complete integration
path.

```rust
// src/nouns/lint.rs — a new command following the three-tier pattern

pub struct LintNoun;
impl NounCommand for LintNoun {
    fn name(&self) -> &'static str { "lint" }
    fn about(&self) -> &'static str { "Lint checks" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(LintRunVerb)]
    }
}

pub struct LintRunVerb;
impl VerbCommand for LintRunVerb {
    fn name(&self) -> &'static str { "run" }
    fn about(&self) -> &'static str { "Run clippy on changed files" }
    fn run(&self, args: &VerbArgs) -> Result<()> {
        // Tier 2: call Tier 3, format output
        let changed = ChangedFileDetector::query(args.base_branch())?;
        let results = run_clippy_on(&changed)?;   // <-- Tier 3 function
        print_lint_results(&results);
        Ok(())
    }
}

// src/adapters/clippy.rs — Tier 3, no clap
pub fn run_clippy_on(files: &[PathBuf]) -> Result<Vec<LintResult>> {
    // pure logic here
}
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
