# Diagnostic Code Catalog

All diagnostic codes emitted by cargo-cicd-lsp follow the pattern `CICD-{FAMILY}-{NNN}`.

Each entry records:
- **Code** — the CICD-FAMILY-NNN identifier
- **Title** — snake_case name matching the `CicdCode` enum variant
- **Severity** — `Error`, `Warning`, `Information`, or `Hint` (LSP conventions)
- **Observed surface** — what the evaluator reads to detect the condition
- **Repair surface** — what must change to resolve the condition
- **Clears when** — the precise condition under which the diagnostic is removed

Families: GIT, EVIDENCE, WPM, TEST, TARGET, PUBLISH, PUBLIC, GGEN, CLOSE.

---

## GIT — Git Working Tree

Diagnostics in this family observe git index state, branch position, and working tree
cleanliness. They are raised when the local repository is not in a state that admits
further manufacturing transitions.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-GIT-001 | `dirty_tree_blocks_close` | Error | git index | run `cargo cicd git close` after staging all changes | working tree is clean |
| CICD-GIT-002 | `untracked_files_present` | Warning | git index | stage or `.gitignore` the untracked files, then run `cargo cicd git close` | no untracked files remain |
| CICD-GIT-003 | `branch_behind_remote` | Warning | git remote | run `git pull` to sync local branch with remote | local branch is up to date with remote |

---

## EVIDENCE — Process Evidence

Diagnostics in this family observe the process evidence log. They are raised when evidence
is missing, malformed, stale, or structurally invalid.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-EVIDENCE-001 | `evidence_dir_missing` | Error | filesystem | run any `cargo cicd` command to initialise the evidence directory | evidence directory exists |
| CICD-EVIDENCE-002 | `evidence_file_unreadable` | Error | evidence log file | correct file permissions or remove the corrupted file | evidence file is readable |
| CICD-EVIDENCE-003 | `evidence_entry_malformed` | Warning | evidence log entries | remove or repair malformed JSONL entries | all evidence entries parse without error |
| CICD-EVIDENCE-004 | `evidence_stale` | Warning | evidence log timestamps | run a manufacturing pass to emit fresh evidence | most recent evidence entry is within the staleness threshold |

---

## WPM — wasm4pm / wpm Capability

Diagnostics in this family observe the availability and health of the `wpm` oracle binary
and associated wasm4pm capability. They are raised when wpm is absent or cannot be
interrogated.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-WPM-001 | `wpm_binary_not_found` | Warning | `$PATH` | install `wpm`; some LSP diagnostics are unavailable without it | `wpm` is found on `$PATH` |
| CICD-WPM-002 | `wpm_version_incompatible` | Warning | `wpm --version` output | upgrade or downgrade `wpm` to a compatible version | `wpm` version satisfies the required range |
| CICD-WPM-003 | `wpm_oracle_unreachable` | Error | `wpm` process invocation | verify `wpm` is correctly installed and not blocked | `wpm` responds without error |

---

## TEST — Test Coverage

Diagnostics in this family observe test passage and changed-file coverage. They are raised
when tests fail or when changed files have no coverage mapping.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-TEST-001 | `test_failures_block_close` | Error | `cargo test` output | run `cargo cicd test run` and fix all failing tests | all tests pass |
| CICD-TEST-002 | `changed_file_has_no_coverage` | Information | changed file list vs coverage map | add a test that exercises the changed file, or update coverage config | changed file has a coverage mapping |
| CICD-TEST-003 | `coverage_threshold_not_met` | Warning | coverage report | add tests until the threshold is met | coverage meets or exceeds the configured threshold |

---

## TARGET — Build Targets

Diagnostics in this family observe workspace build targets, artifacts, and `Cargo.toml`
structure. They are raised when targets are missing, ambiguous, or their artifacts are
absent.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-TARGET-001 | `no_targets_found` | Warning | workspace `Cargo.toml` members | add at least one `[lib]` or `[[bin]]` to a member manifest | at least one target is present |
| CICD-TARGET-002 | `target_artifact_missing` | Warning | `target/` directory | run `cargo build` to produce the artifact | artifact path exists |
| CICD-TARGET-003 | `duplicate_target_name` | Warning | workspace member manifests | rename one of the duplicate targets | no two targets share a name |
| CICD-TARGET-004 | `target_dir_growth_excessive` | Information | `target/` directory size | run `cargo clean` if growth is from stale artifacts | target directory is within the configured size limit |

---

## PUBLISH — Publish Readiness

Diagnostics in this family observe publish readiness indicators recorded in `cicd.toml`
and the evidence log. They are raised when the workspace is not in a state that admits
a publish transition.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-PUBLISH-001 | `no_cicd_toml_found` | Error | workspace root | run `cargo cicd publish run` to generate `cicd.toml` | `cicd.toml` exists at workspace root |
| CICD-PUBLISH-002 | `cicd_toml_parse_failure` | Error | `cicd.toml` TOML syntax | fix the TOML syntax error identified in the diagnostic message | `cicd.toml` parses without error |
| CICD-PUBLISH-003 | `version_not_bumped` | Warning | `Cargo.toml` version vs last published version | bump the crate version before publishing | version in `Cargo.toml` is greater than the last published version |
| CICD-PUBLISH-004 | `changelog_entry_missing` | Information | changelog file | add a changelog entry for the current version | a changelog entry exists for the current version |

---

## PUBLIC — Public Boundary Safety

Diagnostics in this family observe the public API surface. They are raised when public
items are missing documentation, expose internal types, or violate boundary safety rules.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-PUBLIC-001 | `public_item_missing_doc` | Warning | rendered rustdoc surface | add rustdoc to the undocumented public item | all public items have rustdoc |
| CICD-PUBLIC-002 | `public_boundary_leaks_internal` | Error | public type signatures | remove or re-export the internal type through the public boundary | no internal types appear in public signatures |
| CICD-PUBLIC-003 | `rendered_surface_drift` | Warning | rendered rustdoc vs prior snapshot | re-render docs and update the snapshot, or revert the unintended change | rendered surface matches the current snapshot |

---

## GGEN — Generated Code

Diagnostics in this family observe generated code surfaces (ggen output). They are raised
when generated files are stale relative to their source templates or schema inputs.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-GGEN-001 | `generated_file_stale` | Warning | generated file vs source template | re-run `cargo cicd ggen` to regenerate the file | generated file matches current template output |
| CICD-GGEN-002 | `generated_file_manually_edited` | Error | generated file vs template hash | revert manual edits; edit the source template instead | generated file matches the template without manual edits |
| CICD-GGEN-003 | `ggen_template_missing` | Error | template path declared in `cicd.toml` | restore the missing template or remove the declaration | template file exists at the declared path |

---

## CLOSE — Phase Closure

Diagnostics in this family observe manufacturing phase closure conditions. They are raised
when a required phase transition has not been completed or when phase evidence is absent.

| Code | Title | Severity | Observed surface | Repair surface | Clears when |
|------|-------|----------|-----------------|----------------|-------------|
| CICD-CLOSE-001 | `pipeline_stage_failed` | Error | pipeline run evidence | run `cargo cicd pipeline run` and address reported stage failures | all pipeline stages pass |
| CICD-CLOSE-002 | `phase_evidence_missing` | Error | evidence log phase events | complete the required phase and emit phase evidence | required phase event is present in the evidence log |
| CICD-CLOSE-003 | `close_blocked_by_error_diagnostic` | Error | current diagnostic set | resolve all Error-severity diagnostics in other families | no Error-severity diagnostics remain |
| CICD-CLOSE-004 | `workspace_structure_invalid` | Error | workspace `Cargo.toml` | run `cargo cicd workspace doctor` to diagnose structural issues | workspace doctor reports no violations |

---

## Notes on Severity

- **Error** — the condition blocks a push-ready or publish-ready state; CI would fail
- **Warning** — the condition degrades readiness and should be resolved before release
- **Information** — informational only; CI is unlikely to fail but the condition is worth knowing
- **Hint** — best-practice suggestion; no direct CI impact expected
