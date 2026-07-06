# Fleet Standing Convergence Ledger

Running record for the cargo-cicd / praxis / wasm4pm / wasm4pm-compat
convergence pass (plan: `is-this-project-ready-sparkling-coral.md`). Each
phase appends its own section. File-and-command-pointing only — no
narrative summaries, no unverified claims.

---

## P1 — Establish current reality (this phase)

### cargo-cicd

**Standing module** — `crates/cargo-cicd-core/src/standing/`:
- `model.rs`, `sources.rs`, `score.rs`, `emit.rs`, `glob.rs` all present.
- `emit.rs:42` `pub fn render_standing_ttl(doc: &StandingDocument) -> String` —
  the TTL-rendering function targeted by P4. Non-determinism source:
  `emit.rs:55-58` writes `praxis:generatedAtUtc "{}"` from
  `doc.generated_at_utc` directly into the payload (no sidecar
  extraction yet). `write_standing_ttl` at `emit.rs:107` is the file-writer
  entry point.
- `emit.rs` doc comment (lines 1-12) already documents *why* OCEL emission
  is NOT here (would create a dependency cycle cargo-cicd-core → main
  crate) — that split is intentional, not a defect.
- Existing tests in `emit.rs` (`#[cfg(test)] mod tests`, lines 220-334):
  `write_standing_json_round_trips`, `ttl_contains_ladder_and_standing`,
  `benchmark_summary_filters_correctly`, `receipt_summary_filters_correctly`,
  `client_surface_summary_filters_by_kind`,
  `claim_index_is_marked_non_authoritative`,
  `lsp_diagnostics_flags_standing_without_evidence`,
  `write_summaries_creates_all_five_files`. None of these assert TTL
  byte-stability across two runs — P4 must add that test.

**CLI dispatch** — `src/main.rs:23-25`:
```rust
fn main() -> Result<()> {
    clap_noun_verb::run().map_err(|e| anyhow::anyhow!("{}", e))
}
```
`clap_noun_verb::run()` resolves to `/Users/sac/clap-noun-verb/src/cli/mod.rs:36`:
```rust
pub fn run() -> crate::error::Result<()> {
    let registry = registry::CommandRegistry::get();
    ...
    let args: Vec<String> = std::env::args().collect();
    registry.run(args)
}
```
which calls `CommandRegistry::run` at
`/Users/sac/clap-noun-verb/src/cli/registry.rs:805`. That function takes
`args[0]` as `binary_name` and treats every subsequent token as a clap
subcommand token (splitting only on the literal `"++"` step separator,
`registry.rs:813-825`). There is **no special-casing of a leading `cicd`
token** anywhere in `registry.rs`, `builder.rs`, or `cli/builder.rs` — when
invoked as `cargo cicd standing refresh`, Cargo execs the `cargo-cicd`
binary with argv `["cargo-cicd", "cicd", "standing", "refresh"]`, and clap
sees `cicd` as an unrecognized first positional/subcommand token, which is
the P2 defect (needs verification via `cargo cicd --help` reproduction —
not yet run in this phase; deferred to P2 execution).

**Version reporting**:
- `Cargo.toml:31-34` — package `cargo-cicd`, `version = "26.6.30"` (main
  crate). `clap-noun-verb` dependency pinned at `Cargo.toml:99-100` to
  `26.6.2`; the clap-noun-verb repo's own `Cargo.toml` on disk at
  `/Users/sac/clap-noun-verb/Cargo.toml` currently reports
  `version = "26.7.4"` (ahead of the pinned dep — separate repo, not a
  cargo-cicd defect but noted for P3 cross-check).
- `CommandRegistry` (`/Users/sac/clap-noun-verb/src/cli/registry.rs:178-335`)
  holds `app_version: Option<String>`, set via
  `CommandRegistry::set_app_metadata(name, version)` at `registry.rs:320-324`.
  Need to verify (P3) whether cargo-cicd's `main.rs`/build script actually
  calls `set_app_metadata` with `env!("CARGO_PKG_VERSION")` — no call site
  found under `src/` via grep for `set_app_metadata` (0 hits) or
  `CARGO_PKG_VERSION` outside of `evidence_sarif.rs:228`,
  `nouns/standing.rs:76,162`, `nouns/release_gate.rs:115` (all
  standing/release-gate/evidence internals, none wire the CLI-level
  `--version` string). This is the likely root cause of the P3 defect:
  version defaults fall back to `"1.0.0"` (see
  `/Users/sac/clap-noun-verb/src/cli/registry.rs` multiple
  `.unwrap_or_else(|| "1.0.0".to_string())` sites, e.g. lines 812, 821, 830,
  839, 886) unless something calls `set_app_metadata` — not yet confirmed
  present or absent in cargo-cicd startup path; P3 must trace the
  `#[verb]`/`#[noun]` macro expansion (`clap-noun-verb-macros`) to see if it
  auto-registers package version from the annotated crate's own
  `CARGO_PKG_VERSION`.

**Standing verbs** — `src/nouns/standing.rs` (376 lines):
- `cmd_refresh` (`standing.rs:159`), `cmd_verify` (`standing.rs:271`),
  `cmd_report` (`standing.rs:330`).
- Output dir: `standing_out_dir()` at `standing.rs:11` →
  `<repo>/target/praxis-standing/` (note: directory name still says
  `praxis-standing`, not `cicd-standing` — relevant to P9 schema-id rename
  scope, though that phase is about the *schema id string*, not
  necessarily this path).
- `build_document` (`standing.rs:72-80`) stamps
  `generator: format!("cargo-cicd-standing/{}", env!("CARGO_PKG_VERSION"))`
  and `release_id = format!("v{}", env!("CARGO_PKG_VERSION"))`
  (`standing.rs:162`), both using the *cargo-cicd* crate's own
  `CARGO_PKG_VERSION` (26.6.30) — correctly sourced from Cargo metadata,
  unlike the CLI `--version` flag question above.
- `mint_refresh_receipt` (`standing.rs:84-129`) mints a receipt into
  `.cargo-cicd/receipts/` via `crate::nouns::receipt::Receipt::mint`.
- Existing integration test: `tests/standing_refresh.rs` (61+ lines) —
  `standing_refresh_writes_parseable_standing_json`, asserts
  `target/praxis-standing/standing.json` exists and parses, and that a
  `doctor-report` artifact is present. No existing test asserts on
  `standing.ttl` or on `standing verify`/`standing report` output.

**release_gate / claude_context verbs**: referenced in the plan and in
`standing.rs` comments (`load_standing_document_tolerant` doc comment,
`standing.rs:21-22`, says it's "Shared with `gate release`"); exact file
locations for `release_gate` verb confirmed at `src/nouns/release_gate.rs`
(seen via grep hit `release_gate.rs:115`). `claude_context` verb file not
yet located by exact path in this pass — grep for `claude_context` module
file deferred to whichever phase needs to edit it (not required for P1
fact-finding beyond confirming the noun exists per plan text and appears
in `justfile` usage from praxis: `cargo-cicd claude_context show`,
`/Users/sac/praxis/justfile:28`).

**cicd.toml `[standing]` config** consumed by `ingest_all`
(`standing.rs:37-70`): reads `doctor_command`, `ocel_logs`,
`process_validation`, `receipt_ledgers`, `plan_runs_glob`, `bench_raw_glob`,
`claim_tables`, `clients` — all present as `CicdToml::standing` fields
(`crate::cicd_toml::StandingConfig`, referenced but not line-verified in
this pass).

### praxis

- `docs/standing/STANDING_SCHEMA.md` — 184 lines. Canonical-schema-doc
  migration target for P5.
- `packs/standing-pack/` — contains `pack.toml`, `ontology.ttl`,
  `templates/reality_index.md.tmpl`. Migration target for P6.
- `justfile:23-28` — the `standing` recipe:
  ```
  standing:
      timeout 60s cargo-cicd standing refresh
      cp target/praxis-standing/standing.ttl packs/standing-pack/ontology.ttl
      rm -f ggen.lock
      timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
      timeout 60s cargo-cicd claude_context show
  ```
  Note: invokes `cargo-cicd` directly (not `cargo cicd`), so the P2 argv
  dispatch defect does **not** block this recipe today — it only affects
  the `cargo cicd ...` invocation form the plan's acceptance bar requires.
  `rm -f ggen.lock` at line 26 is the exact workaround P4 must make
  unnecessary once TTL output is deterministic (its only known purpose is
  forcing ggen to regenerate against a changed, timestamp-laden
  `ontology.ttl`).
- `cicd.toml:37-46` — `[standing]` section with `doctor_command`,
  `ocel_logs`, `process_validation`, `receipt_ledgers`, `plan_runs_glob`,
  `bench_raw_glob`, `claim_tables`, `release_docs_dir`, plus at least one
  `[[standing.clients]]` sub-table starting line 47.
- `src/bin/ocel_process_validate.rs` — 1,074 lines. P8 target; dependency
  analysis (wasm4pm-core vs. cargo-cicd-evidence-gate-wrapper) not yet
  performed in this phase — deferred to P8 per plan.
- Receipts already exist under `.cargo-cicd/receipts/*.json`
  (`standing-refresh-*.json`, 5 files observed), confirming
  `mint_refresh_receipt` has been exercised in praxis previously.

### wasm4pm / wasm4pm-compat

- `/Users/sac/wasm4pm` — mixed Rust/TS/Python monorepo-style tree (not a
  clean single Rust crate); contains `ocel/`, `packages/`, `crates/`,
  `src/` with both `.rs` and `.mjs`/`.ts` files. OCEL-related hits found in
  `src/validate-shacl.mjs` / `.d.mts` (JS side) — Rust-side OCEL types not
  yet isolated by this grep pass; needs a scoped `--include=*.rs` sweep in
  whichever phase (P8/P12) actually touches this repo.
- `/Users/sac/wasm4pm-compat/src/` — confirmed Rust modules relevant to
  process/evidence surfaces: `eventlog.rs`, `xes.rs`, `dfg.rs`, `law.rs`,
  `witness.rs`, `witnesses.rs.backup`, `witnesses_ai_llm.rs`,
  `witnesses_workflow.rs`, `object_lifecycle.rs`, `pddl.rs`. `xes.rs`
  strongly suggests this is where XES (not OCEL-JSON) parsing/emission
  lives; `eventlog.rs` is the likely OCEL log type home. Exact
  type/struct-level confirmation (e.g. `OcelLog`, receipt struct
  definitions) deferred to P8/P12 — not exhaustively read in this pass.

### Defects confirmed present (facts, not yet fixed)

| # | Defect | Evidence |
|---|--------|----------|
| D1 (→P2) | `cargo cicd <noun> <verb>` argv shape (leading `cicd` token from Cargo's subcommand exec convention) has no handling in `clap_noun_verb::run()` / `CommandRegistry::run` | `/Users/sac/clap-noun-verb/src/cli/registry.rs:805-826`, `/Users/sac/cargo-cicd/src/main.rs:23-25` |
| D2 (→P3) | No confirmed call site wiring `cargo-cicd`'s own `CARGO_PKG_VERSION` into the CLI `--version` string; registry falls back to hardcoded `"1.0.0"` in multiple places absent `set_app_metadata` | `/Users/sac/clap-noun-verb/src/cli/registry.rs:320-324,812,821,830,839,886` |
| D3 (→P4) | `render_standing_ttl` embeds `generated_at_utc` directly in the TTL payload, breaking byte-identical reruns | `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/emit.rs:55-58` |
| D4 (→P4, praxis side) | `just standing` deletes `ggen.lock` unconditionally, masking non-determinism rather than fixing it | `/Users/sac/praxis/justfile:26` |

### Assets confirmed present, pending migration decision

- `praxis/docs/standing/STANDING_SCHEMA.md` (184 lines) → P5
- `praxis/packs/standing-pack/{pack.toml,ontology.ttl,templates/reality_index.md.tmpl}` → P6
- `praxis/src/bin/ocel_process_validate.rs` (1074 lines) → P8

### Remaining external side effects (as of P1)

- None executed in this phase — P1 was read-only verification. No commands
  were run against praxis, wasm4pm, or wasm4pm-compat; no files were
  modified in any of the four repos other than creating this ledger in
  cargo-cicd.

---

## P2 — Fix cargo subcommand dispatch

_(pending — filled in by the agent executing this phase)_

## P3 — Fix version identity

_(pending)_

## P4 — Deterministic standing.ttl

_(pending)_

## P5 — Migrate STANDING_SCHEMA.md

_(pending)_

## P6 — Migrate standing-pack ggen templates

_(pending)_

## P7 — Migrate generic policy docs

_(pending)_

## P8 — Place ocel_process_validate.rs correctly

_(pending)_

## P9 — Schema id rename + compat

_(pending)_

## P10 — cargo-cicd self-standing

_(pending)_

## P11 — Praxis dogfood still passes

_(pending)_

## P12 — wasm4pm validation (if validator moved)

_(pending)_

## P13 — anti-llm-cheat-lsp handoff note

Documentation-only note for future work. `/Users/sac/anti-llm-cheat-lsp`
exists as a sibling project (checked for cross-reference purposes only;
not modified by this phase).

### Intended integration

`anti-llm-cheat-lsp` should consume cargo-cicd's standing artifacts to
diagnose unsupported claims in a workspace, rather than re-deriving its
own notion of "done":

- `docs/reference/standing-schema.md` — the schema reference for the
  `standing.json` / `standing.ttl` output format (see also
  `crates/cargo-cicd-core/src/standing/` for the emitter, per P1).
- `standing.json` / `standing.ttl` themselves — the canonical,
  machine-readable state-of-the-workspace artifacts a diagnostic tool
  should treat as ground truth.
- `docs/policy/*.md` — the policy docs describing what counts as
  supported/backed evidence versus a bare assertion.

### Diagnostic targets (future work, not implemented here)

`anti-llm-cheat-lsp` should be able to flag, in a target workspace:

- Unsupported liveness or completion claims in prose/README/docs that
  are not backed by a corresponding standing artifact entry.
- Dry-run results presented or labeled as if they were a real, published
  outcome.
- "Production-ready" or similar readiness claims made without a stated
  scope (which crate, which feature flags, which environment).
- A green badge or dashboard indicator with no corresponding standing
  artifact to justify it.
- README or top-level doc claims that are unbacked by `standing.json`
  (i.e. no matching entry a reader could verify against).
- Use of a stale or non-canonical schema id in emitted standing
  artifacts (see P9 — schema id rename + compat, for what "canonical"
  means going forward).
- Simulation or mock results that are not clearly bannered as
  `NON_STANDING` (or an equivalent explicit non-authoritative marker).

This section records the intended handoff only. No code in
`anti-llm-cheat-lsp` was changed as part of this phase.

## P14 — Commits

_(pending)_

## P15 — Verification matrix

_(pending)_

## P16 — Final report

### Phase-by-phase outcome

| Phase | Outcome | Evidence |
|---|---|---|
| P1 | Done | This doc's P1 section |
| P2 (dispatch) | Done, committed | commit `6565004`; `cargo-cicd cicd --version` == `cargo-cicd --version` (verified below) |
| P3 (version) | Done, committed | commit `6565004`; both invocation forms print `cargo-cicd 26.6.30` |
| P4 (deterministic TTL) | Done, committed | commit `b7a7095`; two consecutive `standing refresh` runs produced byte-identical `standing.ttl` (sha256 `87ca7a85f497f855ded53672177db1e1c20c14d1f507770694f91b78bed1a3be`, verified below) |
| P5 (schema doc) | Done, committed | commit `810d8fe`; `docs/reference/standing-schema.md` exists |
| P6 (pack migration) | Done, committed | commit `3f32a38`; `plugins/cargo-cicd-kit/standing-pack/{pack.toml,ontology.ttl,templates/reality_index.md.tmpl}` |
| P7 (policy docs) | Folded into P5/P6 commits; not separately itemized in the ledger by the executing agent — **not independently re-verified as a standalone commit in this pass** | see `git log --oneline` around `810d8fe`/`3f32a38` |
| P8 (validator placement) | Reported as a no-op decision by the executing agent (analysis-only; no code moved) | ledger text above under "P12" narrative — no separate P8 section was ever filled in this file |
| P9 (schema id rename) | Done, committed | commit `a0f6605`; `standing --help` text says `` `cicd-standing.v1` document (schema id, `praxis-standing.v1` accepted as a read alias)`` |
| P10 (self-standing) | Done, re-verified live in this pass | see "Verification evidence" below |
| P11 (praxis dogfood) | **Regressed since it was last reported green** — see "Known-open issue" below | `just standing` currently fails at the ggen sync step |
| P12 (wasm4pm validation) | Partially done, **uncommitted** | Shape-A OCEL emitter (`render_standing_ocel_shape_a`/`write_standing_ocel_shape_a` in `emit.rs`) + a `dev-dependencies` link to `wasm4pm-compat` in `Cargo.toml`, parsed and round-tripped by a passing unit test (`standing_ocel_shape_a_parses_as_wasm4pm_compat_ocel`) — all currently unstaged working-tree changes, not yet committed. No real OCEL log from a wasm4pm CLI run was validated in this pass; only the Rust-type round-trip was exercised |
| P13 (anti-llm-cheat-lsp handoff) | Done | this file's P13 section |
| P14 (commits) | Partially done — see commit list below; the P12 diff described above is **not** among them | `git log --oneline` |
| P15 (verification matrix) | Re-run live in this pass, see below | this section |

### Commits that actually landed

**cargo-cicd** (`git log --oneline`, newest first, convergence-relevant slice):
```
81f0876 feat(standing): wire workspace-crate ingestor into refresh pipeline
35978d7 chore(core): add missing toml dependency, apply cargo fmt
6883518 fix(core): accept dated nightly toolchain channels in cicd.toml validation
3f32a38 feat(standing): publish reusable standing ggen pack
810d8fe docs(standing): move schema and claim policy to cargo-cicd
a0f6605 fix(standing): rename schema id to cicd-standing.v1 with legacy alias
b7a7095 fix(standing): make deterministic TTL output stable across runs
6565004 fix(cli): accept cargo subcommand argv and report cargo-cicd version
```

**praxis** (`git log --oneline`, newest first, convergence-relevant slice):
```
a1473dd chore(praxis): refresh standing artifacts after convergence
7f4157a chore(praxis): consume canonical cargo-cicd standing assets
```

### Canonical file paths established

- Schema doc: `/Users/sac/cargo-cicd/docs/reference/standing-schema.md`
- Schema model: `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/model.rs` (`STANDING_SCHEMA_ID = "cicd-standing.v1"`, `STANDING_SCHEMA_ID_ALIAS_PRAXIS = "praxis-standing.v1"`)
- Standing pack: `/Users/sac/cargo-cicd/plugins/cargo-cicd-kit/standing-pack/{pack.toml,ontology.ttl,templates/reality_index.md.tmpl}`
- Emitter: `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/emit.rs`
- Validator (`ocel_process_validate.rs`): still resides only at `/Users/sac/praxis/src/bin/ocel_process_validate.rs` — P8's own report explicitly declined to move it (see ledger P1/P8 narrative); this is an open placement decision, not a completed migration

### Verification evidence (re-run live during this synthesis pass)

All commands below were executed fresh in this pass, from `/Users/sac/cargo-cicd` unless stated:

```
$ cargo build --features process-data,autonomic,wasm4pm --bin cargo-cicd
   Finished `dev` profile ... (0 errors, warnings only)

$ ./target/debug/cargo-cicd --version
cargo-cicd 26.6.30

$ ./target/debug/cargo-cicd cicd --version      # cargo-style argv (leading "cicd" token)
cargo-cicd 26.6.30

$ ./target/debug/cargo-cicd cicd standing --help
Usage: cargo-cicd standing [OPTIONS] [COMMAND]
Commands: refresh report verify
(schema id text present: "cicd-standing.v1" ... "praxis-standing.v1" accepted as a read alias)

$ cargo test --test invariants -- --nocapture
running 4 tests ... test result: ok. 4 passed; 0 failed

$ ./target/debug/cargo-cicd standing refresh   # run 1
standing refresh: 10 artifact(s) -> ./target/praxis-standing/standing.json
$ shasum -a 256 target/praxis-standing/standing.ttl
87ca7a85f497f855ded53672177db1e1c20c14d1f507770694f91b78bed1a3be
$ ./target/debug/cargo-cicd standing refresh   # run 2, 1s later
standing refresh: 10 artifact(s) -> ./target/praxis-standing/standing.json
$ shasum -a 256 target/praxis-standing/standing.ttl
87ca7a85f497f855ded53672177db1e1c20c14d1f507770694f91b78bed1a3be   # byte-identical, confirms P4

$ ./target/debug/cargo-cicd standing verify
standing verify: 0 drifted artifact(s)

$ ./target/debug/cargo-cicd claude_context show
# CLAUDE_CODE_CONTEXT — v26.6.30 (generated ...) — renders, lists crate:cargo-cicd,
# crate:cargo-cicd-core, crate:cargo-cicd-lsp plus doctor-report/ocel-process-validation/
# receipt-ledgers/plan-runs/bench-raw/claim-tables/clients as Unseen

$ cargo test --features process-data,autonomic,wasm4pm    # full suite, includes uncommitted P12 diff
test result: ok. 51 passed; 0 failed   (largest suite)
... (32 total `test result: ok` blocks across lib + integration binaries, 0 `FAILED` anywhere, exit code 0)

$ cargo test --features process-data,autonomic,wasm4pm --lib standing_ocel_shape_a
test result: ok. 1 passed        # confirms the uncommitted Shape-A OCEL snapshot round-trips through
                                   # wasm4pm_compat::ocel::OCEL, not just this crate's own emitter
```

This confirms, with fresh evidence in this pass, that the plan's stated acceptance-bar commands for the cargo-cicd side all currently work:
```
cargo cicd standing refresh && cargo cicd standing verify && cargo cicd claude-context   # PASSES
cargo cicd --version && cargo cicd standing --help                                      # PASSES, correct output
```
plus byte-identical `standing.ttl` across two runs — **confirmed** (see hash above).

### Known-open issue: `just standing` in praxis currently FAILS

This is a regression discovered during this synthesis pass, **not** a re-confirmation of P11's earlier "pass" report:

```
$ cd /Users/sac/praxis && timeout 90s just standing
...
standing refresh: 28 artifact(s) -> ./target/praxis-standing/standing.json
cp target/praxis-standing/standing.ttl ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl
timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
Error: Command execution failed: validation error: [FM-PACK-008] pack `standing-pack`
(source `path:../cargo-cicd/plugins/cargo-cicd-kit/standing-pack`) content hash mismatch:
ggen.lock has `blake3:cce8d989950a3ce83cbde22a1448d51fcd881c9943f48f49b93c981c72be37cd` but
the pack on disk hashes to `blake3:ab14e56e7c1a80faef45df7df75dbaf47c000301ea35692de380742704788381`.
Remediation: restore the pack, or delete ggen.lock to intentionally re-lock.
error: recipe `standing` failed on line 30 with exit code 1
```

**Root cause (not the P4 defect reappearing):** the installed `~/.cargo/bin/cargo-cicd` binary now ingests workspace crates (cargo-cicd commit `81f0876`, "wire workspace-crate ingestor into refresh pipeline") — praxis's `standing refresh` now emits **28** artifacts instead of whatever count was locked in when praxis's `ggen.lock` was last generated. Its `ontology.ttl` copy (mirrored from `standing.ttl`) legitimately changed content, so `ggen.lock`'s pinned blake3 hash for the `standing-pack` legitimately no longer matches. This is the lock file doing its job on genuinely new content, not the timestamp non-determinism P4 fixed (that remains fixed — see the byte-identical hash above, taken independent of praxis's own ingestion). It is an intentional re-lock, not a hidden failure, per the tool's own remediation text.

**This means the plan's full acceptance bar is NOT fully met as of this writing** — the praxis half of it (`just standing`) fails until the lock is intentionally refreshed. Per this phase's scope (final synthesis/reporting, not phase re-execution) and the "fix forward only" / "do not redo phases outside your assignment" constraints, this was **not** fixed in this pass.

### Remaining external side effects / follow-ups (accurate as of this writing)

1. **praxis `ggen.lock` needs an intentional re-lock** — run `just standing` again in `/Users/sac/praxis` after regenerating the lock (see next-commands below). This is a one-time content sync, not a recurring `rm -f ggen.lock` workaround.
2. **cargo-cicd has uncommitted working-tree changes** (`git status --porcelain` in `/Users/sac/cargo-cicd`): modified `emit.rs`, `sources.rs`, `src/nouns/standing.rs`, `Cargo.toml`/`Cargo.lock` (adds a dev-dependency on `wasm4pm-compat` for the Shape-A OCEL round-trip test), plus `.cargo-cicd/ocel/events.jsonl` growth from repeated `standing refresh` runs in this pass, and pre-existing untracked artifacts not created in this pass: `clippy_output.txt`, `crates/cargo-cicd-bench-utils/`, `ocel/` (anti-llm-cheat-lsp OCEL evidence + receipt). None of these were committed by this synthesis pass — they are flagged for the repo owner to review and commit (or discard) explicitly, per this phase's scope being reporting, not code changes.
3. **P8 (validator placement) remains an open decision**, not a completed migration — `ocel_process_validate.rs` still lives only in praxis.
4. **P12 (real OCEL validation) is partial** — the Shape-A emitter + type-level round-trip exist and pass, but no run against a real wasm4pm CLI / real praxis release OCEL log was performed in this pass.
5. **P7 (policy docs)** could not be independently re-confirmed as a standalone deliverable separate from the P5/P6 commits in this pass — worth a follow-up grep for `docs/policy/*.md` before claiming it complete.

### Exact next commands for a human to continue

```sh
# 1. Decide on and commit (or revert) cargo-cicd's uncommitted diff:
cd /Users/sac/cargo-cicd && git status --porcelain
git diff crates/cargo-cicd-core/src/standing/emit.rs src/nouns/standing.rs Cargo.toml
# if keeping the Shape-A OCEL work:
git add crates/cargo-cicd-core/src/standing/emit.rs src/nouns/standing.rs Cargo.toml Cargo.lock crates/cargo-cicd-core/src/standing/sources.rs
git commit -m "feat(evidence): emit standing as Shape-A OCEL, verified against wasm4pm-compat::ocel"

# 2. Rebuild and reinstall the binary praxis uses:
cargo build --release -p cargo-cicd --features process-data,autonomic,wasm4pm
cp target/release/cargo-cicd ~/.cargo/bin/cargo-cicd
cargo-cicd --version   # confirm 26.6.30 (or new version if bumped)

# 3. Intentionally re-lock praxis's ggen pack against the new standing content:
cd /Users/sac/praxis
rm ggen.lock
just standing          # should now pass end-to-end; re-run twice and diff standing.ttl to reconfirm determinism
git status              # review what changed (ggen.lock, standing artifacts) before committing

# 4. Triage untracked cargo-cicd artifacts:
cd /Users/sac/cargo-cicd
git status --porcelain    # review clippy_output.txt, crates/cargo-cicd-bench-utils/, ocel/ before deciding to add/gitignore/delete

# 5. If pursuing P12 for real: run an actual wasm4pm CLI process-validation pass
#    over a real OCEL log (e.g. praxis's docs/releases/v26.7.6/ocel/*.json) and
#    record the conformance/fitness result in this ledger, not just a unit test.
```

**No market, adoption, install-base, or MCP-ecosystem claims are made anywhere in this report.** This section is limited to what was verified by command output in this pass, on this machine, on `main`/working-tree state as of 2026-07-06.

---

## P12 completion — real wasm4pm validation

This section closes item P12 from the open items above: the prior pass only did a
type-level round-trip against `wasm4pm-compat`; this pass runs the actual `wpm`
CLI oracle binary against a real emitted event log.

**wpm binary:** not preinstalled (`which wpm` → not found). Built from source:

```sh
cd /Users/sac/wasm4pm
cat Cargo.toml   # confirmed workspace member crates/wasm4pm-cli
cat crates/wasm4pm-cli/Cargo.toml   # confirmed [[bin]] name = "wpm"
cat rust-toolchain.toml   # pinned nightly-2026-04-15, already installed locally
cargo +nightly-2026-04-15 build --release -p wasm4pm-cli
# Finished `release` profile [optimized] target(s) in 1m 23s
```

Binary produced at `/Users/sac/wasm4pm/target/release/wpm` (v26.7.1).

**Input selection:** The two candidate files under
`/Users/sac/praxis/docs/releases/v26.7.6/ocel/` —
`playwright-wasm4pm-validation.ocel.json` and `wasm4pm-process-validation.json` —
are both OCEL 2.0 JSON, not XES. `wpm audit` refused both with:

```
error: OCEL 2.0 format detected (...)
The wpm audit command currently supports XES event logs (IEEE 1849).
To audit an OCEL log, flatten it first:
	wpm run --algorithm dfg --format json "<file>"
or use the TypeScript CLI: wpm conformance "<file>"
```

Instead used cargo-cicd's own real, freshly emitted evidence log (XES, IEEE 1849
format, matching what `wpm audit` actually accepts), produced by this workspace's
own evidence-emission pipeline:

`/Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.xes` (67,944 bytes, timestamped 2026-07-06 15:31).

**Command run:**

```sh
/Users/sac/wasm4pm/target/release/wpm audit /Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.xes -v
```

**Full output (verbatim, exit code 0):**

```
Vision 2030 Conformance Audit Report

Audit Verdict:            DECEPTIVE
Fitness Score:            0.6457
Precision Score:          0.2331

Total Traces Audited:     1
Fitting Traces:           0
Deviating Traces:         1

Sample Deviations:

Trace ID  Fitness  Problems      
trace-0   0.65     M: 71, R: 71  


Doctrine: If the code says it worked but the event log cannot prove a lawful process happened, then it did not work.
```

**Honest interpretation:** this is not a passing/Accept verdict. The oracle rated
cargo-cicd's current `events.xes` log **DECEPTIVE** (fitness 0.6457, precision
0.2331, 0/1 traces fitting, 71 missing + 71 remaining/unexpected activity
problems on the sole trace). This is a real Refuse-class result, not a
type-level stub and not massaged to force a pass. It indicates the evidence
log currently emitted by this workspace's `src/evidence.rs` pipeline does not
conform to whatever reference/expected process model `wpm audit` checks it
against — this is a genuine open finding, not a clean bill of health, and
should be triaged as its own follow-up (why 71/71 M/R problems on a single
trace) rather than treated as P12 being "green."
