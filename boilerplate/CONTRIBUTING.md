# Contributing to cargo-project

## Quick start

Five commands to be productive:

```sh
# 1. Clone and enter the repo
git clone https://github.com/seanchatmangpt/cargo-project && cd cargo-project

# 2. Install the required toolchain (pinned in rust-toolchain.toml)
rustup show

# 3. Install git hooks (runs fmt + clippy on commit; tests on push)
./scripts/install-hooks.sh

# 4. Run the full test suite
cargo make test

# 5. Run the binary
cargo run -- --help
```

After step 3 your commits are gated by `cargo fmt --check` and `cargo clippy`, and pushes are gated by `cargo test --workspace`.

---

## Development workflow

### Branch naming

| Prefix | Use for |
|--------|---------|
| `feat/` | New features |
| `fix/` | Bug fixes |
| `chore/` | Build, deps, tooling |
| `docs/` | Documentation only |
| `refactor/` | Code restructuring with no behaviour change |
| `test/` | Test additions or fixes |

Branch names should be lowercase and hyphenated: `feat/add-publish-verb`.

WIP branches may use `wip/` or `draft/` — the pre-push hook skips these automatically.

### Commit message format

```
<type>(<scope>): <short description>

[optional body — wrap at 72 chars]

[optional footer: Refs #123, Breaking-Change: ...]
```

**Type** must be one of: `feat`, `fix`, `docs`, `test`, `ci`, `chore`, `refactor`, `perf`.

**Scope** (optional) should match the part of the codebase changed:

| Scope | Covers |
|-------|--------|
| `cli` | `src/nouns/`, `src/main.rs` |
| `core` | `crates/core/` |
| `config` | `crates/config/` |
| `ci` | `.github/`, `scripts/` |

Examples:
```
feat(cli): add target prune --dry-run flag
fix(core): correct off-by-one in changed-file detector
test(cli): add integration test for publish run
chore: upgrade clap to 4.5
ci: cache cargo registry in release workflow
```

The `commit-msg` hook validates format automatically and rejects non-conforming messages.

---

## Running tests

### Unit and integration tests

```sh
# All tests, default features
cargo test --workspace

# All tests via cargo-make (matches CI exactly)
cargo make test

# Single test binary
cargo test --test invariants

# Single test function
cargo test --test cli test_status_show_exits_zero

# With verbose output
cargo test --workspace -- --nocapture
```

### Feature flag tests

```sh
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
cargo test --features advanced
cargo test --features advanced,autonomic   # combinations work
```

### Snapshot tests

This project uses `cargo-insta` for snapshot testing.

```sh
# Run tests and review new/changed snapshots interactively
cargo insta test --review

# Accept all pending snapshots non-interactively (CI only)
cargo insta test --accept

# View pending snapshots without running tests
cargo insta review
```

Snapshot files live next to the test file as `<test_name>.snap`. Commit them alongside the code change that produces them.

---

## CI checks

Every PR runs the following jobs (see `.github/workflows/`):

| Job | Command | Local equivalent |
|-----|---------|-----------------|
| `check` | `cargo make check` | `cargo make check` |
| `test` | `cargo make test` | `cargo make test` |
| `test-features` | `cargo test --all-features` | `cargo test --all-features` |
| `clippy` | `cargo clippy --all-features -- -D warnings` | `cargo clippy --all-features -- -D warnings` |
| `fmt` | `cargo fmt --check` | `cargo fmt --check` |
| `deny` | `cargo deny check` | `cargo deny check` |
| `docs` | `cargo doc --no-deps` | `cargo doc --no-deps` |

Run `cargo make ci` to execute the full local CI equivalent before opening a PR.

---

## Adding a new crate

1. Create the crate under `crates/`:
   ```sh
   cargo new --lib crates/my-crate
   ```

2. Add it to the workspace in the root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
     ".",
     "crates/core",
     "crates/my-crate",   # add here
   ]
   ```

3. Add it as a dependency where needed:
   ```toml
   # In the crate that depends on it
   [dependencies]
   my-crate = { path = "../../crates/my-crate" }
   ```

4. If the crate is feature-gated, declare the feature in the root `Cargo.toml`:
   ```toml
   [features]
   my-feature = ["my-crate"]
   ```

5. Add CI coverage: ensure the new crate is covered by `--workspace` tests. If it introduces a new optional feature, add it to the `test-features` matrix in `.github/workflows/ci.yml`.

---

## Release process

Releases are managed by maintainers only (write access to `main` required).

```sh
# Bump version, update CHANGELOG, create git tag
./scripts/release.sh <major|minor|patch>

# Example: patch release
./scripts/release.sh patch
```

The script:
1. Bumps the version in all `Cargo.toml` files (workspace-wide)
2. Updates `CHANGELOG.md` with the new version header
3. Runs `cargo make ci` to verify everything passes
4. Creates a signed `vX.Y.Z` tag and pushes to `origin`

**CHANGELOG entry format:**

```markdown
## [0.4.1] — 2026-06-20

### Added
- `target prune --dry-run` flag (#42)

### Fixed
- Status command now correctly reports dirty files when index is empty (#39)

### Changed
- Minimum Rust version bumped to 1.86
```

Do not edit `CHANGELOG.md` manually for releases — `release.sh` handles the header. Do add entries under `## [Unreleased]` as you merge PRs.

---

## Code style

### rustfmt

Configuration lives in `rustfmt.toml` at the workspace root. Run `cargo fmt` before committing (the pre-commit hook enforces this).

Notable settings: `edition = "2021"`, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`.

### clippy

Configuration lives in `clippy.toml`. CI denies all warnings (`-D warnings`). The pre-commit hook allows pedantic lints to keep iteration fast; the pre-push hook and CI use `--all-features -D warnings`.

When `#[allow(...)]` is necessary, always add a comment explaining why:

```rust
// SAFETY: we verified the pointer is non-null and aligned above
#[allow(clippy::cast_ptr_alignment)]
let val = unsafe { ptr.cast::<u32>().read() };
```

Never use `#[allow(clippy::all)]` or `#[allow(warnings)]` — prefer targeted suppressions.

### Doc comments

Public items (`pub`, `pub(crate)`) require doc comments. Follow this pattern:

```rust
/// Short one-line summary (no period at the end unless multi-sentence).
///
/// Longer explanation if needed. Use `backticks` for code references.
///
/// # Errors
///
/// Returns [`Error::NotFound`] when the workspace root cannot be located.
pub fn workspace_name() -> Result<String> { ... }
```

Private helpers do not require doc comments but benefit from a `//` line explaining non-obvious intent.
