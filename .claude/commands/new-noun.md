---
description: Scaffold a new CLI noun: module, registration, UI output, OCEL evidence emission, projection test.
argument-hint: [noun-name]
allowed-tools: Bash, Read, Grep, Edit, Write
---

Trigger: user says "add noun", "new command", or "scaffold $ARGUMENTS".

Noun name: `$ARGUMENTS` (snake_case module, kebab-case CLI).

## 0 — Read canonical pattern before writing

```
Read: src/nouns/status.rs
Read: src/nouns/mod.rs
Read: src/main.rs
```

## 1 — Create `src/nouns/$ARGUMENTS.rs`

Failure mode: using `ProcessEvent` without OCEL emission skips wpm adjudication — tests will fail at Tier 2.

Canonical template:

```rust
//! Noun: $ARGUMENTS

use anyhow::Result;
use clap_noun_verb::{NounCommand, VerbCommand};
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;
use crate::engine::EngineState;
use crate::ui;

pub struct Show;

impl VerbCommand for Show {
    fn name(&self) -> &'static str { "show" }

    fn run(&self, state: &EngineState) -> Result<()> {
        // 1. Build OCEL
        let log = OCEL {
            event_types: vec![OCELType { name: "$ARGUMENTS:show".into(), attributes: vec![] }],
            object_types: vec![],
            events: vec![OCELEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: "$ARGUMENTS:show".into(),
                time: chrono::Utc::now().to_rfc3339(),
                attributes: vec![],
                relationships: vec![],
            }],
            objects: vec![],
        };
        // 2. Wrap
        let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
        // 3. Serialize
        let path = state.evidence_path("$ARGUMENTS-show");
        let f = std::fs::File::create(&path)?;
        serde_json::to_writer(f, &evidence.inner())?;
        // 4. Shell out (wpm audit path) — verdict is Accept|Refuse|Blocked
        // cargo-cicd never adjudicates itself (invariant E1); wpm issues verdicts

        ui::panel::render_panel("$ARGUMENTS", &format!("{:?}", state))?;
        Ok(())
    }
}

pub struct ${ARGUMENTS_PASCAL}Noun;

impl NounCommand for ${ARGUMENTS_PASCAL}Noun {
    fn name(&self) -> &'static str { "$ARGUMENTS" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> { vec![Box::new(Show)] }
}
```

Replace `${ARGUMENTS_PASCAL}` with PascalCase of noun name (e.g. `my-noun` → `MyNoun`).

FORBIDDEN: hand-rolling `OcelLog`/`OcelEvent`/`OcelObject` structs. Import from `wasm4pm_compat`.
FORBIDDEN: raw `println!` for output. Use `crate::ui`.

## 2 — Register in `src/nouns/mod.rs`

Add alphabetically:
```rust
pub mod $ARGUMENTS;
```

## 3 — Register in `src/main.rs`

In noun list:
```rust
nouns.push(Box::new(crate::nouns::$ARGUMENTS::${ARGUMENTS_PASCAL}Noun));
```

In `inject_default_verbs()`:
```rust
"$ARGUMENTS" => Some("show"),
```

## 4 — Verify (read-only, no `cargo build`)

- `src/nouns/mod.rs`: `pub mod $ARGUMENTS` present.
- `src/main.rs`: noun in list + `inject_default_verbs` entry.
- `src/nouns/$ARGUMENTS.rs`: both `NounCommand` and `VerbCommand` implemented.

## 5 — Add projection test

Append to `tests/feature_projection.rs` (fallback: `tests/cli.rs`):

```rust
#[test]
fn test_$ARGUMENTS_show_exits_clean() {
    assert_cmd::Command::cargo_bin("cargo-cicd").unwrap()
        .args(["$ARGUMENTS", "show"])
        .assert().success();
}
```

## 6 — Commit message

```
feat(cli): add $ARGUMENTS noun with show verb, OCEL evidence emission, and projection test
```

## Checklist

- [ ] `src/nouns/$ARGUMENTS.rs`: `NounCommand` + `VerbCommand` + OCEL emission via `wasm4pm_compat`
- [ ] Output via `crate::ui`, not `println!`
- [ ] `pub mod $ARGUMENTS` in `src/nouns/mod.rs` (alphabetical)
- [ ] Noun + default verb in `src/main.rs`
- [ ] Projection test passing
