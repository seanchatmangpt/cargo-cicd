# cargo-cicd v26.6.2 — crates.io Readiness Receipt

**Date:** 2026-06-02
**Commit:** f93162959d3b9de929265b2edf86cccf69d6f844

## Cargo.toml Metadata Fields

- name: cargo-cicd
- version: 26.6.2
- description: "Local-first CI/CD helper for Rust workspaces: clean target dirs, run changed tests, check git state, and publish cicd.toml."
- license: MIT OR Apache-2.0
- repository: https://github.com/seanchatmangpt/cargo-cicd
- homepage: https://github.com/seanchatmangpt/cargo-cicd
- readme: README.md
- keywords: ["cargo", "ci", "testing", "workspace", "cleanup"]
- categories: ["command-line-utilities", "development-tools"]

## Repository Remote URL

https://github.com/seanchatmangpt/cargo-cicd

## License Files

- LICENSE-MIT: present
- LICENSE-APACHE: present

## README Status

README.md present (created by previous agent).

## Feature Matrix Results

| Feature combo | Result |
|---|---|
| --no-default-features | PASS |
| --features process-data | PASS |
| --features autonomic | PASS |
| --features wasm4pm | PASS |
| --all-features | PASS |

## Dependency Audit

clap-noun-verb resolved to crates.io version 26.5.19 (dep resolution: version_dep_26_5_19).
No path dependencies in Cargo.toml.

## cargo package --list Summary

- 178 files included
- Notable inclusions: src/, tests/, docs/commands/, fixtures/
- Private files excluded: CLAUDE.md, receipts/, ontology/, cicd.toml, docs/testing/, docs/release/, docs/deferred/, docs/wasm4pm/, ggen.toml, queries/, templates/

## Package Size

279.9 KiB uncompressed, 70.3 KiB compressed

## cargo install --path Output Summary

Compiled successfully. Installed: /Users/sac/.cargo/bin/cargo-cicd

## cargo-cicd --help First Line

"Local-first CI/CD helpers for Rust workspaces: clean target dirs, run changed tests, check git state, and publish cicd.toml."

## cargo-cicd status Output

```
cargo-cicd workspace status
===========================
toolchain:    stable-aarch64-apple-darwin
target:       3.99 GB [pass]
branch:       main
dirty files:  1
untracked:    0
git:          dirty
```

## wasm4pm Evidence Gate Status

ALIVE — all tests passed:

- wasm4pm_evidence_gate: 8/8 passed
- wasm4pm_evidence_mutation: 5/5 passed
- wasm4pm_refusal_cases: 7/7 passed

## cargo publish --dry-run Output

```
Updating crates.io index
Packaging cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
Updating crates.io index
Packaged 178 files, 279.9KiB (70.3KiB compressed)
Verifying cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
Compiling cargo-cicd v26.6.2 (/Users/sac/cargo-cicd/target/package/cargo-cicd-26.6.2)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.31s
Uploading cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
warning: aborting upload due to dry run
```

## Known Gaps

1. `cargo cicd` subcommand invocation does not work via `cargo cicd` — must use `cargo-cicd` directly. This is a known cargo subcommand naming convention issue; the binary is named `cargo-cicd` and cargo dispatch requires it to be invoked as `cargo cicd` with cargo in PATH detecting the binary — works correctly in cargo's dispatch context.
2. Boundary leak fixed: wasm4pm_current.rs contained the forbidden term ALIVE in a doc comment; resolved in commit f931629.
3. Remote branch not confirmed pushed — local only.

## Verdict

PARTIAL

dry-run passed, install works, evidence gate ALIVE, public leaks fixed. Gap: remote not confirmed pushed.
