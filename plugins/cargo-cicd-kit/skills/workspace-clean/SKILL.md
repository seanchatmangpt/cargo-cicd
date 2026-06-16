---
name: workspace-clean
description: Safely prunes a bloated cargo-cicd workspace target directory. Use this skill when the user asks to clean, shrink, or prune the target directory, or when cargo cicd target show reports a large artifact cache. Dry-run by default; pass --apply to execute the prune.
---

# workspace-clean

Safely shrink the `target/` directory in a cargo-cicd workspace using the `cargo cicd target` noun.

## Steps

### 1. Inspect the target directory

Run:
```
cargo cicd target show
```

Report the output: total size, per-profile breakdown (debug / release / doc), and any stale artifact warnings. If `target/` is under 500 MB, inform the user and ask whether to proceed.

### 2. Determine the run mode

Check whether the user passed `--apply` as an argument to the skill invocation.

- **Without `--apply` (default — dry run):** Proceed to step 3 with dry-run flag.
- **With `--apply`:** Proceed to step 3 with the apply flag and warn the user that artifacts will be deleted.

### 3. Run the prune

**Dry run (default):**
```
cargo cicd target prune
```
Show the list of artifacts that *would* be removed and their combined size. Do NOT delete anything. Tell the user to re-run with `--apply` to execute.

**Apply mode:**
```
cargo cicd target prune --apply
```
Report which artifacts were removed and how much space was reclaimed.

### 4. Safety rules — never auto-delete

- Release artifacts (anything under `target/release/`) are **never** automatically deleted, even in apply mode. The `prune` command respects this boundary; confirm it in the output.
- If the output contains the word "release" in a deletion list, stop and alert the user before proceeding.
- Do not delete `target/cargo-cicd/evidence/` — process evidence files are needed for receipt verification.

### 5. Confirm

After a successful apply, re-run `cargo cicd target show` and report the new total size so the user can see the space reclaimed.
