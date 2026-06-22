# /test — Run cargo-cicd Test Suites

Trigger: user says "run tests", "test", or runs `/test`.
Run `invariants` first — failure there invalidates all other results.

## Tier 1 — Unit/Smoke (no external deps)

| Suite | File | Gate |
|-------|------|------|
| `invariants` | `tests/invariants.rs` | **Run first. Fail = stop.** |
| `cli` | `tests/cli/` | noun/verb dispatch + output |
| `feature_projection` | `tests/feature_projection.rs` | feature flag contracts |
| `autonomic_policies` | `tests/autonomic_policies.rs` | policy eval logic |
| `git_phase_closure` | `tests/git_phase_closure.rs` | git state detection |
| `changed_tests` | `tests/changed_tests.rs` | changed-file classification |
| `cicd_toml_truth` | `tests/cicd_toml_truth.rs` | toml round-trip |

## Tier 2 — Evidence Gate (requires `wpm` on PATH; release gate)

| Suite | Passing verdict |
|-------|-----------------|
| `wasm4pm_evidence_gate` | `Accept` |
| `wasm4pm_evidence_mutation` | `Refuse` (corrupt evidence rejected) |
| `wasm4pm_refusal_cases` | `Refuse` or `Blocked` |

No release without Tier 2 passing with live oracle.

## Execution order

```bash
# Step 1 — invariants gate
cargo test --test invariants

# Step 2 — Tier 1
cargo test --test cli
cargo test --test feature_projection
cargo test --test autonomic_policies
cargo test --test changed_tests
cargo test --test git_phase_closure
cargo test --test cicd_toml_truth

# Step 3 — Tier 2 (requires wpm)
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

Full run via cargo-make:
```bash
cargo make test
```

## Feature flags

```bash
cargo test --features process-data    # Level 5 engine, adapters, cicd.toml
cargo test --features autonomic       # policy suggestions (implies process-data)
cargo test --features wasm4pm         # oracle integration (implies process-data)
cargo test --features autonomic,wasm4pm
```

## Offline / no wpm

Tier 2 tests must declare `ExpectedWpmVerdict::Blocked` — first-class expectation, not an error.
`Blocked` passes in CI without wpm but cannot close the release gate.

To unblock: `which wpm && wpm --version` — add wpm binary dir to `PATH`, then re-run Tier 2.

## Invariants enforced (7 rules in `tests/invariants.rs`)

1. No forbidden terms in `--help` output: `ALIVE`, `Inspection Gate`, `wall`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`
2. No destructive action without `--confirm`
3. No full trybuild run by default
4. Noun names are lowercase ASCII
5. Binary name is `cargo-cicd`
6. `cargo cicd status show` exits 0
7. `git close` prints safety warning before phase transition

## Verbose / failure triage

```bash
cargo test -- --nocapture 2>&1 | tee test-output.log
cargo test 2>&1 | grep -E "^(test .* FAILED|FAILED|error)"
```
