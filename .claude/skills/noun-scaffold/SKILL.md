---
name: noun-scaffold
description: Scaffolds a new `cargo cicd <noun>` command with NounCommand + VerbCommand implementations, mod registration, main.rs wiring, UI output, evidence emission, and a projection test. Use when the user says "add a noun", "new command", "scaffold <noun>", or asks to extend the CLI with a new top-level subcommand.
---

# Noun Scaffold

Trigger: "add a noun", "new command", "scaffold <noun>", or extend CLI with a new subcommand.

Reference implementation: `src/nouns/status.rs`. Read it before writing anything.

## Step 1 — Create `src/nouns/<noun>.rs`

```rust
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;
use crate::ui::{badge, panel};

pub struct <Noun>Noun;
impl <Noun>Noun {
    pub fn new() -> Self { Self }
    pub fn run_direct() -> anyhow::Result<()> { <Noun>ShowVerb.execute() }
}
impl Default for <Noun>Noun { fn default() -> Self { Self::new() } }

impl NounCommand for <Noun>Noun {
    fn name(&self) -> &'static str { "<noun>" }
    fn about(&self) -> &'static str { "One-line public description" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(<Noun>ShowVerb)]
    }
}

pub struct <Noun>ShowVerb;

impl <Noun>ShowVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        // Build OCEL
        let log = OCEL {
            event_types: vec![OCELType { name: "<noun>:show".into(), attributes: vec![] }],
            object_types: vec![],
            events: vec![OCELEvent {
                id: case_id.clone(),
                event_type: "<noun>:show".into(),
                time: chrono::Utc::now().to_rfc3339(),
                attributes: vec![],
                relationships: vec![],
            }],
            objects: vec![],
        };
        let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
        let ocel_path = evidence_dir.join(format!("{}.ocel.json", case_id));
        serde_json::to_writer(std::fs::File::create(&ocel_path)?, &evidence.inner())?;

        println!("{}", panel::header("<noun> status"));
        // render with panel::kv / badge::tag

        Ok(())
    }
}

impl VerbCommand for <Noun>ShowVerb {
    fn name(&self) -> &'static str { "show" }
    fn about(&self) -> &'static str { "Show <noun> status" }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}
```

Output rules (violations cause test failures):
- Color: `crate::ui::theme::paint(text, Role::*)` or `Style::paint` only. No raw `\x1b[` sequences.
- Glyphs: `crate::ui::symbols::*()` only. No embedded Unicode literals.
- Width: `crate::ui::text::display_width(s)` — never `.len()` on styled strings.
- Forbidden terms: none of the terms in `CLAUDE.md` FORBIDDEN section in any output/help text.

Evidence format: OCEL 2.0 JSON only. Do not emit XES for new code.

## Step 2 — Register in `src/nouns/mod.rs`

```rust
pub mod <noun>;  // alphabetical order
```

## Step 3 — Wire into `src/main.rs`

```rust
// In .noun(...) chain:
.noun(nouns::<noun>::<Noun>Noun::new())

// If bare noun needs default-verb dispatch, in inject_default_verbs match:
"<noun>" => Some("show"),

// In needs_default match:
matches!(noun.as_str(), "status" | "publish" | "workspace" | "evidence" | "<noun>")

// In dispatch block:
"<noun>" => return nouns::<noun>::<Noun>Noun::run_direct(),
```

## Step 4 — Add Projection Test

Append to `tests/cli/command_projection.rs`:

```rust
#[test]
fn test_<noun>_show_parses_and_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "show"]);
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]))
        .stdout(predicate::str::contains("<noun> status"));
}
```

## Step 5 — Verify

```sh
cargo build
cargo test --test cli command_projection::test_<noun>_show_parses_and_runs
```

Fix all compiler errors before marking done.

## Checklist

- [ ] `src/nouns/<noun>.rs` compiles with no warnings
- [ ] OCEL 2.0 evidence emitted to `target/cargo-cicd/evidence/<case_id>.ocel.json`
- [ ] `src/nouns/mod.rs` declares `pub mod <noun>;`
- [ ] `src/main.rs` registers noun; default-verb wired if needed
- [ ] Output uses only `panel`, `badge`, `theme`, `symbols` — no raw ANSI
- [ ] Projection test passes
- [ ] Help text contains no forbidden terms
