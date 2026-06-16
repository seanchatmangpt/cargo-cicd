# /status — cargo-cicd Workspace Health Snapshot

Run a complete workspace health check across all dimensions. Each step feeds into the final summary. Collect all output before reporting — do not stop on warnings.

---

## Step 1 — Workspace status snapshot

```bash
cargo cicd status show
```

**What this reports:**
- Git branch, dirty/staged/untracked file counts
- Ahead/behind counts relative to origin
- Active Rust toolchain and edition
- Workspace name and root path

**Capture:** Note the exit code and whether it printed PASS or WARN. A non-zero exit code here is a red flag for the overall health.

---

## Step 2 — Git phase

```bash
cargo cicd git status
```

**What this reports:**
- Current git phase (clean, dirty, staged, committed, pushed)
- Specific file counts for dirty, staged, and untracked buckets
- Whether the branch is ahead or behind origin

**Capture:** Record the phase label and any file counts above zero.

---

## Step 3 — Workspace diagnostics

```bash
cargo cicd workspace doctor
```

**What this reports:**
- Workspace member crates and their manifest validity
- Any missing required metadata fields (description, license, readme, etc.)
- Autonomic policy recommendations (if `--features autonomic` was used at build time)
- Policy verdicts: Pass, Warn, or Skip for each active policy

**Capture:** List every WARN or policy recommendation emitted.

---

## Step 4 — Target directory size

```bash
cargo cicd target show
```

**What this reports:**
- Absolute path to the target directory
- Total disk usage in bytes (and human-readable form if available)
- Comparison against any configured threshold

**Capture:** Record the total size. If it exceeds 500 MB, flag it as a WARN dimension. If it exceeds 2 GB, flag as FAIL.

**If size is large:**
```bash
cargo cicd target prune --dry-run
```
This shows what would be removed without deleting anything. To actually prune:
```bash
cargo cicd target prune --confirm
```

---

## Step 5 — Policy recommendations (autonomic layer)

The `workspace doctor` output from Step 3 already includes policy output, but call it out explicitly here. Review each policy entry:

| Policy | What it checks | Action if WARN |
|--------|---------------|----------------|
| `target_pressure` | Target dir above threshold | Run `cargo cicd target prune` |
| `toolchain_mismatch` | rustc version vs lockfile | Update toolchain or re-lock |
| `trybuild_changed` | Trybuild fixtures changed but not run | Run `cargo cicd trybuild changed` |
| `branch_behind` | Local branch behind origin/main | Pull or rebase |
| `evidence_stale` | Last evidence emission too old | Re-run `cargo cicd evidence doctor` |
| `publish_not_adjudicated` | Publish ran but no wpm verdict | Require `wpm audit` before release |
| `git_phase_dirty` | Uncommitted changes present | Commit or stash |

List each policy that emitted a WARN and its specific recommendation.

---

## Step 6 — Final summary report

After collecting output from all steps, produce a structured health report:

```
WORKSPACE HEALTH SNAPSHOT — <date>
===================================

Dimension         | Verdict | Detail
------------------|---------|-------
Git status        | PASS    | Branch: main, clean, 0 ahead/behind
Git phase         | PASS    | Phase: clean
Workspace doctor  | WARN    | 1 crate missing license field
Target directory  | PASS    | 312 MB (below threshold)
Evidence          | PASS    | Last emission: <timestamp>

POLICY RECOMMENDATIONS
----------------------
[ ] <policy_name>: <recommendation text>

OVERALL: WARN (1 issue requires attention)
```

**Verdict rules:**
- **PASS** — All dimensions green, no policy warnings
- **WARN** — One or more dimensions have non-blocking issues or policy recommendations
- **FAIL** — Any dimension exits non-zero, or a blocking policy is active

**If overall is WARN or FAIL:** List each issue with its fix command so the user can act immediately.

---

## Quick reference: fix commands

| Issue | Fix |
|-------|-----|
| Dirty files | `git add <files> && git commit -m "..."` or `git stash` |
| Branch behind | `git pull --rebase origin main` |
| Target too large | `cargo cicd target prune --confirm` |
| Stale evidence | `cargo cicd evidence doctor` |
| Toolchain mismatch | `rustup update` or `rustup override set <version>` |
| Trybuild fixtures changed | `cargo cicd trybuild changed` |
| Missing manifest fields | Edit `Cargo.toml` and add `license`, `description`, `readme` as needed |
| Publish not adjudicated | `wpm audit target/cargo-cicd/evidence/evt-*.xes` |
