# /status — cargo-cicd Workspace Health Snapshot

Trigger: user says "status", "health check", or runs `/status`.
Collect ALL output before reporting. Do not stop on warnings.

## Execution sequence

```bash
cargo cicd status show        # step 1
cargo cicd git status         # step 2
cargo cicd workspace doctor   # step 3
cargo cicd target show        # step 4
```

If target exceeds 500 MB: `cargo cicd target prune --dry-run`
If target exceeds 2 GB: flag as FAIL; run `cargo cicd target prune --confirm` only with explicit user approval.

## Policy table (from `workspace doctor` output)

| Policy | Checks | WARN action |
|--------|--------|-------------|
| `target_pressure` | target dir above threshold | `cargo cicd target prune` |
| `toolchain_mismatch` | rustc vs lockfile | `rustup update` or `rustup override set <ver>` |
| `trybuild_changed` | fixtures changed, not run | `cargo cicd trybuild changed` |
| `branch_behind` | behind origin/main | `git pull --rebase origin main` |
| `evidence_stale` | last emission too old | `cargo cicd evidence doctor` |
| `publish_not_adjudicated` | publish ran, no wpm verdict | `wpm audit target/cargo-cicd/evidence/evt-*.ocel.json` |
| `git_phase_dirty` | uncommitted changes | `git add && git commit` or `git stash` |

## Output format

```
WORKSPACE HEALTH SNAPSHOT — <date>
===================================
Dimension         | Verdict | Detail
------------------|---------|-------
Git status        | PASS    | Branch: main, clean, 0 ahead/behind
Git phase         | PASS    | Phase: clean
Workspace doctor  | WARN    | 1 crate missing license field
Target directory  | PASS    | 312 MB
Evidence          | PASS    | Last emission: <timestamp>

POLICY RECOMMENDATIONS
[ ] <policy>: <fix command>

OVERALL: WARN (1 issue)
```

Verdict rules: PASS = all green. WARN = non-blocking issue. FAIL = any exit non-zero or blocking policy.

For every WARN/FAIL: emit the exact fix command.
