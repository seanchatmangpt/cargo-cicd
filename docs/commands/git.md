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
- Recommended next action

### close

```bash
cargo cicd git close
```

Enforces phase closure. Exits non-zero if the working tree is dirty.

Use this before declaring a phase complete — it prevents closing a phase while uncommitted work remains. It does not stash or hide files; it only checks and reports.

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
branch     main
dirty      0
staged     0
untracked  2
action     commit or stash untracked files before CI

$ cargo cicd git close
error: working tree is dirty — phase closure refused
  2 untracked files
  run: git status
```

## Notes

- `git close` does not commit, stash, or modify any files.
- The `git_phase_dirty` autonomic policy surfaces the same signal in suggest mode.
- For a full git summary alongside workspace health, use `cargo cicd status`.
