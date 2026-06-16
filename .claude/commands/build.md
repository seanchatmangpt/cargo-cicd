# /build

Run the cargo-cicd build pipeline and verify the binary was produced.

## Steps

1. **Check for cargo-make**

   Run `cargo make --version` to determine whether `cargo-make` (the `cargo-make` crate, invoked as `cargo make`) is available. If the command exits successfully, prefer `cargo make build`. If it is not installed or exits non-zero, fall back to `cargo build`.

2. **Run the build with timing**

   Preferred command:
   ```
   cargo make build
   ```
   Fallback command:
   ```
   cargo build
   ```

   Capture the full output (stdout + stderr). Note the wall-clock time from when the command starts to when it finishes, so you can report build duration to the user.

3. **Detect warnings and errors in the output**

   Scan the build output for lines that begin with `warning:` or `error:`. Collect them separately:
   - Warnings: lines starting with `warning[` or `warning:`
   - Errors: lines starting with `error[` or `error:`

   If there are warnings, print a summary: "Build succeeded with N warning(s)." listing each unique warning code (e.g., `unused_imports`, `dead_code`).

   If there are errors, print the relevant lines (up to 20) and report the build as failed. Stop here; do not check the binary.

4. **Verify the binary was produced**

   After a successful build, check that the `cargo-cicd` binary exists at the expected location:
   ```
   target/debug/cargo-cicd
   ```
   (or `target/release/cargo-cicd` if `--release` was part of the build).

   If the binary is present, report its absolute path and file size. If it is missing despite a zero exit code, treat this as a failure and say so explicitly.

5. **Confirm the binary is executable and identifies itself**

   Run:
   ```
   ./target/debug/cargo-cicd --version
   ```
   Report the version string. This confirms the binary links and initialises correctly.

## Success output

Report to the user:
- Build command used (cargo make build or cargo build)
- Build duration (e.g., "Compiled in 4.3s")
- Warning count (0 or more), with a brief list if any
- Binary path (absolute)
- Binary size in MB or KB
- Version string from `--version`

Example:
```
Build succeeded in 4.3s using `cargo make build`.
Warnings: 0
Binary: /home/user/cargo-cicd/target/debug/cargo-cicd (3.2 MB)
Version: cargo-cicd 26.6.2
```

## Failure output

If the build fails, report:
- Which command was run
- The first error line(s) with context
- The Rust compiler error code if present (e.g., `E0308`)
- A suggestion to run `cargo check` for faster feedback on type and lint errors

## cargo-cicd-specific notes

- The primary binary is declared in `Cargo.toml` as `[[bin]] name = "cargo-cicd" path = "src/main.rs"`.
- The workspace has three members: the root crate, `crates/cargo-cicd-core`, and `crates/cargo-cicd-lsp`. Errors in sub-crates will surface during the root build.
- Feature flags (`process-data`, `autonomic`, `wasm4pm`) are off by default. If you want to verify a feature-gated build, run `cargo build --features autonomic` or `cargo build --features wasm4pm` separately.
- The `Makefile.toml` (cargo-make config) is the canonical build entry point; always prefer it over bare `cargo build` when available.
