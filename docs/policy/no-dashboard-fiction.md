# No Dashboard Fiction

A dashboard, badge, README status line, or agent-generated summary is a
*rendering* of standing evidence, not a source of it. This document states
the rule plainly and describes how an external linter/diagnostic consumer
(such as anti-llm-cheat-lsp) can enforce it mechanically.

## The rule

- **A badge or dashboard tile must point at a specific standing artifact and
  status**, traceable back to the compiled `cicd-standing.v1` index. A green
  badge with no backing index entry is fiction, not standing.
- **A README claim ("fully tested", "production-ready", "shipped") is not
  itself standing** — it is prose that must be checked against the index the
  same way any other claim is (see `claim-rules.md`). An agent's own summary
  of its work carries no more weight than that prose.
- **Mock, prototype, and simulation artifacts stay unpromoted.** Anything
  explicitly labeled (in code, docs, or its own generation metadata) as a
  mock, prototype, stub, or simulation may not carry any standing status
  above the baseline "discovered" rung until it earns the same evidence
  (build, test, receipt, OCEL) any other artifact would need. A convincing
  demo is not evidence.
- **Published differs from dry-run.** A dry-run-verified publish
  (`PUBLISH_READY`) and a completed real publish
  (`EXTERNAL_OPERATOR_SIDE_EFFECT` recorded) are different statuses — see
  `external-operator-side-effects.md`. Rendering both as "published" in a
  dashboard erases a real distinction.
- **A stale index renders a stale dashboard.** If the underlying standing
  index has not been refreshed within its freshness window, any dashboard
  or summary built from it should be treated as unreliable until refreshed
  — not patched by hand.

## Enforcement as diagnostics

A consumer such as anti-llm-cheat-lsp can turn each bullet above into a
mechanical diagnostic that scans source, docs, and prose for:

- unsupported readiness claims (unscoped "production-ready" language, or a
  named status that does not appear in the index for that artifact)
- "published" language without a corresponding external-operator
  side-effect record
- claims of being live, running, or verified without receipt or OCEL
  evidence backing them
- a missing, unparseable, or stale standing index when a claim depends on
  one
- badges/dashboards with no traceable index entry

This document states the intent; the specific diagnostic identifiers, CLI
invocation, and configuration for any given linter live in that linter's own
repository and docs — cargo-cicd does not duplicate that surface here, only
the policy it should enforce.

## Why

The whole point of a standing index is that it is compiled from evidence,
not asserted. A dashboard or summary that drifts from that index — because
it was hand-edited, generated once and never refreshed, or written from an
agent's impression of its own work — reintroduces exactly the failure mode
the standing compiler exists to close. Treat every rendering as disposable
and regenerate it from the index rather than trusting or patching the
rendering itself.
