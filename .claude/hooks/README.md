# cargo-cicd Hooks

## Auto-wired hooks (`settings.json`)

### `session-start.sh` — trigger: `SessionStart`, matcher `.*`
Prints project-readiness summary: toolchain version, build commands, noun-verb table, commit format.
No builds, no network. Always exits 0.

### `pre-tool-use.sh` — trigger: `PreToolUse`, matcher `Bash`
Reads command from `CLAUDE_TOOL_INPUT`. Warns on:
- `git reset --hard`, `git push --force`, `git push -f`, `git clean -f`, `git checkout --`, `git branch -D`
- `rm -rf`
- `cargo publish` (requires evidence gate + invariants green first)

| Exit | Effect |
|------|--------|
| `0` | Proceed (warn-only, current default) |
| `2` | Block tool call entirely |

To hard-block a command class: change exit code to `2` on that branch.

### `post-tool-use.sh` — trigger: `PostToolUse`, matcher `Bash`
After `cargo (build|make|test)`: scans `target/cargo-cicd/evidence/` for `*.xes`, `*.jsonl`.
If found: prints `INFO` to stderr suggesting `cargo cicd evidence doctor`.
Advisory only. Always exits 0.

## Manual scripts (not in `settings.json`)

### `public-boundary-guard.sh`
```sh
bash .claude/hooks/public-boundary-guard.sh
```
Scans public surface (`src/**`, `README.md`, `docs/**`) for forbidden terms. Skips `CLAUDE.md`, `.claude/`.
Warn-only, exits 0.

### `cargo-check.sh`
```sh
bash .claude/hooks/cargo-check.sh
```
Runs `cargo fmt --all -- --check` then `cargo check`. No file modifications.

## `settings.json` wiring

```json
{
  "hooks": {
    "SessionStart": [{"matcher": ".*", "hooks": [{"type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh"}]}],
    "PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/pre-tool-use.sh"}]}],
    "PostToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use.sh"}]}]
  }
}
```
