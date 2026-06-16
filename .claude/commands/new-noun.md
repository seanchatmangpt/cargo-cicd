---
description: Guide adding a new CLI noun: module scaffold, registration, UI rendering, process-event emission, and projection test.
argument-hint: [noun-name]
allowed-tools: Bash, Read, Grep, Edit, Write
---

You are adding the CLI noun **$ARGUMENTS** to cargo-cicd. Work through every step below in order. After each step, confirm the change is in place before moving to the next.

The noun name is: `$ARGUMENTS`
Use it verbatim (snake_case for the module, kebab-case for the CLI name if they differ).

---

## Step 0 — Understand the existing pattern

Read an existing noun module to understand the required structure:

```
Read: src/nouns/status.rs      ← canonical example
Read: src/nouns/mod.rs         ← registration
Read: src/main.rs              ← inject_default_verbs + noun routing
```

Note the imports, trait impls, and how the noun calls `crate::ui` for output.

---

## Step 1 — Create `src/nouns/$ARGUMENTS.rs`

Create `src/nouns/$ARGUMENTS.rs` implementing the noun. The file must:

1. Import the `NounCommand` and `VerbCommand` traits from `clap_noun_verb`.
2. Define a primary verb (e.g. `Show`) that implements `VerbCommand`:
   - `fn name() -> &'static str` — returns the verb name.
   - `fn run(&self, state: &EngineState) -> anyhow::Result<()>` — reads from `state` and renders via `crate::ui`.
3. Define the noun struct implementing `NounCommand`:
   - `fn name() -> &'static str` — returns `"$ARGUMENTS"`.
   - `fn verbs() -> Vec<Box<dyn VerbCommand>>` — returns all verbs for this noun.
4. Emit a `ProcessEvent` so the noun's activity is recorded in evidence:
   ```rust
   use crate::evidence::ProcessEvent;
   let event = ProcessEvent::new("$ARGUMENTS:show");
   state.emit_event(event)?;
   ```
5. Render output using `crate::ui`:
   ```rust
   use crate::ui::{panel, text, badge};
   panel::render_panel("$ARGUMENTS", &content)?;
   ```

Skeleton template:

```rust
//! Noun: $ARGUMENTS

use anyhow::Result;
use clap_noun_verb::{NounCommand, VerbCommand};
use crate::engine::EngineState;
use crate::evidence::ProcessEvent;
use crate::ui;

// ── verbs ────────────────────────────────────────────────────────────────────

pub struct Show;

impl VerbCommand for Show {
    fn name(&self) -> &'static str { "show" }

    fn run(&self, state: &EngineState) -> Result<()> {
        let event = ProcessEvent::new(concat!("$ARGUMENTS", ":show"));
        state.emit_event(event)?;

        ui::panel::render_panel("$ARGUMENTS", &format!("{:?}", state))?;
        Ok(())
    }
}

// ── noun ─────────────────────────────────────────────────────────────────────

pub struct $ARGUMENTS_pascal_case_Noun;

impl NounCommand for $ARGUMENTS_pascal_case_Noun {
    fn name(&self) -> &'static str { "$ARGUMENTS" }

    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(Show)]
    }
}
```

Replace `$ARGUMENTS_pascal_case_Noun` with the PascalCase version of the noun name (e.g. `my-noun` → `MyNounNoun`).

---

## Step 2 — Register in `src/nouns/mod.rs`

Add a `pub mod $ARGUMENTS;` line in `src/nouns/mod.rs` alongside the existing noun modules. Keep the list alphabetically sorted.

---

## Step 3 — Register in `src/main.rs`

In `main.rs`, locate the noun-routing block (the `match` or `register_nouns` call) and add:

```rust
nouns.push(Box::new(crate::nouns::$ARGUMENTS::$ARGUMENTS_pascal_case_Noun));
```

Then, in `inject_default_verbs()`, map the bare noun to its default verb:

```rust
"$ARGUMENTS" => Some("show"),
```

This ensures `cargo cicd $ARGUMENTS` (with no verb) routes to `cargo cicd $ARGUMENTS show`.

---

## Step 4 — Verify registration compiles (read-only check)

Do NOT run `cargo build` (other agents may be editing source). Instead:

- Read `src/nouns/mod.rs` and confirm the new `pub mod` line is present.
- Read `src/main.rs` and confirm the noun is registered in both the noun list and `inject_default_verbs`.
- Read `src/nouns/$ARGUMENTS.rs` and confirm `NounCommand` and `VerbCommand` are both implemented.

---

## Step 5 — Add a CLI projection test

Open `tests/feature_projection.rs` (or the closest equivalent projection test file). Add a test that:

1. Invokes `cargo cicd $ARGUMENTS show` (using `assert_cmd::Command`).
2. Asserts exit code 0.
3. Asserts the output contains something noun-specific (the panel header, for instance).

Example:

```rust
#[test]
fn test_$ARGUMENTS_show_exits_clean() {
    let mut cmd = assert_cmd::Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["$ARGUMENTS", "show"])
       .assert()
       .success();
}
```

If `tests/feature_projection.rs` already has a projection scaffold, add the new test alongside the existing ones. If the file is owned by another agent, add the test to `tests/cli.rs` instead.

---

## Step 6 — Commit-message suggestion

When all changes are in place, suggest the commit message:

```
feat(cli): add $ARGUMENTS noun with show verb, ProcessEvent emission, and projection test
```

---

## Checklist

- [ ] `src/nouns/$ARGUMENTS.rs` created with `NounCommand` + `VerbCommand` impls
- [ ] `ProcessEvent` emitted in the verb's `run()` method
- [ ] Output rendered via `crate::ui` (not raw `println!`)
- [ ] `pub mod $ARGUMENTS;` added to `src/nouns/mod.rs`
- [ ] Noun registered in `src/main.rs` noun list
- [ ] Default verb wired in `inject_default_verbs()`
- [ ] Projection test added in `tests/`
