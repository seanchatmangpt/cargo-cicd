# /check

Run lint and type-checking across the cargo-cicd workspace, interpret the output, and report a summary with actionable next steps.

## Steps

1. **Check for cargo-make**

   Run `cargo make --version`. If `cargo-make` is available, use `cargo make check` as the preferred command. If unavailable, fall back to running both steps:
   ```
   cargo check
   cargo clippy -- -D warnings
   ```

2. **Run the check**

   Preferred command:
   ```
   cargo make check
   ```
   Fallback sequence (run both; collect all output before summarising):
   ```
   cargo check 2>&1
   cargo clippy -- -D warnings 2>&1
   ```

   Capture all stdout and stderr. Note whether each invocation exits zero or non-zero.

3. **Categorise the output**

   Parse the combined output and separate lines into:
   - **Errors** — lines matching `error[E` or `error:` (compilation failures, type errors, unresolved imports)
   - **Clippy warnings** — lines matching `warning[clippy::` (lint violations)
   - **Compiler warnings** — lines matching `warning[` but not `clippy::` (unused code, dead_code, etc.)

   For each category, count occurrences and extract the distinct diagnostic codes (e.g., `clippy::unwrap_used`, `unused_imports`, `E0308`).

4. **Report the summary line**

   Always print a one-line summary first:
   ```
   Check result: N errors, M clippy warnings, K compiler warnings
   ```

   If all counts are zero: "Check passed: 0 errors, 0 warnings."

5. **Highlight real issues vs. noise**

   Clippy warnings that indicate real problems in cargo-cicd (act on these):
   - `clippy::unwrap_used` — adapters must not panic; they silently fail and return defaults. Replace `.unwrap()` with `unwrap_or_default()` or `?` propagation.
   - `clippy::expect_used` — same reasoning as `unwrap_used`.
   - `clippy::panic` — forbidden in adapters and verb handlers; panics break the silent-failure contract.
   - `clippy::todo` or `clippy::unimplemented` — stubs that should not ship in released code.
   - `clippy::clone_on_ref_ptr` — signals incorrect ownership in state structs.

   Clippy warnings that are usually acceptable noise in this codebase:
   - `clippy::missing_docs` — docs are generated from the ontology; not all internal functions need doc comments.
   - `clippy::module_name_repetitions` — noun modules (e.g., `WorkspaceState` inside `workspace_state.rs`) intentionally repeat the module name.
   - `dead_code` on `#[cfg(feature = "...")]` items — feature-gated code appears dead when the feature is off; this is expected.

   When reporting, mark each finding as "act on this" or "expected / low priority".

6. **Mention the invariants test for public boundary issues**

   If errors or warnings touch any of the following, flag it explicitly:
   - Files in `src/nouns/` — these directly produce user-visible help text. Any string literal change risks introducing a forbidden term.
   - The string constants `ALIVE`, `Nehemiah`, `CONSTRUCT8`, `Instinct8`, `Inspection Gate`, `Cargo Court`, `AGI`, `Truex`, `Field8`, `wall` — these are forbidden in public output.
   - `src/main.rs` — contains `inject_default_verbs()` and top-level help text.

   In those cases, add: "Run `cargo test --test invariants` to verify the public boundary has not been broken."

7. **Check all feature variants if errors occurred**

   If `cargo check` or `cargo clippy` found errors, suggest checking feature-gated code separately, since errors in feature-gated paths only appear when the feature is enabled:
   ```
   cargo check --features process-data
   cargo check --features autonomic
   cargo check --features wasm4pm
   cargo clippy --features autonomic -- -D warnings
   ```
   Run these only if the baseline check had errors or if the user explicitly wants comprehensive coverage.

8. **Suggest fixes for common patterns**

   For each distinct error or warning code found, offer a one-line remediation hint:

   | Code / Lint | Suggestion |
   |---|---|
   | `unused_imports` | Remove the `use` line or gate it with `#[cfg(feature = "...")]` |
   | `dead_code` | Either delete the item, or gate it with a feature flag if it is intentionally unused by default |
   | `clippy::unwrap_used` | In adapters: replace with `.unwrap_or_default()`. In verb handlers: replace with `?` and propagate `anyhow::Result`. |
   | `clippy::expect_used` | Same as `unwrap_used`. Prefer `.unwrap_or_else(\|_\| default_value)` for adapters. |
   | `E0308` (type mismatch) | Check `EngineState` field types — e.g., `total_size_bytes` is `u64`, not `usize`. |
   | `E0412` (unresolved type) | Verify the adapter is imported in `src/adapters/mod.rs` via `pub use`. |
   | `E0432` / `E0433` (unresolved import) | Check that the module is declared in `mod.rs` before being imported elsewhere. |
   | `clippy::clone_on_ref_ptr` | Use `Arc::clone(&ptr)` instead of `.clone()` on reference-counted values. |

## Success output

When there are no errors and no high-priority warnings:
```
Check passed: 0 errors, 0 warnings.
cargo make check exited 0 in 2.1s.
No action needed.
```

When warnings exist but no errors:
```
Check result: 0 errors, 3 clippy warnings, 1 compiler warning.

Clippy warnings (act on these):
  - clippy::unwrap_used in src/adapters/target_scanner.rs:42 — replace .unwrap() with .unwrap_or_default()

Compiler warnings (low priority):
  - dead_code: field `pruned_bytes` in src/engine/target_state.rs:18 — expected when process-data feature is off

Run `cargo test --test invariants` if any noun files were changed.
```

## Failure output

When errors are present:
```
Check FAILED: 2 errors, 5 clippy warnings.

Errors:
  error[E0308]: mismatched types in src/adapters/target_scanner.rs:55
    expected u64, found usize — cast with `as u64`

  error[E0412]: cannot find type `CicdTomlWriter` in scope at src/engine/mod.rs:12
    add `use crate::adapters::CicdTomlWriter;` or check src/adapters/mod.rs

Next steps:
  1. Fix the errors above (type mismatches first, then missing imports).
  2. Re-run /check to confirm the fix.
  3. Then run /build to verify the binary compiles.
```

## cargo-cicd-specific notes

- **Adapter contract:** All adapter methods (`src/adapters/*.rs`) must be infallible from the caller's perspective — they return defaults on failure and never panic. Clippy warnings about `.unwrap()` in adapter files should always be fixed.
- **Evidence pattern:** `ProcessEvent` construction in `src/evidence.rs` uses string fields. Type errors here often mean a field was changed from `String` to `Option<String>` or vice versa.
- **Feature coupling:** `autonomic` implies `process-data`; `wasm4pm` implies `process-data`. Errors in `src/autonomic/` will only surface with `--features autonomic`.
- **Noun modules and forbidden terms:** `src/nouns/*.rs` files contain string literals that go into help text. The `invariants` test (`tests/invariants.rs`, function `invariant_public_boundary_no_forbidden_terms_in_all_help`) scans all `--help` output for the ten forbidden terms. Any lint-driven string change in noun files warrants running that test.
- **Generated code guard:** `tests/ggen_customization_guard.rs` checks that ggen-generated files have not drifted from the ontology. If you see unexpected changes in `src/nouns/` or `README.md`, run `ggen` to regenerate before assuming a lint fix is complete.
