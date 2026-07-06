# Fleet Unification

## Context

The seanchatmangpt Rust fleet (praxis, wasm4pm, wasm4pm-compat, star-toml, chicago-tdd-tools, ggen, clap-noun-verb, cargo-cicd) has accumulated toolchain, version, and interface drift across repos that are meant to interoperate as a single system. cargo-cicd is meant to be the front door — the CLI a new developer runs without needing to understand cargo directly — but it currently fails to build on its own because of an unpinned git dependency on a nightly-only crate. This document maps the fleet as it stands, applies an ERRC (Eliminate/Reduce/Raise/Create) grid to the drift, and lays out the per-repo migration checklist needed to bring the fleet back into alignment.

## Fleet Map

| Repo | Version | Toolchain Pin | Task Runner | Role |
|------|---------|---------------|-------------|------|
| wasm4pm-compat | 26.6.29 | nightly-2026-06-22 | Justfile | Shared type-law layer (nightly-only, everyone's foundation) |
| wasm4pm | 26.7.1 | nightly-2026-04-15 | Justfile + Makefile.toml + Makefile | Engine; wpm oracle binary |
| praxis | 26.7.2 | nightly-2026-04-15 | justfile | House-style kit; already consumes cargo cicd CLI |
| star-toml | 26.7.3 | nightly-2026-04-15 | — | Config framework (dependency of cargo-cicd) |
| chicago-tdd-tools | 26.7.1 | nightly-2026-04-15 | justfile | Test framework |
| ggen | 26.7.4 | nightly-2026-06-22 | justfile | Codegen |
| clap-noun-verb | 26.7.4 | none | justfile | CLI framework; contains a stale embedded cargo-cicd fork at v26.6.2 |
| cargo-cicd | 26.6.30 | nightly-2026-06-22 (as of this fix) | justfile | The front door |

The dependency seam runs bottom-up. wasm4pm-compat sits at the base of the fleet: it defines structural types and type-law only, with no engine logic, so its toolchain pin is the tightest constraint in the fleet and everything downstream must be able to build against it. wasm4pm builds on top of wasm4pm-compat and ships the `wpm` binary — the oracle that adjudicates process evidence. cargo-cicd is the front-door CLI: it emits evidence (XES/JSONL) and calls out to `wpm` for adjudication, but it never adjudicates itself. praxis sits above all of this as the house style and law layer, and notably it consumes cargo-cicd as a CLI dependency — a binary it shells out to — not as a Cargo (`Cargo.toml`) dependency. That distinction matters: cargo-cicd's Rust API surface is not part of the contract praxis relies on, only its command-line behavior is.

## ERRC Grid

### Eliminate

- The unpinned `[patch.crates-io]` git dependency on wasm4pm-compat in cargo-cicd (now removed — 26.6.29 is published on crates.io)
- The stale embedded cargo-cicd fork inside `~/clap-noun-verb/crates/cargo-cicd` (v26.6.2) — one canonical repo should exist
- Redundant task runners in wasm4pm (hand-written Makefile + Makefile.toml + Justfile should collapse to one justfile)
- Stable-Rust CI in cargo-cicd (now fixed — the fleet is nightly by design, stable CI could never pass)

### Reduce

- Two competing nightly pins (2026-04-15 vs 2026-06-22) down to one: nightly-2026-06-22, because wasm4pm-compat's pin is the most constrained — everything downstream must be able to build it, not the other way around
- Version families (26.6.x vs 26.7.x) down to one CalVer train per release wave
- Path/git dependency edges down to crates.io version edges wherever the crate is already published

### Raise

- cargo-cicd to first-class fleet citizen: pinned toolchain, nightly CI, green build (done in this pass)
- The justfile convention to a verified contract: `cargo cicd workspace doctor` should be extended to check that a repo's justfile exposes the canonical verb set
- Onboarding: `just` plus `cargo cicd status` should be the complete surface a new developer ever needs to touch

### Create

- A fleet justfile contract (below)
- This document, as the fleet map and migration checklist of record

## The Justfile Contract

Every fleet repo should expose these recipes, with consistent semantics:

- `check` — lint and type-check
- `build` — build the crate/workspace
- `test` — run the full test suite
- `clippy` — run clippy with the fleet's standard lint set
- `fmt` — format the codebase
- `doctor` — diagnose workspace health (toolchain, dependencies, drift)
- `verify-all` — the full pre-merge gate (check + build + test + clippy + fmt --check)
- `evidence` — emit or audit process evidence where applicable

praxis and chicago-tdd-tools already model most of this contract; cargo-cicd's justfile was extended to match in this pass.

There is also a casing inconsistency worth resolving: wasm4pm and wasm4pm-compat use `Justfile` (capital J) while cargo-cicd and praxis use `justfile` (lowercase). Standardize on lowercase `justfile` — it's the more common convention in the ecosystem and it already matches the majority of the fleet.

## Migration Checklist (per repo, not yet executed)

- [ ] praxis: bump rust-toolchain.toml from nightly-2026-04-15 to nightly-2026-06-22 after running `just verify-all` on the new pin
- [ ] wasm4pm: same toolchain bump; additionally collapse Makefile + Makefile.toml into the existing Justfile and delete the redundant files; rename Justfile to justfile
- [ ] star-toml: same toolchain bump
- [ ] chicago-tdd-tools: same toolchain bump; rename Justfile to justfile if applicable
- [ ] clap-noun-verb: delete the embedded crates/cargo-cicd fork; if praxis or others need cargo-cicd, depend on the published crate or the canonical repo instead
- [ ] ggen: already on nightly-2026-06-22, no toolchain change needed; audit justfile against the canonical verb contract
- [ ] All repos: converge on the next shared 26.7.x release wave together once toolchain pins match, so version drift (26.6.x vs 26.7.x) doesn't recur
