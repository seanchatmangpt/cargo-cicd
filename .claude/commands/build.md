# /build

Trigger: user requests a build or binary verification.

## Steps

```bash
cargo make --version 2>/dev/null && cargo make build || cargo build
```

1. If `cargo make` exits 0 → run `cargo make build`. Else → run `cargo build`.
2. On non-zero exit: print first 20 `error[` lines + compiler error codes. Stop.
3. On zero exit: verify binary exists at `target/debug/cargo-cicd` (or `target/release/` if `--release`).
4. Binary missing despite exit 0 → FAIL.
5. Run `./target/debug/cargo-cicd --version` and capture version string.

## Output format

```
Build succeeded in <Ns> using `<command>`.
Warnings: N  [list codes if N>0]
Binary: <abs-path> (<size>)
Version: cargo-cicd <ver>
```

On failure:
```
Build FAILED using `<command>`.
error[E####]: <message> in <file>:<line>
Suggestion: cargo check for faster feedback
```

## Facts

- Binary declared: `[[bin]] name = "cargo-cicd" path = "src/main.rs"`
- Workspace members: root, `crates/cargo-cicd-core`, `crates/cargo-cicd-lsp`
- Feature flags off by default: `process-data`, `autonomic`, `wasm4pm`
- Feature-gated builds: `cargo build --features autonomic` or `--features wasm4pm`
- `Makefile.toml` is canonical; always prefer `cargo make` when available
