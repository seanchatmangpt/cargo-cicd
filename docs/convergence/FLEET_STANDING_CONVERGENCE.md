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

_(pending)_
