# cargo-cicd Solution Architecture

**Version:** v26.6.2
**Date:** 2026-06-03
**Status:** Canonical — supersedes ARCHITECTURE.md for law-based decisions

---

## Mission

cargo-cicd is a Level 5 process-data engine exposed as a Rust CI/CD helper.

Public: "cargo-cicd keeps Rust workspaces clean, fast, and push-ready."
Private: Every command emits process evidence. Release is gated on adjudicated evidence.

---

## Master Laws

These laws are non-negotiable. Each has a corresponding ADR.

| # | Law | ADR |
|---|-----|-----|
| L1 | CLI, integration, and domain logic live in separate crates | ADR-001 |
| L2 | Every command emits XES evidence; wasm4pm adjudicates release | ADR-002 |
| L3 | ReceiptDoctor is the primary publish gate — not internal tests | ADR-003 |
| L4 | The LSP observer reads workspace state; it never acts on it | ADR-004 |
| L5 | Receipt lifecycle follows keyed subtraction — no phantom records | ADR-005 |
| L6 | Trailing var-arg is the canonical pattern for noun-verb routing | ADR-006 |
| L7 | No silent fallback when verdict keys are absent from wpm output | ADR-007 |
| L8 | Pipeline traces are first-class; ambient traces are inadmissible | ADR-008 |
| L9 | Forbidden terms never appear in any public surface | ADR-009 |
| L10 | Publish proceeds only on an adjudicated receipt — never on internal pass | ADR-010 |

---

## Three-Crate Separation

```
┌─────────────────────────────────────────────────────────┐
│  CRATE 1 — CLI (cargo-cicd-cli)                         │
│  NounCommand + VerbCommand traits                        │
│  Responsibility: argument validation and output only     │
│  May import: clap, anyhow, integration crate            │
│  Must NOT: contain business logic or spawn processes     │
└────────────────────────┬────────────────────────────────┘
                         │ delegates immediately
┌────────────────────────▼────────────────────────────────┐
│  CRATE 2 — INTEGRATION (cargo-cicd-integration)         │
│  CliBuilder, VerbArgs, command wiring                   │
│  Responsibility: register nouns/verbs, route calls      │
│  May import: clap internals, domain crate               │
│  Must NOT: contain business logic                       │
└────────────────────────┬────────────────────────────────┘
                         │ calls pure functions
┌────────────────────────▼────────────────────────────────┐
│  CRATE 3 — DOMAIN (cargo-cicd-core)                     │
│  Pure functions in domain modules                       │
│  Responsibility: all computation and state derivation   │
│  May import: std, anyhow, serde, domain types           │
│  Must NOT: import clap, std::process::Command for CLI   │
└─────────────────────────────────────────────────────────┘
```

The arrows flow downward only. The domain crate never imports the CLI or integration crates.

---

## Evidence Gate Architecture

```
cargo-cicd command
  → emits ProcessEvent (start + complete, real UTC timestamp)
  → appends to target/cargo-cicd/evidence/events.jsonl
  → rebuilds target/cargo-cicd/evidence/events.xes

cargo cicd evidence doctor
  → build_receipt_json(events, command, 0)  — OCEL 2.0 compliant receipt
  → wpm receipt doctor --format json --strict latest.json
  → state: Admitted

cargo cicd publish run
  → ReceiptDoctor::emit_and_adjudicate()
  → wpm receipt doctor --format json --strict
  → RECEIPT_DOCTOR:accepted → publish proceeds
  → RECEIPT_DOCTOR:refused → AndonPull (publish blocked)
  → oracle unavailable → WARN:oracle_unavailable (proceed with warning)
```

### Evidence Format

- Format: XES (XML Event Stream)
- Evidence directory: `target/cargo-cicd/evidence/`
- OCEL 2.0 receipt: `target/cargo-cicd/evidence/receipts/latest.json`
- Receipt uses `algorithms`-based OCEL2 structure

### wpm Oracle Commands

| Command | Use |
|---------|-----|
| `wpm audit <file.xes>` | Primary adjudication of XES evidence |
| `wpm receipt doctor --format json --strict` | Receipt structure validation |
| `wpm doctor` | Environment health check |
| `wpm lean` | Workspace analysis |
| `wpm spc status` | Statistical process control status |

### Verdict Handling

| wpm Verdict | Publish Outcome |
|-------------|-----------------|
| `Accept` | ADJUDICATED:accept — proceed |
| Refused (exit 1) | AndonPull — publish blocked |
| Not available | WARN:oracle_unavailable — proceed with warning |

---

## Declared Process Model

The manufacturing pipeline is declared in two artifacts:

1. `ontology/cicd-process.ttl` — OWL/PROV-N ontology declaring `CicdActivity` subclasses for each command, their predecessors, and the `ProcessEvidence` entity. `PublishCommand` has `requiresAdjudicatedEvidence: true` and `requiredConformanceScore: 1.0`.

2. `process/cicd-process.powl.json` — POWL choice graph with 10 activities, partial ordering constraints (`status:show → test:changed → publish:run`), required stages, object type lifecycles for `ProcessEvidence`, and the admission gate.

### Declared Activity Order

```
status:show → status:audit → test:changed → publish:run
```

Single-stage traces produce fitness 0.0 — classified DECEPTIVE. Release requires the full declared sequence.

---

## cicd.toml Carrier

`cicd.toml` is the persistent output of domain functions — not a log file, not a cache.

```toml
[workspace]
name = "my-workspace"

[state]
publish_ready = false

[target]
count = 3

[[events]]
command = "status show"
timestamp = "2026-06-03T12:00:00Z"
exit_code = 0
```

Rules:
- Domain functions read from and write to `cicd.toml` as structured state.
- Any tool needing process state reads `cicd.toml`; it does not re-derive by re-running CLI commands.
- `publish_ready = true` only if `wpm receipt doctor --strict` returns `Admitted`.

---

## Noun-Verb CLI Grammar

The CLI uses `clap-noun-verb` (local crate at `/Users/sac/clap-noun-verb`). Each noun is a module in `src/nouns/` implementing `NounCommand`. Verbs implement `VerbCommand`. Default verb injection in `main.rs::inject_default_verbs()` enables bare nouns.

**Canonical nouns:** `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`, `evidence`

### Trailing Var-Arg Pattern

```rust
// CORRECT: trailing var-arg receives all positional args after the noun
fn run(&self, args: &VerbArgs) -> anyhow::Result<()> {
    let targets: Vec<String> = args.trailing_vararg("targets")?;
    // ...
}
```

The trailing var-arg is the only sound way to accept open-ended positional arguments. Any other pattern produces ambiguous parse results under clap-noun-verb routing.

---

## LSP Observer Pattern

The LSP integration reads workspace symbol information to inform `EngineState`. It is an observer — it never mutates files, never runs commands, never acts on what it sees.

```rust
// CORRECT: observer reads, domain functions act
let symbols = lsp_adapter.workspace_symbols()?;
let state = engine_state.with_lsp_symbols(symbols);
domain::compute_target_plan(&state)?;

// WRONG: observer acting
lsp_adapter.rename_symbol("old", "new")?; // forbidden
```

---

## Receipt Lifecycle (Keyed Subtraction)

Receipts are managed by keyed subtraction: each receipt key maps to exactly one live record. Emitting a new receipt for the same key replaces the prior one — no accumulation.

```
emit(key="publish:run", event) → receipts[key] = event   // replace
```

Phantom records (receipts without a corresponding live event) are inadmissible. An empty key slot is preferable to a stale record.

---

## Feature Flags

| Flag | Enables |
|------|---------|
| `process-data` | Level 5 engine internals |
| `autonomic` | implies `process-data`; policy/suggest mode |
| `wasm4pm` | implies `process-data`; richer runtime integration |
| `contrib` | implies `process-data` |

The `wasm4pm` feature flag gates richer runtime integration. It does NOT gate the evidence-gate acceptance law — that law holds unconditionally.

---

## Engine State Aggregate

`EngineState` is the aggregate root — a struct of all runtime dimensions:

| Dimension | Type |
|-----------|------|
| Workspace | `WorkspaceState` |
| Toolchain | `ToolchainState` |
| Target | `TargetState` |
| Changed files | `ChangedFileState` |
| Test plan | `TestPlanState` |
| Trybuild | `TrybuildState` |
| Git phase | `GitPhaseState` |
| Process events | `ProcessEventState` |
| Artifacts | `ArtifactState` |
| Policies | `PolicyState` |
| Projection | `ProjectionProfile` |

Nouns read from `EngineState`; adapters populate it from external sources. No business logic lives in adapters.

---

## Test Hierarchy

### Tier 1 — Non-closing tests
- Unit, smoke, projection, invariant tests
- Tools: `assert_cmd`, `tempfile`, fixture workspaces
- Files: `tests/invariants.rs`, `tests/cli/`, `tests/feature_projection.rs`
- May pass independently of wasm4pm

### Tier 2 — Evidence-gate tests (release gate)
- Must emit XES process evidence
- Must invoke wpm oracle: `wpm audit <file.xes>`
- Must assert wasm4pm Accept/Refuse verdict
- Files: `tests/wasm4pm_evidence_gate.rs`, `tests/wasm4pm_evidence_mutation.rs`, `tests/wasm4pm_refusal_cases.rs`

**No release may claim completion solely from Tier 1 tests.**

### 7 Non-negotiable Invariants

These invariants are enforced in `tests/invariants.rs`:

1. Public boundary: only sanctioned types cross crate boundaries
2. Evidence emission: every command emits at least one event
3. Verdict routing: Accept/Refuse/NotAvailable are the only valid outcomes
4. Receipt lifecycle: no phantom receipts
5. Forbidden terms: no forbidden terms in any public surface
6. Feature flag surface: flag combinations are stable and documented
7. Publish gate: publish never proceeds without oracle verdict check

---

## Adapters

Each adapter owns one external source:

| Adapter | Source |
|---------|--------|
| `GitStatusAdapter` | `git status` output |
| `TargetScannerAdapter` | cargo metadata |
| `ToolchainDetector` | rustup/cargo version |
| `CargoMetadataAdapter` | `cargo metadata` JSON |
| `ChangedFileDetector` | git diff |
| `CicdTomlWriter` | `cicd.toml` writes |
| `TrybuildDetector` | trybuild test discovery |
| `WpmShellOutAdapter` | wpm binary invocation |

Adapters translate external representations into the internal state model. No business logic in adapters.

---

## Anti-Patterns

### Silent Verdict Key Fallback

```rust
// WRONG: silently treats missing key as Accept
let verdict = output.get("state").unwrap_or("Admitted");

// CORRECT: absent key is an error
let verdict = output.get("state")
    .ok_or_else(|| anyhow::anyhow!("wpm output missing 'state' key"))?;
```

### Ambient Trace Admissibility

```rust
// WRONG: accepting any trace that mentions the right activity names
if trace.contains("publish") { accept() }

// CORRECT: trace must follow declared pipeline order with real timestamps
if pipeline_trace.conforms_to_declared_model()? { accept() }
```

### Logic Accumulation in run()

```rust
// WRONG: all logic in run()
impl VerbCommand for MyVerb {
    fn run(&self, _args: &VerbArgs) -> anyhow::Result<()> {
        let output = std::process::Command::new("git").arg("status").output()?;
        // 50 more lines of logic...
        Ok(())
    }
}

// CORRECT: delegate immediately
impl VerbCommand for MyVerb {
    fn run(&self, args: &VerbArgs) -> anyhow::Result<()> {
        let result = domain::my_logic(args.get("flag"))?;
        println!("{}", result);
        Ok(())
    }
}
```

---

## Manufacturing Pipeline

```
ggen.toml
  + ontology/cargo-cicd.ttl
  + SPARQL queries in queries/
  + Tera templates in templates/
  → ggen manufacture
  → noun modules in src/nouns/
  → CLI test scaffolding
```

Run `ggen` to regenerate from ontology changes. The ontology is the source of truth for noun/verb surface and evidence model.

---

## Forbidden Terms in Public Surfaces

A set of 10 internal manufacturing terms must never appear in any public doc, CLI help text, or public API. The authoritative list is in the project CLAUDE.md under "FORBIDDEN in public docs/CLI/help text".

These are internal manufacturing terms. Their presence in public surfaces is a defect. See ADR-009 and the invariants test for enforcement details.

---

## Canonical Build Commands

```sh
cargo make build     # preferred
cargo make check     # lint + type-check
cargo make test      # all tests
cargo test --test invariants
cargo test --test wasm4pm_evidence_gate
cargo test --features wasm4pm
```
