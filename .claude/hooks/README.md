# cargo-cicd Claude Code Hooks

This directory contains shell hooks invoked by Claude Code at various lifecycle
events, plus one manual convenience script.

---

## Hooks

### `session-start.sh` — SessionStart event

Wired in `.claude/settings.json` under the `SessionStart` event with
`"matcher": ".*"` (runs unconditionally at the start of every Claude Code session).

Prints a concise, friendly project-readiness summary to stdout:

- Project name and one-line description
- Detected Rust toolchain version (`rustc --version`)
- Key build/check/test commands
- Full noun-verb table for the `cargo cicd` CLI
- Commit-message format reminder

**Performance:** no builds or network calls — exits in milliseconds.  
**Exit code:** always 0 (never blocks session start).

---

### `post-tool-use.sh` — PostToolUse event

Wired in `.claude/settings.json` under the `PostToolUse` event with
matcher `Bash`.

After every Bash tool call, this hook:

1. Reads the tool JSON from stdin and extracts the `command` field.
2. Checks whether the command matches `cargo (build|make|test)`.
3. If it matches, scans `target/cargo-cicd/evidence/` for evidence
   files (`*.xes`, `*.jsonl`).
4. If any evidence files are found, prints an `INFO` reminder to stderr
   suggesting `cargo cicd evidence doctor` be run if the evidence gate
   is relevant.

**This hook is advisory only.** It always exits 0 — Claude is never
blocked. The message nudges you to adjudicate evidence after build/test
runs that may have emitted XES/JSONL files.

---

### `pre-tool-use.sh` — PreToolUse event

Wired in `.claude/settings.json` under the `PreToolUse` event with
matcher `Bash`.

Before every Bash tool call, this hook reads the tool JSON from the
`CLAUDE_TOOL_INPUT` environment variable and inspects the command string
for patterns that could cause irreversible harm:

1. **Destructive git operations** — `git reset --hard`, `git push --force`,
   `git push -f`, `git clean -f`, `git checkout --`, `git branch -D`.
   Prints a warning to stderr advising safer alternatives (stash, backup
   branch, dry-run).
2. **`rm -rf`** — warns to verify the path does not include source files,
   evidence, or receipts.
3. **`cargo publish`** — warns that the evidence gate (`wpm audit`) and
   invariants must be green before publishing to crates.io.

**Exit-code semantics:**

| Exit code | Effect |
|-----------|--------|
| `0` | Allows the Bash tool call to proceed (including after printing warnings). |
| `2` | Blocks the tool call entirely — Claude Code will not execute the command. |

This hook currently always exits `0` (warn-only). Change the exit code to
`2` on a specific check branch to hard-block a class of commands.

---

## Manual convenience scripts

### `public-boundary-guard.sh` — NOT auto-wired

Checks public-boundary terms in edited files. Run directly when you
want to scan a file for forbidden terms before committing:

```sh
bash .claude/hooks/public-boundary-guard.sh
```

Checks whether the file is on the **public surface**:
- `src/**` (all Rust source)
- `README.md`
- `docs/**`

Skips `CLAUDE.md` and anything under `.claude/` (internal files).
Greps for forbidden public-boundary terms and prints a `WARNING` to
stderr naming the term and file if any are found.

**This script is not registered** in `settings.json` and is never
invoked automatically. It is warn-only and always exits 0.

---

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

The auto-wired hooks are registered in `.claude/settings.json` like this:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/pre-tool-use.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use.sh"
          }
        ]
      }
    ]
  }
}
```
