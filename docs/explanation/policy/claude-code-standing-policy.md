# Claude Code Standing-Consumption Policy

This is the policy an agent (Claude Code or otherwise) should follow in any
workspace using cargo-cicd's standing compiler, before claiming any artifact
is real, tested, or ready.

## Before claiming anything

1. Read the workspace's compiled `cicd-standing.v1` index (schema in
   `docs/reference/standing-schema.md`) and its rendered human summary, if
   one has been generated (a REALITY_INDEX.md-style report).
2. If either is missing, or the index's generation timestamp looks stale
   (see the stale-index rule in `claim-rules.md`, a sensible default
   threshold is 24h), **run `cargo cicd standing refresh` first**. Do not
   reason from memory of a prior run.
3. Only then state what an artifact's standing actually is — quote the
   `standing` list and `ladder_level` for the specific artifact `id`, not a
   release-wide impression.

## Rules

- **Never trust prior-agent summaries, README claims, or code comments over
  the standing index.** A comment saying "fully tested" or a prior
  transcript saying "this is production-ready" is not evidence; the index
  is. If they disagree, the index wins and the doc/comment is out of date.
- **Never say "production-ready" (or pilot-ready/publish-ready/publication-ready)
  unscoped.** Every one of those four claims requires a stated scope — see
  the scoped-readiness rule in `docs/reference/standing-schema.md` and rule 2
  in `claim-rules.md`. "Production-ready" with no named scope is not a
  claim, it is a category error.
- **External actions are side effects, not blockers.** `cargo publish`,
  paper submission, repository-visibility changes — see
  `external-operator-side-effects.md`. An artifact can be fully
  `PUBLISH_READY` while the real publish is still pending operator action;
  do not describe that as "blocked" or "not ready."
- **If evidence is absent, run the gate — don't assert.** If a claim would
  need `TESTED`/`RECEIPTED`/`OCEL_PROVEN`/etc. and the index does not show
  it for that artifact, the correct action is to run the underlying
  command/test/OCEL pass and refresh the index, not to write the claim and
  hope the index catches up later.
- **State findings, not verdicts.** Report exactly what the standing index
  says for the artifact in question — statuses, ladder level, scope,
  evidence pointers — rather than a summarizing adjective.
- **A prototype, mock, or simulation stays unpromoted.** Nothing described
  as a prototype, mock, or simulation may be claimed as any status above
  `DISCOVERED` until it is backed by the same evidence any other artifact
  would need (build, test, receipt, OCEL) — a demo script producing
  plausible-looking output is not evidence.
- **A dashboard, summary doc, or generated report is a rendering of the
  index, not a second source of truth.** If a rendered report and the index
  it was generated from disagree, the index is stale or the report was
  hand-edited; regenerate rather than trusting either in isolation.

## Quick reference

| Question | Where to look |
|---|---|
| Is artifact X built/tested? | standing index → artifact `X`'s `standing` list; ladder rungs 1-2 |
| Is X receipted and receipt-verified? | rung 3 (`RECEIPTED`); `RECEIPT_VERIFIED` is off-ladder but implies it |
| Is X OCEL/wasm4pm proven? | rungs 4-5 |
| Is X ready to publish/pilot/go to production? | rungs 7-9, **and** check `scope` is non-empty |
| What's left before the next rung? | `cargo cicd claude-context` — prints exactly this, per artifact |
| Is the whole picture summarized anywhere? | the workspace's generated reality-index report (do not hand-edit) |

## If refreshing the index itself fails

Report the exact failure (command, exit code, stderr) rather than falling
back to an old index or a guess. The same discipline applies to any agent
reasoning about readiness by hand.
