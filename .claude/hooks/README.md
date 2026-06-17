# cargo-cicd Claude Code Hooks

This directory contains shell hooks invoked by Claude Code at various lifecycle
events, plus one manual convenience script.

---

## Hooks

### `session-start.sh` — SessionStart event

Wired in `.claude/settings.json` under the `SessionStart` event with no
matcher (runs unconditionally at the start of every Claude Code session).

Prints a concise, friendly project-readiness summary to stdout:

- Project name and one-line description
- Detected Rust toolchain version (`rustc --version`)
- Key build/check/test commands
- Full noun-verb table for the `cargo cicd` CLI
- Commit-message format reminder

**Performance:** no builds or network calls — exits in milliseconds.  
**Exit code:** always 0 (never blocks session start).

---

### `public-boundary-guard.sh` — PostToolUse event

Wired in `.claude/settings.json` under the `PostToolUse` event with
matcher `Edit|Write|MultiEdit`.

After every file-edit tool call, this hook:

1. Reads the tool JSON from stdin and extracts the edited file path.
2. Checks whether the file is on the **public surface**:
   - `src/**` (all Rust source)
   - `README.md`
   - `docs/**`
3. Skips `CLAUDE.md` and anything under `.claude/` (internal files).
4. Greps the file for forbidden public-boundary terms and prints a
   `WARNING` to stderr naming the term and file if any are found.

**This hook is warn-only.** It always exits 0 — Claude is never blocked.
The warning is advisory: review and remove the offending term before
committing.

---

## Manual convenience script

### `cargo-check.sh` — NOT auto-wired

Run directly when you want a quick format + type-check pass:

```sh
bash .claude/hooks/cargo-check.sh
```

Steps:
1. `cargo fmt --all -- --check` — verifies formatting without modifying files.
2. `cargo check` — full type-check without emitting compiled artefacts.

This script is **not registered** in `settings.json` and is never invoked
automatically. It exists as a lightweight pre-commit sanity check you can
run on demand.

---

## settings.json wiring (reference)

The two auto-wired hooks are registered in `.claude/settings.json` like this:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/session-start.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/public-boundary-guard.sh"
          }
        ]
      }
    ]
  }
}
```
