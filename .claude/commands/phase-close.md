---
description: Run git status then git close, explaining the clean-tree requirement and refusal to hide dirty files.
allowed-tools: Bash, Read
---

You are helping the user close the current git phase in cargo-cicd. Phase closure is a deliberate, audited operation — it requires an honest view of the working tree and refuses to proceed if unrelated dirty files would be hidden. Work through the steps below in order.

---

## Step 1 — Inspect current git state

Run the git noun's status verb:

```
cargo cicd git status
```

This reads `GitPhaseState` from the engine and reports:
- The current branch and HEAD commit
- Staged files (ready to commit)
- Modified but unstaged files
- Untracked files
- Whether the working tree is clean

Capture the full output and display it.

---

## Step 2 — Understand what "phase close" means

Before proceeding, explain the concept to the user:

A **phase close** in cargo-cicd marks the boundary between two units of work. It is recorded as a `ProcessEvent` in `target/cargo-cicd/evidence/` and written as a structured entry in `cicd.toml`. The close event captures:
- The branch name and HEAD SHA at the moment of closure
- The list of files changed during this phase
- A timestamp

Phase closure is **not** a substitute for `git commit`. It is an audit-trail marker that sits on top of the normal git workflow.

---

## Step 3 — Pre-flight check

Before running `git close`, examine the output from Step 1:

**If the tree is clean (no modified, staged, or untracked files):**
Proceed to Step 4.

**If the tree is dirty:**
Explain the refusal logic: `cargo cicd git close` will refuse to close a phase when there are uncommitted modifications to files that are *not* part of the current phase's change set. This is intentional — closing over unrelated dirty files would corrupt the audit trail by attributing changes to the wrong phase.

Tell the user what they need to do:
1. Commit or stash all unrelated changes first.
2. If the dirty files *are* part of this phase, stage and commit them (`git add <file> && git commit -m "feat(...): ..."`) and then re-run `/phase-close`.
3. Only proceed with `cargo cicd git close` once `cargo cicd git status` reports a clean tree.

---

## Step 4 — Close the phase

Once the tree is clean, run:

```
cargo cicd git close
```

This command:
1. Writes a closure record to `cicd.toml` under `[[events]]`.
2. Emits an XES process event to `target/cargo-cicd/evidence/`.
3. Confirms the closed phase by printing the HEAD SHA and timestamp.

Capture and display the full output.

---

## Step 5 — Verify closure

Confirm closure succeeded by re-running git status:

```
cargo cicd git status
```

The output should now show the phase as closed. Also verify that a new `.xes` event file was written to `target/cargo-cicd/evidence/` and that `cicd.toml` contains the new `[[events]]` entry.

---

## Step 6 — Commit message reminder

Phase closure does not create a git commit automatically. If the user still needs to commit the `cicd.toml` update (which now contains the closure event), remind them to use the correct format:

```
feat(git): close phase <short-description>
```

For example:
```
feat(git): close phase add-target-prune-verb
```
