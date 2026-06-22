# cargo-cicd Architecture

Version: 26.6.2 | State carrier: `cicd.toml` | Oracle: `wpm` binary

---

## 1. Top-Level Flow

```
User: cargo cicd <noun> [<verb>] [flags]
  → Clap parser
  → Noun Router → Verb Dispatcher
  → EngineState::from_workspace()   # parallel adapter queries
  → Verb implementation (emits ProcessEvents)
  → Evidence dir (OCEL JSON)
  → wpm audit <file.ocel.json>      # shell-out only, never linked
  → Accept | Refuse | Blocked
```

Default verb injection (`main.rs::inject_default_verbs`):

| Bare noun | Resolves to |
|-----------|-------------|
| `status` | `status show` |
| `publish` | `publish run` |
| `workspace` | `workspace doctor` |
| `evidence` | `evidence doctor` |

---

## 2. wasm4pm-compat Integration (CANONICAL)

**Cargo.toml dependency:**
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

**Required imports — never hand-roll these types:**
```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence, AdmittedOcelEvidence};
use wasm4pm_compat::state::{Raw, Admitted};
use wasm4pm_compat::witness::Ocel20;
use wasm4pm_compat::receipt::Receipt;
use wasm4pm_compat::conformance::ConformanceResult;
```

**OCEL 2.0 JSON shape (what wpm expects on disk):**
```json
{ "eventTypes": [...], "objectTypes": [...], "events": [...], "objects": [...] }
```
- `OCELEvent.relationships`: `Vec<OCELRelationship { objectId, qualifier }>`
- `OCELObject.relationships`: `Vec<OCELObjectRelationship { objectId, qualifier }>`

**Emission pattern (every noun handler):**
```rust
// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Shell out
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

**Object types in cargo-cicd domain:**
`Workspace` · `Crate` · `TestRun` · `GitCommit` · `Release` · `Receipt` · `EvidenceFile` · `Policy` · `Toolchain`

**FORBIDDEN:**
- Hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs
- Calling `wpm` on `.xes` files in new code
- Adjudicating inside cargo-cicd (invariant E1)
- Extending `evidence_xes_v2.rs` — LEGACY, do not touch
- `src/ocel.rs` — DELETE if present; replace with imports above

---

## 3. Evidence Invariants

| ID | Rule |
|----|------|
| E1 | cargo-cicd never adjudicates itself; only `wpm` issues verdicts |
| E2 | XES/OCEL file must exist before `audit_xes()` is called |
| E3 | Oracle unavailable + non-Blocked expectation = panic |
| E4 | Tests assert on wpm verdict only, never on internal state |
| E5 | XES groups by `case_id` into `<trace>` elements |
| E6 | JSONL mirrors XES |
| E7 | `Blocked` is a first-class expectation, not an error |

Verdicts: `PASS` · `WARN` · `FAIL` · `WARN:dry_run` · `WARN:oracle_unavailable`

---

## 4. EngineState (Aggregate Root)

`EngineState::from_workspace()` — called once per command, queries all adapters in parallel, silently handles failures.

```
EngineState {
  workspace:     WorkspaceState       # name, root_path, members, toolchain, rust_edition
  toolchain:     ToolchainState       # active, rust_version
  target:        TargetState          # path, total_size_bytes
  changed_files: ChangedFileState     # base_ref, total_changed, changed_rs_files, changed_test_files
  test_plan:     TestPlanState        # estimated_count, conservative_mode
  trybuild:      TrybuildState        # fixture_count, changed_fixtures
  git_phase:     GitPhaseState        # branch, dirty_files, staged_files, untracked_files, ahead, behind
  process_events: ProcessEventState   # events: Vec<ProcessEvent>
  artifacts:     ArtifactState        # binaries, publish_ready
  policies:      PolicyState          # entries: Vec<PolicyEntry>
  projection:    ProjectionProfile    # version
}
```

**Adapter → State mapping:**

| Adapter | Source | Populates |
|---------|--------|-----------|
| `CargoMetadataAdapter` | `Cargo.toml`, `Cargo.lock` | `workspace.*` |
| `GitStatusAdapter` | `.git/HEAD`, `.git/refs/*` | `git_phase.*` |
| `ToolchainDetector` | `rust-toolchain`, `rustc --version` | `toolchain.*` |
| `TargetScannerAdapter` | `target/` | `target.*` |
| `ChangedFileDetector` | `git diff origin/main` | `changed_files.*` |
| `TrybuildDetector` | `tests/trybuild/`, `tests/ui/` | `trybuild.*` |
| `CicdTomlWriter` | `cicd.toml` | persistence only |

Adapter rules: one adapter = one source · read-only (except `CicdTomlWriter`) · no business logic · silent failures.

---

## 5. Noun-Verb Registry

| Noun | Module | Verbs |
|------|--------|-------|
| `status` | `src/nouns/status.rs` | `show` (default) |
| `git` | `src/nouns/git.rs` | `status`, `close`, `diff`, `stage`, `commit`, `fetch` |
| `test` | `src/nouns/test.rs` | `changed`, `all`, `by-name` |
| `workspace` | `src/nouns/workspace.rs` | `doctor` (default), `members`, `graph` |
| `target` | `src/nouns/target.rs` | `clean`, `inspect`, `estimate` |
| `publish` | `src/nouns/publish.rs` | `run` (default), `dry-run`, `yank` |
| `trybuild` | `src/nouns/trybuild.rs` | `test`, `update` |
| `evidence` | `src/nouns/evidence.rs` | `doctor` (default), `audit`, `export` |
| `lsp` | `src/nouns/lsp.rs` | `run`, `check` |
| `pipeline` | `src/nouns/pipeline.rs` | `run`, `dry-run` |
| `analyze` | `src/nouns/analyze.rs` | `deps`, `complexity`, `coverage` |
| `autoarch` | `src/nouns/autoarch.rs` | `show`, `apply` |
| `certification` | `src/nouns/certification.rs` | `check`, `sign`, `verify` |
| `sbom` | `src/nouns/sbom.rs` | `generate`, `verify` |
| `ui` | `src/nouns/ui.rs` | `demo`, `dashboard` |

---

## 6. Evidence Emission Lifecycle

```rust
// Every verb handler:
ProcessEvent::started("noun:verb");          // push to ProcessEventState
// [do work]
ProcessEvent::completed("noun:verb", start, end, verdict);  // push verdict

// Persist to target/cargo-cicd/evidence/:
//   events.jsonl  — JSONL mirror
//   <name>.ocel.json  — OCEL 2.0 for wpm

// Audit:
// $ cargo cicd evidence doctor
// $ wpm audit target/cargo-cicd/evidence/<name>.ocel.json
// → Accept | Refuse | Blocked
```

---

## 7. Autonomic Policies

All policies: suggest-mode only (read-only). `EngineState` is never mutated.

| Policy | Signal | Action |
|--------|--------|--------|
| `GitPhaseDirtyPolicy` | `dirty_files > 0` | Suggest commit |
| `TargetPressurePolicy` | `size > 5GB` | Suggest `target clean` |
| `ToolchainMismatchPolicy` | toolchain mismatch | Suggest update |
| `TrybuildChangedPolicy` | changed fixtures | Suggest re-run |
| `BranchBehindPolicy` | `behind > 10` | Suggest pull |
| `EvidenceStalePoliciy` | stale events | Suggest evidence run |
| `PublishNotAdjudicatedPolicy` | no wpm verdict | Suggest `evidence doctor` |

Output per policy: `name`, `mode` (Disabled|Suggest|Apply), `signals`, `recommendation`, `verdict` (Pass|Warn|Fail).

Add a policy: create `src/policies/<name>.rs` with `fn eval(&EngineState) -> PolicyEntry`, register in `policies::run_all_policies()`, test in `tests/autonomic_policies.rs`.

---

## 8. Feature Flags

| Flag | Implies | Enables |
|------|---------|--------|
| `process-data` | — | EngineState, adapters, cicd.toml, LSP analyzers |
| `autonomic` | `process-data` | PolicyEngine, suggest-mode policies |
| `wasm4pm` | `process-data` | wpm oracle shell-out, OCEL evidence |
| `affidavit` | `process-data` | `affi` receipt engine, `affidavit` noun |
| `advanced` | — | parallel_scan, blake3, tracing, miette, moka, bitcode, petgraph, jiff, hdrhistogram, aho-corasick |

```sh
cargo build                              # minimal
cargo build --features process-data     # full engine
cargo build --features autonomic        # + policies
cargo build --features wasm4pm          # + evidence gate
cargo build --features autonomic,wasm4pm # release build
```

---

## 9. Test Tiers

**Tier 1 — Unit/Smoke** (no external deps):
```sh
cargo test --test invariants           # 7 public boundary invariants
cargo test --test cli
cargo test --test feature_projection
cargo test --test cicd_toml_truth
cargo test --test autonomic_policies
cargo test --test changed_tests
cargo test --test git_phase_closure
```

7 invariants: no forbidden terms · no destructive action without `--confirm` · no full trybuild by default · lowercase noun names · binary is `cargo-cicd` · status exits 0 · git close has safety warnings

**Tier 2 — Evidence Gate** (requires `wpm` on PATH):
```sh
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```
Assert on wpm verdict (`Accept`/`Refuse`/`Blocked`). Use `ExpectedWpmVerdict::Blocked` when wpm unavailable.

---

## 10. Release Gate

```sh
cargo make test
cargo test --test invariants
cargo build --features autonomic,wasm4pm
cargo test --test wasm4pm_evidence_gate
wpm receipt doctor --format json --strict receipts/*.json
# verify: ggen run, CHANGELOG updated, version bumped, git clean
git tag -a v<VERSION> -m "Release v<VERSION>"
git push origin main --tags
```

Forbidden terms (enforced by `invariant_public_boundary_no_forbidden_terms_in_all_help`):
`ALIVE` · `Inspection Gate` · `wall` · `Nehemiah` · `Field8` · `Instinct8` · `Cargo Court` · `AGI` · `Truex` · `CONSTRUCT8`

---

## 11. File Structure

```
src/
  main.rs                     # CLI entry, inject_default_verbs()
  evidence.rs                 # ProcessEvent, evidence emission API
  cicd_toml.rs                # cicd.toml schema
  adapters/                   # External source → EngineState (pure translators)
  engine/                     # EngineState aggregate root + state structs
  nouns/                      # Noun implementations
  policies/                   # 7 autonomic policy implementations
  autonomic/                  # PolicyEngine, signals, CicdPolicy trait
  integrations/
    wasm4pm_shell.rs          # shell invocation of wpm
    wasm4pm_current.rs        # OCEL serialization
tests/
  invariants.rs
  cli/
  wasm4pm_evidence_gate.rs
  wasm4pm_evidence_mutation.rs
  wasm4pm_refusal_cases.rs
  fixtures/                   # simple/ multi-member/ with-trybuild/ dirty-git/
ontology/
  cargo-cicd-capabilities.ttl # manufacturing spec → ggen → src/nouns/*.rs
ggen.toml                     # code generation config
cicd.toml                     # generated workspace state (gitignored)
```

**Manufacturing pipeline:** Edit TTL → run `ggen` → implement handlers in `src/nouns/`. Never hand-edit generated files.
