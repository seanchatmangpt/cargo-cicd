See [SOLUTION_ARCHITECTURE.md](SOLUTION_ARCHITECTURE.md) for the canonical law-based architecture.

# cargo-cicd Architecture

## Three-Tier Domain Separation

cargo-cicd enforces a three-tier separation between presentation, integration, and domain logic.
Each tier has a single responsibility and strict import rules.

```
┌─────────────────────────────────────────────────────────┐
│  TIER 1 — PRESENTATION LAYER (CLI)                      │
│  NounCommand + VerbCommand traits                        │
│  Responsibility: argument validation and output only     │
│  May import: clap, anyhow, integration layer            │
│  Must NOT: contain business logic or spawn processes     │
└────────────────────────┬────────────────────────────────┘
                         │ delegates immediately
┌────────────────────────▼────────────────────────────────┐
│  TIER 2 — INTEGRATION LAYER                             │
│  CliBuilder, VerbArgs, command wiring                   │
│  Responsibility: register nouns/verbs, route calls      │
│  May import: clap internals, domain layer               │
│  Must NOT: contain business logic                       │
└────────────────────────┬────────────────────────────────┘
                         │ calls pure functions
┌────────────────────────▼────────────────────────────────┐
│  TIER 3 — DOMAIN LOGIC LAYER                            │
│  Pure functions in domain modules                       │
│  Responsibility: all computation and state derivation   │
│  May import: std, anyhow, serde, domain types           │
│  Must NOT: import clap, std::process::Command for CLI   │
└─────────────────────────────────────────────────────────┘
```

The arrows flow downward only. Tier 3 never imports Tier 1 or Tier 2.

---

## Why This Matters

**Tier 3 domain logic is testable without invoking a CLI.**
Unit tests call `workspace_state()`, `target_scan()`, or `git_phase_close()` directly
with typed arguments. No subprocess, no arg parsing, no global state required.

**Tier 1 is replaceable without touching domain logic.**
If you add an HTTP endpoint, a TUI, or a JSON output mode, Tier 3 functions remain
unchanged. The new surface is a new Tier 1 implementation that calls the same domain
functions.

**A flat `run()` method that does everything is the wrong pattern.**
When `run()` contains subprocess calls, file I/O, parsing, and output formatting all
in one body, none of it is independently testable or reusable. It also couples the
domain behavior to the specific CLI invocation context, making it impossible to call
the same logic from a test or a different front-end without refactoring.

---

## cargo-cicd Implementation

The existing nouns follow the three-tier pattern:

| Noun | Tier 1 (Presentation) | Tier 3 (Domain) |
|------|----------------------|-----------------|
| `StatusNoun` | validates args, formats output | `workspace_state()` |
| `TargetNoun` | validates args, formats output | `target_scan()` |
| `GitNoun` | validates args, formats output | `git_phase_close()` |

Each noun struct implements `NounCommand` (Tier 1 registration) and its verb impls
delegate to the corresponding Tier 3 function on the first meaningful line of `run()`.
No business logic accumulates in `run()`.

---

## Correct Pattern

```rust
// Tier 1: Presentation — validation and output only
struct MyNoun;
struct MyVerb;

impl NounCommand for MyNoun {
    fn name(&self) -> &'static str { "my-noun" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> { vec![Box::new(MyVerb)] }
}

impl VerbCommand for MyVerb {
    fn name(&self) -> &'static str { "do" }

    fn run(&self, args: &VerbArgs) -> anyhow::Result<()> {
        // delegate immediately — no logic here
        let result = domain::my_logic(args.get("flag"))?;
        println!("{}", result);
        Ok(())
    }
}

// Tier 3: Domain Logic — no CLI imports allowed
mod domain {
    pub fn my_logic(flag: Option<&str>) -> anyhow::Result<MyResult> {
        // all computation lives here
        // independently testable: #[test] fn test_my_logic() { ... }
        todo!()
    }
}
```

The rule: `run()` must delegate to a Tier 3 function within its first few lines.
If `run()` is growing, the growth belongs in a domain function.

---

## Wrong Pattern

```rust
// WRONG: all logic in run() — not testable, not composable
impl VerbCommand for MyVerb {
    fn run(&self, _args: &VerbArgs) -> anyhow::Result<()> {
        let output = std::process::Command::new("git")
            .arg("status")
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        // 50 more lines of logic here...
        // parsing, decisions, file writes — all unreachable by tests
        Ok(())
    }
}
```

This pattern makes the logic impossible to test without invoking the binary,
impossible to reuse from another surface, and impossible to inspect without
reading the full method body.

---

## Process Data Layer

`cicd.toml` is the persistent output of Tier 3 domain functions.

Domain functions read from and write to `cicd.toml` as structured state — not as
side effects of CLI execution. This means:

- `workspace_state()` derives its result from `cicd.toml` plus the live workspace.
- `git_phase_close()` writes phase records to `cicd.toml` as a pure state transition.
- `target_scan()` populates the `[targets]` table in `cicd.toml` from cargo metadata.

`cicd.toml` is not a log file and not a cache. It is the authoritative record of
process state as computed by Tier 3. Any tool that needs to read process state
reads `cicd.toml`; it does not re-derive state by re-running CLI commands.
