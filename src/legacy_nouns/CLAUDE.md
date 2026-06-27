# src/nouns — Noun-Verb CLI Command Modules

This directory contains one module per noun in the `cargo cicd <noun> <verb>` CLI.
Each noun module implements the `clap-noun-verb` grammar from the local
`clap-noun-verb` crate.

---

## Noun-Verb Grammar

Every noun module must:

1. Define a top-level struct (e.g., `StatusNoun`) that implements `NounCommand`.
2. Define one struct per verb (e.g., `StatusShowVerb`, `StatusAuditVerb`), each
   implementing `VerbCommand`.
3. Return the list of verbs from `NounCommand::verbs()`.
4. Implement `VerbCommand::run(&self, state: &EngineState)` for each verb —
   this is where output is produced and process events are emitted.

**Current nouns and their verbs:**

| File | Noun | Verbs |
|---|---|---|
| `status.rs` | `status` | `show`, `audit` |
| `target.rs` | `target` | `show`, `prune` |
| `test.rs` | `test` | `changed` |
| `trybuild.rs` | `trybuild` | `changed` |
| `git.rs` | `git` | `status`, `close` |
| `publish.rs` | `publish` | `run` |
| `workspace.rs` | `workspace` | `doctor` |
| `evidence.rs` | `evidence` | `doctor`, `audit` |
| `ui.rs` | `ui` | `demo`, `dashboard` |
| `lsp.rs` | `lsp` | _(see file)_ |
| `pipeline.rs` | `pipeline` | _(see file)_ |

---

## Default-Verb Injection

Bare noun invocations (e.g., `cargo cicd status` with no verb) are handled by
`main.rs::inject_default_verbs()`. When adding a new noun, register its default
verb there:

```rust
// In main.rs::inject_default_verbs()
"status" => Some("show"),
"target" => Some("show"),
"workspace" => Some("doctor"),
// add your noun here
"mynoun" => Some("mydefaultverb"),
```

Without this entry, a bare `cargo cicd mynoun` will print an error instead of
running the most common verb.

---

## Adding a New Noun

1. Create `src/nouns/<noun>.rs`.
2. Implement `NounCommand` for a `<Noun>Noun` struct and `VerbCommand` for each
   verb struct.
3. Add `pub mod <noun>;` to `src/nouns/mod.rs` and register the noun in the
   noun list returned by the top-level noun dispatcher.
4. Add the default verb mapping in `main.rs::inject_default_verbs()`.
5. Emit at least one `ProcessEvent` per verb run (see Process Events section
   below).
6. Render all output through `crate::ui` (see `src/ui/CLAUDE.md`).
7. Add a fixture-based integration test in `tests/cli/` or `tests/invariants.rs`
   asserting the public-boundary output contract (see below).

---

## Process Events

Every verb must emit process events to `target/cargo-cicd/evidence/` so that
wasm4pm can adjudicate the process trace.

Use `src/evidence.rs` to construct and write events:

```rust
// Minimal pattern — adapt to the actual evidence API in evidence.rs
let event = ProcessEvent::new(noun, verb, outcome);
evidence::emit(&event, &state.artifact_state.evidence_dir)?;
```

Events are written as XES (XML Event Stream) format. The evidence directory is
`target/cargo-cicd/evidence/` and is set via the `CICD_EVIDENCE_DIR` env var in
`.claude/settings.json`.

**Why this matters:** No release passes the wasm4pm evidence gate unless every
command emits valid XES that `wpm audit` accepts. A noun that skips event
emission will cause the evidence-gate tests to fail, blocking release closure.

---

## Public-Boundary Output Contracts

The `tests/invariants.rs` test enforces non-negotiable output contracts. If you
change the output of any noun/verb, verify these pass:

| Command | Required output (substring match) |
|---|---|
| `cargo cicd status show` | Must print `"cargo-cicd workspace status"` |
| `cargo cicd workspace doctor` | Must print `"workspace doctor"` and `"Cargo.toml"` |
| `cargo cicd target show` | Must include target count and size summary |
| `cargo cicd git status` | Must reflect the current git phase from `cicd.toml [state]` |
| `cargo cicd evidence doctor` | Must print verdict line from `wpm receipt doctor` |

Run the invariants suite after any noun output change:

```sh
cargo test --test invariants
```

Adding a new noun does not require a new invariant entry by default, but if the
noun produces output consumed by external tools or scripts, add a contract to
`tests/invariants.rs` before merging.

---

## Rendering Output

All terminal output — colour, glyphs, tables, panels, badges — must go through
`crate::ui`. Never write raw ANSI codes or hard-coded Unicode glyphs in noun
modules.

```rust
// Correct
use crate::ui::{style::Style, symbols, table::Table};

// Wrong
println!("\x1b[32m{}\x1b[0m", msg);
println!("✓ {}", msg);
```

See `src/ui/CLAUDE.md` for the full rendering rules and how to add new
components.

---

## Reading from EngineState

Nouns must not call external processes or read files directly. All runtime data
comes from `EngineState`, which is built by adapters in `src/adapters/` before
`VerbCommand::run` is called. If a noun needs data that is not currently in
`EngineState`, the right fix is to extend the relevant adapter and state struct,
not to add `std::process::Command` calls inside the noun.
