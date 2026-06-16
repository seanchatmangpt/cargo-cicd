---
description: Run cargo cicd status and workspace doctor, then summarise workspace health.
allowed-tools: Bash(cargo cicd status), Bash(cargo cicd workspace doctor), Read
---

Run the following two commands in order and collect their output:

1. `cargo cicd status`
2. `cargo cicd workspace doctor`

Then produce a concise workspace health summary with these sections:

**Workspace Status**
- Report the overall status verdict from `cargo cicd status` (clean / dirty / stale).
- List any changed files or uncommitted work detected.
- Note the active toolchain and any toolchain mismatches.

**Doctor Findings**
- List every issue flagged by `cargo cicd workspace doctor`, one bullet per finding.
- Distinguish warnings (yellow) from errors (red) if the output indicates severity.
- If no issues were found, say "No issues detected — workspace is push-ready."

**Recommended Next Steps**
- For each error or warning, suggest the specific `cargo cicd` command that resolves it (e.g. `cargo cicd git close`, `cargo cicd target prune`, `cargo cicd test changed`).
- If the workspace is clean, confirm it is ready to publish or push.

Keep the summary under 30 lines. Use plain text — no emojis.
