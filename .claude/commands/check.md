# /check

Trigger: user requests lint or type-check.

## Steps

```bash
cargo make --version 2>/dev/null && cargo make check || (cargo check 2>&1 && cargo clippy -- -D warnings 2>&1)
```

1. Collect all stdout+stderr.
2. Categorize:
   - **Errors**: lines matching `error[E` or `error:`
   - **Clippy warnings**: lines matching `warning[clippy::`
   - **Compiler warnings**: lines matching `warning[` (not clippy)
3. Emit summary: `Check result: N errors, M clippy warnings, K compiler warnings`

## Act-on-these clippy lints

| Lint | Fix |
|------|-----|
| `clippy::unwrap_used` | Adapters: `.unwrap_or_default()`. Handlers: `?` propagation |
| `clippy::expect_used` | Same as unwrap_used |
| `clippy::panic` | Forbidden in adapters and verb handlers |
| `clippy::todo` / `clippy::unimplemented` | Remove before release |
| `clippy::clone_on_ref_ptr` | Use `Arc::clone(&ptr)` |

## Expected noise (low priority)

- `clippy::missing_docs` — ontology-generated; not required on internals
- `clippy::module_name_repetitions` — noun modules intentionally repeat name
- `dead_code` on `#[cfg(feature)]` items — expected when feature is off

## Error code fixes

| Code | Fix |
|------|-----|
| `unused_imports` | Remove `use` or gate with `#[cfg(feature)]` |
| `dead_code` | Delete or add feature gate |
| `E0308` | Check `EngineState` field types (`total_size_bytes` is `u64` not `usize`) |
| `E0412` | Verify adapter exported in `src/adapters/mod.rs` via `pub use` |
| `E0432`/`E0433` | Declare module in `mod.rs` before importing |

## Forbidden term risk

Any change to `src/nouns/*.rs` or `src/main.rs` string literals → run:
```bash
cargo test --test invariants
```

Forbidden: `ALIVE` `Nehemiah` `CONSTRUCT8` `Instinct8` `Inspection Gate` `Cargo Court` `AGI` `Truex` `Field8` `wall`

## Feature-gated errors

If baseline check has errors, also run:
```bash
cargo check --features process-data
cargo check --features autonomic
cargo check --features wasm4pm
cargo clippy --features autonomic -- -D warnings
```

`autonomic` implies `process-data`; `wasm4pm` implies `process-data`.

## Adapter contract

All `src/adapters/*.rs` methods must be infallible from callers — return defaults on failure, never panic. Any `unwrap` in adapter files = must fix.
