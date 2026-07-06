# cargo cicd git status / close

Git phase management commands.

## Commands

### status

```bash
cargo cicd git status
```

Shows:

- Current branch name
- Dirty file count (modified, not staged)
- Staged file count
- Untracked file count
- Ahead/behind counts relative to the tracking branch
- Per-file listing of dirty and untracked files
- Recommended next action

### close

```bash
cargo cicd git close
```

Enforces phase closure. Exits non-zero if the working tree is dirty.

Use this before declaring a phase complete — it prevents closing a phase while uncommitted work remains. It does not stash, commit, or hide files; it only checks and reports. If the tree is clean, it records a `PASS` event and exits 0. If the tree is dirty, it records a `FAIL` event and exits non-zero with a message explaining which files need attention.

## Exit codes

### git status

| Code | Meaning |
|------|---------|
| 0 | Always exits 0; status is informational |

### git close

| Code | Meaning |
|------|---------|
| 0 | Tree is clean — phase closure allowed |
| 1 | Tree is dirty — phase closure refused |

## Example output

```bash
$ cargo cicd git status
git status
==========
branch:       main
staged:       0
dirty:        1
untracked:    2
ahead:        0
behind:       0

dirty files:
  M src/lib.rs
untracked:
  ? notes.txt
  ? scratch.rs
next: recommendation: run 'cargo cicd git close' to stage and commit

$ cargo cicd git close
git phase closure
=================
dirty files:   1
untracked:     2

phase closure requires a clean tree.
stage and commit your changes before closing the phase.

refusing to hide unrelated dirty files — no silent batch commit.
error: phase closure refused: tree is dirty. Stage and commit manually, then re-run.
```

## Notes

- `git close` does not commit, stash, or modify any files.
- The `git_phase_dirty` autonomic policy surfaces the same signal in suggest mode.
- For a full workspace summary alongside git state, use `cargo cicd status`.

## Three-tier architecture

The git commands follow the same three-tier structure as every other noun in this tool. The separation is worth understanding because it determines what is testable and what has side effects.

### Tier 1 — Presentation (`GitNoun`, `GitStatusVerb`, `GitCloseVerb`)

`GitNoun` implements `NounCommand`. It declares the noun name (`"git"`) and registers two verbs: `GitStatusVerb` and `GitCloseVerb`.

Each verb implements `VerbCommand`. The verb's `run` method receives parsed `VerbArgs`, calls the adapter, formats output to stdout, and emits an evidence event. The verb itself contains no parsing logic and no git subprocess calls — those belong to the tiers below.

**Why this matters:** The verbs can be driven by a test harness that passes synthetic `VerbArgs` without touching a real terminal. Adding a third git verb (e.g. `git push-check`) means writing one new struct. Nothing else changes.

### Tier 2 — Integration (adapter wiring in `VerbCommand::run`)

Both `GitStatusVerb::run` and `GitCloseVerb::run` call `GitStatusAdapter::query()` and interpret the returned `GitStatusResult` to decide what to print and what event verdict to emit.

`GitCloseVerb::run` also decides whether to return `Ok(())` or `Err(NounVerbError)` based on whether the tree is clean — this is the only place where the dirty/clean signal becomes a process exit code.

This layer knows which adapter to call and how to map its result to CLI behavior. It does not contain git subprocess logic.

### Tier 3 — Domain logic (`GitStatusAdapter`)

`GitStatusAdapter::query()` is a near-pure function: it shells out to `git rev-parse --abbrev-ref HEAD` and `git status --porcelain`, parses the output into a `GitStatusResult` struct, and returns it. It does not print anything, does not exit, and does not modify any state outside of running those two read-only git subprocesses.

`GitStatusResult` is a plain data struct:

```rust
pub struct GitStatusResult {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}
```

**Why this matters:** `GitStatusAdapter::query()` can be called in a unit test pointed at any git repository. The test can assert on the returned struct without capturing stdout. The logic that computes "is the tree dirty?" lives entirely in the adapter and is independent of how the result is displayed or what exit code it produces. A change to the output format in the verb does not risk breaking the dirty-detection logic, and a change to dirty-detection logic does not risk breaking the output format.
