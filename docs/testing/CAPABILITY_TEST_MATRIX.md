---
artifact: CAPABILITY_TEST_MATRIX
date: 2026-06-02
---

# Capability Test Matrix

Each row is one test scenario. Columns: **Capability** (command + verb), **Workspace**,
**Git**, **Toolchain**, **Target**, **Tests**, **CicdToml**, **Expected**.

Baseline values: single-crate, clean, pinned-nightly, small, unit-passing, absent.

---

## Primary Matrix (12 rows)

| Capability | Workspace | Git | Toolchain | Target | Tests | CicdToml | Expected |
|---|---|---|---|---|---|---|---|
| status | single | clean | pinned-nightly | small | unit | absent | exit 0, all-green report |
| status | single | dirty-tracked | pinned-nightly | small | unit | absent | exit 0, dirty warning present |
| status show | single | clean | pinned-nightly | small | unit | valid | exit 0, structured output |
| target show | workspace | clean | nightly | over-limit | unit | absent | exit 0, size warning |
| target prune | workspace | clean | nightly | over-limit | unit | absent | exit 0, suggest plan, no deletion |
| test changed | single | dirty-tracked | nightly | small | changed-src | absent | exit 0, conservative changed plan |
| trybuild changed | single | dirty-tracked | nightly | small | changed-fixture | absent | exit 0, changed-only plan |
| git status | single | dirty-tracked | any | any | any | absent | exit 0, dirty state reported |
| git close | single | clean | any | any | any | absent | exit 0, no-op pass |
| git close | single | dirty-unrelated | any | any | any | absent | exit non-0, refuse with named law |
| publish run | single | clean | pinned-nightly | small | unit | absent | exit 0, cicd.toml written |
| workspace doctor | single (missing manifest) | any | any | any | none | absent | exit non-0, explains missing Cargo.toml |

---

## Critical 3-Wise Cases (5 rows)

These are manually identified dangerous triangles that pairwise coverage would miss.

| Case | State 1 | State 2 | State 3 | Expected |
|---|---|---|---|---|
| dirty+trybuild+close | dirty-tracked git | changed trybuild fixture | `git close` invoked | exit non-0; refuse close, name dirty law |
| mismatch+changed+process-data | toolchain mismatch | changed source tests | `process-data` feature on | exit non-0; mismatch detected, event emitted |
| overlimit+release+prune | target over-limit | release artifacts present | `target prune` invoked | exit 0; release artifacts preserved, only incremental pruned |
| corrupted+publish+autonomic | corrupted cicd.toml | `publish run` invoked | `autonomic` feature on | exit non-0; refuse or offer repair, no silent overwrite |
| wasm4pm-missing+feature+publish | wasm4pm binary absent | `wasm4pm` feature enabled | `publish run` invoked | exit 0; PARTIAL reported, no fabricated capability |
