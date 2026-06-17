---
description: Inspect build artifacts with target show, preview what target prune would free, and explain the --apply flag.
allowed-tools: Bash, Read
---

You are helping the user understand and safely reclaim disk space from the Rust build cache. Work through the steps below in order.

---

## Step 1 — Show current target inventory

Run the target noun's show verb to get a structured view of what lives in the target directory:

```
cargo cicd target show
```

This command reads `TargetState` from the engine and prints a breakdown by artifact kind: debug builds, release builds, incremental cache, test runners, proc-macro artifacts, documentation, and any stale dependency objects. Capture the full output.

Note specifically:
- Total disk usage of `target/`
- Any artifacts flagged as stale (older than the configured retention window in `cicd.toml [target]`)
- Whether `target/cargo-cicd/evidence/` is included in the inventory

---

## Step 2 — Dry-run prune

Run the prune verb in dry-run mode (the default — no `--apply` flag means nothing is deleted):

```
cargo cicd target prune
```

This prints every path that *would* be removed along with its size. Capture and display the full list, grouped by artifact kind.

Pay attention to:
- The total reclaimable bytes shown at the end of the dry-run summary
- Any items the prune heuristic has marked as "safe to remove" vs. "retained"

---

## Step 3 — Explain what `--apply` would do

Based on the dry-run output, explain to the user:

1. **What would be freed** — total bytes and the breakdown by artifact category.
2. **What would be kept** — `target/release/` artifacts (including the `cargo-cicd` binary and any `.rlib` / `.a` files that are part of a publish-ready build) are never auto-deleted. Release artifacts are only removed when the user explicitly passes `--apply --include-release`.
3. **How to trigger the actual prune** — when the user is satisfied with the dry-run output, run:

   ```
   cargo cicd target prune --apply
   ```

   This performs the deletion for real. It cannot be undone. Confirm the user wants to proceed before suggesting this command.

---

## Step 4 — Evidence directory note

The directory `target/cargo-cicd/evidence/` holds XES process-event files and JSON receipts emitted by cargo-cicd commands. The prune command never touches this directory, even with `--apply`, because it is part of the process-data audit trail. If the user needs to clear evidence files, they must do so manually after reviewing the receipts with `cargo cicd evidence doctor`.

---

## Step 5 — Policy context

If autonomic policies are enabled (`[autonomic]` section in `cicd.toml`), `cargo cicd target prune` operates in `suggest` mode by default — it prints recommendations without taking action. The `--apply` flag overrides suggest mode for this single invocation only. The policy configuration can be inspected with:

```
cargo cicd status show
```

Look for the `PolicyState` section in the output to confirm the active autonomic mode.
