# External Operator Side Effects

`EXTERNAL_OPERATOR_SIDE_EFFECT` is a standing status (and
`external_operator_side_effects` an artifact field) for actions that require
a human operator holding external credentials or making an external, often
irreversible, decision. These are never blockers on an artifact's own
standing — they are typed, packaged, and dry-run-verified locally, then
handed off to a human. A workspace's own release-status ledger is the
source of truth for the current state of each side effect below; this
document is the reusable checklist template, not a duplicate status
tracker.

## 1. Package-registry publish (e.g. `cargo publish`, `npm publish`)

**Typical status**: local packaging dry-run-verified (`cargo publish
--dry-run` or equivalent → exit 0, with the raw output recorded as
evidence). Real publish not yet performed — the artifact is ready for
everything except the actual external publish step.

**Operator checklist**:

- [ ] Authenticate with the registry (requires an API token or credential —
      an operator credential, never entered by an agent)
- [ ] Decide whether to bump the package version — a one-line change, the
      operator's call
- [ ] Run the real publish command
- [ ] After publish, update the artifact's `external_operator_side_effects`
      entry in the next standing refresh pass to record completion (do not
      hand-edit the standing index — it is compiled, not asserted)

## 2. Paper or article submission (e.g. arXiv)

**Typical status**: ready for everything except the actual external
submission step. Artifact bundle built and referenced from the workspace's
release-status ledger.

**Operator checklist**:

- [ ] Make the artifact bundle / repository public if the submission
      requires it (see item 3 below if this means flipping repository
      visibility)
- [ ] Upload the submission bundle to the target venue
- [ ] Record required category/classification metadata per the venue's
      submission form
- [ ] After submission, record the resulting identifier as an `artifact`
      evidence entry on the paper's standing artifact at the next refresh

## 3. Repository visibility change

Changing a repository from private to public (a prerequisite for some
publish/submission flows) is an access-control change and falls under the
same operator-only category as the other two — it is a prohibited-for-agents
action ("modifying access controls or sharing permissions on any resource")
independent of the standing schema.

**Operator checklist**:

- [ ] Confirm which repository/bundle needs public visibility (often a
      packaged artifact bundle, not necessarily the full private working
      repo)
- [ ] Review the bundle contents for anything that should stay private
      before flipping visibility (credentials, unrelated work-in-progress,
      personal data)
- [ ] Perform the visibility change directly in the hosting provider's UI —
      no agent action substitutes for this
- [ ] Record the change as a dated line in the workspace's release-status
      ledger, not as a new standing status (visibility is not itself part of
      `cicd-standing.v1`)

## Why these are typed, not blockers

Rather than a claim silently failing or a doc describing progress as
"blocked," each external action gets its own typed status
(`EXTERNAL_OPERATOR_SIDE_EFFECT`) and a checklist. A standing refresh and
the gates it feeds never wait on operator action to report a clean local
state — they report `PUBLISH_READY` / `PUBLICATION_READY` (rung 7,
off-ladder) as already earned, and list the remaining external action as a
side-effect field, not a red gate.
