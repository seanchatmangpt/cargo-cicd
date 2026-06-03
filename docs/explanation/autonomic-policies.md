<!-- BEGIN custom:full-doc -->
# Autonomic Policies

This document explains the four suggest-mode policies in cargo-cicd, what each
one monitors, and how to read the suggestions they produce.

## What suggest mode is

Suggest mode is a non-blocking advisory layer. When a policy detects a
condition worth attention, it emits a structured suggestion — a human-readable
recommendation with a machine-readable severity level and a suggested command.
Suggest mode never refuses a command and never modifies workspace state on its
own.

This design follows the principle of least surprise: the tool tells you what
it noticed and what you could do about it, then gets out of the way.

## The four policies

### 1. StaleTargetPolicy

**Monitors:** The age and size of the Cargo `target/` directory.

**Triggers when:**
- The oldest artefact exceeds `prune_older_than_days` (default: 14), or
- Total target size exceeds `max_size_gb` (if configured).

**Suggestion emitted:**

```
[suggest] StaleTargetPolicy: target directory is 3.8 GB with artefacts 22 days old.
  Consider: cargo cicd target prune
```

**How to read it:** This is informational. The workspace is not blocked. The
suggestion fires once per session when the threshold is crossed, not on every
command.

---

### 2. UncommittedEvidencePolicy

**Monitors:** Whether `cicd.toml` has been modified but not committed.

**Triggers when:** `cicd.toml` has uncommitted changes and more than one
lifecycle command has been run since the last commit.

**Suggestion emitted:**

```
[suggest] UncommittedEvidencePolicy: cicd.toml has 3 uncommitted events.
  Consider: git add cicd.toml && git commit -m "chore(cicd): record events"
```

**How to read it:** Evidence records have the most value when they are
committed alongside the code they describe. This policy nudges you to commit
`cicd.toml` regularly so the event history stays co-located with the code
history.

---

### 3. DivergentBranchPolicy

**Monitors:** How far the current branch has diverged from trunk.

**Triggers when:** The current branch is more than a configurable number of
commits ahead of trunk (default: 10) without a recent `git close`.

**Suggestion emitted:**

```
[suggest] DivergentBranchPolicy: feat/my-feature is 14 commits ahead of main.
  Consider: cargo cicd git close
```

**How to read it:** Long-lived branches accumulate merge risk. This policy
does not know whether your branch is intentionally long-lived. If it is,
configure a higher threshold or suppress the policy for that branch.

In `cicd.toml`:

```toml
[policy.divergent_branch]
max_commits_ahead = 20          # raise the threshold
suppress_for_branches = ["long-lived-feature"]
```

---

### 4. PublishReadinessPolicy

**Monitors:** Whether workspace members have been in a validated state for an
extended period without being published.

**Triggers when:** One or more members have been publish-ready for longer than
`publish_readiness_stale_days` (default: 7).

**Suggestion emitted:**

```
[suggest] PublishReadinessPolicy: my-api has been publish-ready for 9 days.
  Consider: cargo cicd publish run
```

**How to read it:** This policy exists to prevent validated work from
accumulating indefinitely. If you are intentionally holding a release, the
suggestion is noise — suppress it for the affected crate:

```toml
[policy.publish_readiness]
suppress_for_crates = ["my-api"]
```

## How to read suggestion output

All four policies use the same output format:

```
[suggest] <PolicyName>: <human-readable condition description>.
  Consider: <suggested command>
```

- `[suggest]` prefix — always present; distinguishes suggestions from errors
  and warnings.
- `<PolicyName>` — identifies which policy fired, so you can tune or suppress
  it by name.
- `Consider:` — the suggested command is always a valid cargo-cicd command
  you can copy and run directly.

Suggestions appear at the end of any command's output, after the command's
own result. They do not affect the exit code.

## Suppressing policies globally

To disable a policy entirely:

```toml
[policy]
disabled = ["StaleTargetPolicy", "DivergentBranchPolicy"]
```

Disabled policies emit nothing. They do not appear in output and do not
record events.
<!-- END custom:full-doc -->
