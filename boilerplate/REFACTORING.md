# Refactoring an existing Rust project using this boilerplate

This guide covers what to keep, what to adapt, what to drop, and a step-by-step migration path.

---

## Rename checklist

Ten find-and-replace operations to rename this boilerplate for your project. Run them in order from the repository root.

```sh
# 1. Binary name
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" -o -name "*.yml" \) \
  | xargs sed -i 's/cargo-project/your-binary-name/g'

# 2. Crate / library name (underscored form used in Rust code)
find . -type f \( -name "*.toml" -o -name "*.rs" \) \
  | xargs sed -i 's/cargo_project/your_crate_name/g'

# 3. Human-readable project name in docs/README
find . -type f \( -name "*.md" -o -name "*.toml" \) \
  | xargs sed -i 's/cargo-project/your-project-name/g'

# 4. GitHub username / org
find . -type f \( -name "*.yml" -o -name "*.toml" -o -name "*.md" \) \
  | xargs sed -i 's/seanchatmangpt/your-github-username/g'

# 5. GitHub repository name (if different from binary name)
find . -type f \( -name "*.yml" -o -name "*.md" \) \
  | xargs sed -i 's|seanchatmangpt/cargo-project|your-github-username/your-repo|g'

# 6. Package description in root Cargo.toml
#    Edit manually: description = "Your project description"

# 7. License (if changing from MIT)
#    Edit Cargo.toml: license = "Apache-2.0"
#    Replace LICENSE file contents

# 8. Rust edition (if not 2021)
find . -name "Cargo.toml" | xargs sed -i 's/edition = "2021"/edition = "2024"/g'

# 9. MSRV — minimum supported Rust version
find . -name "Cargo.toml" -o -name "rust-toolchain.toml" -o -name "*.yml" \
  | xargs sed -i 's/1\.86/1.XX/g'

# 10. Verify no boilerplate strings remain
grep -r "cargo.project\|seanchatmangpt\|cargo-project" \
  --include="*.toml" --include="*.rs" --include="*.yml" --include="*.md" \
  | grep -v "target/" | grep -v REFACTORING.md
```

---

## What to keep (drop-in files)

These files are designed to work unchanged for any Rust workspace. Copy them verbatim:

| File / Directory | Why it's drop-in |
|-----------------|-----------------|
| `.github/workflows/ci.yml` | Runs `check`, `test`, `fmt`, `clippy`, `deny` on all PRs |
| `.github/workflows/release.yml` | Publishes to crates.io on `vX.Y.Z` tags |
| `.github/dependabot.yml` | Weekly cargo + actions dependency updates |
| `.git-hooks/commit-msg` | Validates conventional commits |
| `.git-hooks/pre-commit` | `fmt --check` + `clippy` on staged files |
| `.git-hooks/pre-push` | `cargo test --workspace` before push |
| `.git-hooks/prepare-commit-msg` | Injects scope hints into blank commit messages |
| `scripts/install-hooks.sh` | Installs the above hooks |
| `scripts/install-dev-tools.sh` | Installs `cargo-make`, `cargo-deny`, `cargo-insta` |
| `deny.toml` | Audits dependencies for licenses, vulnerabilities, duplicates |
| `rustfmt.toml` | Formatting config (edition 2021, grouped imports) |
| `clippy.toml` | Clippy threshold config |
| `rust-toolchain.toml` | Pins stable toolchain channel |
| `Makefile.toml` | `build`, `check`, `test`, `fmt`, `ci`, `pre-push` tasks |

---

## What to adapt

These files have the right structure but need domain-specific content replaced:

### `src/nouns/`

This is the CLI grammar layer. Delete all existing noun modules and replace with your own. Each noun follows the same pattern:

```
src/nouns/your_noun.rs   — NounCommand impl + VerbCommand impls
src/nouns/mod.rs         — pub mod your_noun; + registration
src/main.rs              — inject_default_verbs() entry
```

Keep `src/nouns/mod.rs` registration boilerplate; replace the list of nouns.

### `crates/core/src/`

Entity types, event types, and domain logic live here. Keep the crate structure (`lib.rs`, `error.rs`, `Cargo.toml`) and replace the domain types:

- `entity.rs` — Replace with your aggregate roots
- `event.rs` — Replace with your domain events
- `repository.rs` — Adapt to your storage interface

### `crates/config/src/schema.rs`

Replace the `Config` struct fields with your application's configuration keys. The serialization infrastructure (serde, `config::Config` loading from env + file) is reusable.

### `README.md`

Replace the Quick Start section, command reference, and badges. Keep the structure (install, usage, config, contributing).

### `CHANGELOG.md`

Delete all existing entries. Keep the `## [Unreleased]` header and the format.

---

## What to drop

Remove these if they don't apply to your project:

| Component | Remove when |
|-----------|-------------|
| `crates/mcp-server/` | You don't need an MCP (Model Context Protocol) server |
| `crates/wasm/` | You don't need a WASM compilation target |
| `tui` feature + `crates/tui/` | You don't need a terminal UI |
| `crates/cargo-cicd-lsp/` | You don't need a Language Server |
| `.github/workflows/wasm.yml` | No WASM target |
| `scripts/build-wasm.sh` | No WASM target |
| `scripts/docker-build.sh` | No Docker publishing |
| `scripts/generate-man.sh` | No man page generation |

To drop a crate cleanly:
1. Remove the directory: `rm -rf crates/my-crate`
2. Remove it from `[workspace.members]` in root `Cargo.toml`
3. Remove any `[dependencies]` references to it
4. Remove associated feature flags
5. Run `cargo check` to confirm no dangling references

---

## Step-by-step migration

Migrate an existing Rust project into this boilerplate structure:

1. **Copy your source code in.** Move your existing `src/` into `src/` here (or into `crates/core/src/` if it's a library). Resolve any `Cargo.toml` dependency conflicts.

2. **Run the rename checklist** above (10 sed commands).

3. **Install the dev toolchain and hooks:**
   ```sh
   ./scripts/install-dev-tools.sh
   ./scripts/install-hooks.sh
   ```

4. **Get the build green:**
   ```sh
   cargo build 2>&1 | head -40
   ```
   Fix compilation errors before anything else.

5. **Get fmt and clippy clean:**
   ```sh
   cargo fmt
   cargo clippy --workspace --all-targets -- -D warnings -A clippy::pedantic
   ```
   Fix or `#[allow(...)]` (with comment) each remaining warning.

6. **Port your tests.** Move existing tests into `tests/` using `assert_cmd` + `tempfile` patterns (see `tests/cli/` for examples). Snapshot tests go through `cargo-insta`.

7. **Get `cargo make test` green:**
   ```sh
   cargo make test
   ```

8. **Add `deny.toml` entries for your licenses.** Open `deny.toml` and add your dependency licenses to the `[licenses]` allow-list. Run `cargo deny check` and fix any issues.

9. **Wire CI.** Push a branch and verify the CI workflows in `.github/workflows/ci.yml` pass. Check the Actions tab for the first run.

10. **Update `README.md`** with your project's install instructions, usage examples, and badges. Run `cargo doc --no-deps --open` to preview generated API docs.

At this point the project structure, CI, hooks, and tooling match the boilerplate. Iterate on domain logic from here.
