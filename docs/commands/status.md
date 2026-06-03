# cargo cicd status

Show workspace health at a glance: toolchain, target directory size, and git state.

## Usage

```bash
cargo cicd status
# or equivalently
cargo cicd status show
```

## What it shows

| Field | Meaning |
|-------|---------|
| `toolchain` | Active Rust toolchain string (e.g. `stable-aarch64-apple-darwin`) |
| `target` | Total size of the `target/` directory and a pass/warn/fail verdict |
| `branch` | Current git branch name |
| `dirty files` | Count of modified-but-unstaged files |
| `untracked` | Count of files git does not track |
| `git` | Overall tree cleanliness: `clean` or `dirty` |

Verdict thresholds for `target`:

| Verdict | Condition |
|---------|-----------|
| `pass` | Size is below 70% of the configured maximum (default 20 GB) |
| `warn` | Size is between 70% and 100% of the maximum |
| `fail` | Size meets or exceeds the maximum |

## When to use it

- At the start of a work session, to confirm the workspace is in a known state before making changes.
- Before pushing to CI, to verify the tree is clean and the target directory is not bloated.

## Example output

```bash
$ cargo cicd status
cargo-cicd workspace status
===========================
toolchain:    stable-aarch64-apple-darwin
target:       4.31 GB [pass]
branch:       main
dirty files:  0
untracked:    2
git:          dirty
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Always exits 0; the command is informational |

## Three-tier architecture

Understanding how `status` is structured explains why it behaves predictably and is straightforward to test in isolation.

### Tier 1 — Presentation (`StatusNoun`, `StatusShowVerb`)

`StatusNoun` implements `NounCommand`. Its only job is to declare the noun name (`"status"`), a description, and the list of verbs it accepts. It owns no data and contains no logic.

`StatusShowVerb` implements `VerbCommand`. Its `run` method receives parsed `VerbArgs` and immediately calls adapters, then formats the result for the terminal.

**Why this matters:** The presentation layer can be replaced or extended (a JSON output mode, a web endpoint) without touching anything below it. It can also be tested by calling the verb's `run` method with synthetic args.

### Tier 2 — Integration (adapter wiring in `VerbCommand::run`)

`StatusShowVerb::run` wires three adapters together:

- `ToolchainDetector::active_toolchain()` — reads the active Rust toolchain string.
- `TargetScannerAdapter::total_size_gb("target")` and `verdict(size_gb, 20.0)` — walks the target directory and computes a size verdict.
- `GitStatusAdapter::query()` — shells out to `git status --porcelain` and parses the result into a typed `GitStatusResult`.

This layer knows which adapters exist and how to combine them. It does not contain business rules.

### Tier 3 — Domain logic (pure adapter functions)

`TargetScannerAdapter::total_size_bytes` walks the directory tree using `walkdir` and sums file sizes. It takes a path string and returns a `u64`. No `println!`, no `process::exit`, no global state.

`TargetScannerAdapter::verdict` takes two `f64` values and returns `"pass"`, `"warn"`, or `"fail"`. It is a pure function: the same inputs always produce the same output.

`GitStatusAdapter::query` shells out to git and returns a `GitStatusResult` struct with categorized file lists. The caller decides what to display.

**Why this matters:** Pure domain functions are testable without a terminal, without a running process, and without a real filesystem. You can call `verdict(15.0, 20.0)` in a unit test and assert `"warn"` without invoking the CLI at all. The domain logic never breaks because of a CLI refactor.

## Related commands

- `cargo cicd target show` — detailed target directory breakdown
- `cargo cicd git status` — detailed git state
