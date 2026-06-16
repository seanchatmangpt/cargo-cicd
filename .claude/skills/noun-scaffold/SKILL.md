---
name: noun-scaffold
description: Scaffolds a new `cargo cicd <noun>` command with NounCommand + VerbCommand implementations, mod registration, main.rs wiring, UI output, evidence emission, and a projection test. Use when the user says "add a noun", "new command", "scaffold <noun>", or asks to extend the CLI with a new top-level subcommand.
---
# Noun Scaffold

Step-by-step instructions for adding a new `cargo cicd <noun>` command to cargo-cicd.

## 1. Identify the noun name and its verbs

Decide on the kebab-case noun name (e.g. `deploy`) and its verbs (e.g. `show`, `run`).
Determine whether the bare noun should dispatch to a default verb (like `status` → `show`).

## 2. Study the reference implementation

Read `src/nouns/status.rs` before writing anything — it is the canonical example:
- `StatusNoun` implements `NounCommand` (name, about, verbs list).
- `StatusShowVerb` implements `VerbCommand` (name, about, run).
- `run()` delegates to an `execute()` method that returns `anyhow::Result<()>`.
- Output uses `crate::ui::panel::header(...)`, `crate::ui::panel::kv(...)`,
  `crate::ui::badge::tag(...)`, and `crate::ui::theme::paint(...)`.
- Evidence is emitted via `ProcessEvent::started` / `ProcessEvent::completed`
  then written with `crate::evidence::append_events(...)`.

## 3. Create `src/nouns/<noun>.rs`

```rust
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};
use crate::evidence::ProcessEvent;
use crate::ui::{badge, panel};
use crate::ui::badge::Verdict;

pub struct <Noun>Noun;
impl <Noun>Noun {
    pub fn new() -> Self { Self }
    // Add run_direct() if bare-noun dispatch is needed:
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

        let (mut start_evt, t0) = ProcessEvent::started("<noun>:show");
        start_evt.case_id = Some(case_id.clone());

        println!("{}", panel::header("<noun> status"));
        // ... collect data from adapters, render with panel::kv / badge::tag ...

        let verdict = "PASS";
        let mut complete_evt = ProcessEvent::completed("<noun>:show", t0, verdict);
        complete_evt.case_id = Some(case_id.clone());

        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
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

Rules for output:
- All color goes through `crate::ui::theme::paint(text, Role::*)` or `Style::paint`.
- All glyphs go through `crate::ui::symbols::*()` (e.g. `symbols::success()`), which auto-falls back to ASCII.
- Width measurement uses `crate::ui::text::display_width(s)` — never `.len()` on styled strings.
- Never print private architecture terms in help text or output.

## 4. Register in `src/nouns/mod.rs`

Add one line in alphabetical order:

```rust
pub mod <noun>;
```

## 5. Wire into `src/main.rs`

Add to the `.noun(...)` chain in `main()`:

```rust
.noun(nouns::<noun>::<Noun>Noun::new())
```

If the bare noun should dispatch to a default verb, add two places:

a. In `inject_default_verbs`, extend the `match noun` arm:
```rust
"<noun>" => Some("show"),
```

b. In the `needs_default` match and the dispatch block:
```rust
// needs_default pattern:
matches!(noun.as_str(), "status" | "publish" | "workspace" | "evidence" | "<noun>")
// dispatch block:
"<noun>" => return nouns::<noun>::<Noun>Noun::run_direct(),
```

## 6. Add a projection test in `tests/cli/command_projection.rs`

Append a new test that:
1. Runs `cargo-cicd <noun> show` with `assert_cmd::Command::cargo_bin("cargo-cicd")`.
2. Asserts `.success()` (exit 0).
3. Asserts `.stdout(predicate::str::contains("<expected substring>"))`.

Example:
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

## 7. Verify the build and test

Run these commands in order (do not skip):

```sh
cargo build
cargo test --test cli command_projection::test_<noun>_show_parses_and_runs
```

Fix any compiler errors before proceeding.

## 8. Checklist before done

- [ ] `src/nouns/<noun>.rs` compiles with no warnings.
- [ ] `src/nouns/mod.rs` declares `pub mod <noun>;`.
- [ ] `src/main.rs` registers the noun in the `.noun(...)` chain.
- [ ] Default-verb injection added if the bare noun needs it.
- [ ] Output uses only `panel`, `badge`, `theme`, `symbols` — no raw ANSI escape strings.
- [ ] A `ProcessEvent` pair (started + completed) is appended to `target/cargo-cicd/evidence/`.
- [ ] Projection test passes in `tests/cli/command_projection.rs`.
- [ ] Help text contains no forbidden terms.
