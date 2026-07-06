# Standing Schema (`cicd-standing.v1`)

Schema of record for compiled standing documents (`standing.json`) that
describe the release-readiness of every tracked artifact in a workspace.
The matching Rust types live in
`crates/cargo-cicd-core/src/standing/model.rs` (`serde`-derived, same
field names, `SCREAMING_SNAKE_CASE` status tags). When either side
changes, update both together.

## Schema id

The canonical schema id is `cicd-standing.v1`, carried on the
`schema_id` field of the top-level document. `praxis-standing.v1` is
accepted as a legacy alias on read (`is_recognized_schema_id`) — it was
the canonical id before the standing compiler moved into `cargo-cicd`.
Documents missing `schema_id` entirely (written before the field
existed) default to the canonical id on read. New documents always
emit the canonical id.

## The 20 statuses

| Status | Meaning |
|---|---|
| `UNSEEN` | Artifact has not been discovered/indexed by any tooling pass yet. |
| `DISCOVERED` | Artifact is indexed (path, kind known) but nothing has been run against it. |
| `BUILDS` | Artifact compiles/builds cleanly. |
| `TESTED` | Artifact's test suite passes. |
| `LINT_CLEAN` | Artifact passes lint/clippy with no warnings. |
| `BENCHMARKED` | Artifact has at least one attached benchmark result. |
| `RECEIPTED` | A receipt (BLAKE3, genesis-folded) has been computed for the artifact's build/test evidence. |
| `RECEIPT_VERIFIED` | The receipt chain recomputes and verifies (linkage + hash recompute pass). |
| `OCEL_PROVEN` | Claim is backed by events in a validated OCEL v2 log. |
| `WASM4PM_PROVEN` | Claim is backed by wasm4pm process validation (conformance/replay). |
| `CLIENT_VISIBLE` | Artifact is exercised end-to-end from a real client surface (e.g. Playwright-driven UI). |
| `PUBLICATION_READY` | Artifact is ready for a publication artifact (paper, arXiv package) to reference it. Requires `scope`. |
| `PUBLISH_READY` | Artifact is packaged and dry-run verified for publishing (e.g. `cargo publish --dry-run`). Requires `scope`. |
| `PILOT_READY` | Artifact is ready to run in a scoped pilot deployment. Requires `scope`. |
| `PRODUCTION_READY` | Artifact is ready for production use within a stated scope. Requires `scope`. |
| `EXTERNAL_OPERATOR_SIDE_EFFECT` | Remaining action requires a human operator with external credentials (publish, submit, change visibility); packaged locally, executed operator-side. |
| `NON_STANDING` | Artifact intentionally has no standing tracked (e.g. scratch/throwaway). It is out of scope for every gate: readiness policies, the ladder, and the scoped-readiness validation rule do not apply to it. |
| `QUARANTINED` | Artifact is known-bad and excluded from normal gates pending repair. Present in the document for visibility, but its statuses (if any) must not be treated as current truth until repair completes and it is re-evaluated. |
| `RETIRED` | Artifact is no longer maintained/shipped. |
| `DUPLICATE` | Artifact is a duplicate of another tracked artifact; see its `evidence`/notes for the canonical one. |

`model.rs`'s `StandingStatus` enum is the exact set above (`Unseen` …
`Duplicate`), serialized via `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`.

## Readiness ladder (0-9)

A single artifact can carry multiple statuses (its `standing` list). The
ladder collapses that list to one integer, the artifact's furthest
achieved rung, so dashboards and gates can threshold on a single number.

| Level | Status |
|---|---|
| 0 | `DISCOVERED` |
| 1 | `BUILDS` |
| 2 | `TESTED` |
| 3 | `RECEIPTED` |
| 4 | `OCEL_PROVEN` |
| 5 | `WASM4PM_PROVEN` |
| 6 | `REPLAYABLE` |
| 7 | `PUBLISH_READY` |
| 8 | `PILOT_READY` |
| 9 | `PRODUCTION_READY_FOR_SCOPE` |

`ladder_level` is computed (`compute_ladder_level` in `model.rs`), never
asserted directly: it is the max ladder rung among the statuses present
in `standing`. Statuses outside this ladder (`UNSEEN`, `LINT_CLEAN`,
`BENCHMARKED`, `RECEIPT_VERIFIED`, `CLIENT_VISIBLE`,
`PUBLICATION_READY`, `EXTERNAL_OPERATOR_SIDE_EFFECT`, `NON_STANDING`,
`QUARANTINED`, `RETIRED`, `DUPLICATE`) do not move the ladder position
on their own; they are recorded in `standing` but do not raise
`ladder_level` beyond what the ladder statuses in the same list
justify. `RECEIPT_VERIFIED` implies but is distinct from ladder rung 3
(`RECEIPTED`).

Rung 6 (`REPLAYABLE`) has no dedicated status in the v1 status list — it
is reached implicitly once verified receipts and OCEL/wasm4pm proof are
combined by an upstream policy, not computed from a single status by
`ladder_rung`.

Rung 9 (`PRODUCTION_READY_FOR_SCOPE`) is only reached when
`PRODUCTION_READY` is present **and** carries a non-empty `scope`; an
unscoped `PRODUCTION_READY` falls back to rung 8, matching
`StandingStatus::ladder_rung`'s implementation.

## Artifact JSON shape

```json
{
  "id": "string, stable identifier",
  "kind": "rust_crate | client | doc | paper | bench | workflow | ontology | binary",
  "path": "string, repo-relative path",
  "standing": ["STATUS", "..."],
  "scope": "string, required iff PRODUCTION_READY/PILOT_READY/PUBLISH_READY/PUBLICATION_READY present, else omitted",
  "ladder_level": 0,
  "evidence": [
    {"kind": "command", "command": "string", "exit_code": 0, "utc": "RFC3339", "artifact": "path or null"},
    {"kind": "ocel_event", "event_id": "string", "path": "path to OCEL log"},
    {"kind": "receipt", "chain_hash": "blake3:...", "path": "path to receipt chain"},
    {"kind": "artifact", "path": "string", "hash": "blake3:..."}
  ],
  "external_operator_side_effects": ["string, e.g. 'crates.io publish requires operator credentials'"]
}
```

This is `StandingArtifact` in `model.rs`; `scope` maps to `Option<String>`
(omitted from JSON when `None`), `evidence` to `Vec<EvidenceRef>` (an
internally-tagged enum on `kind`), and `kind` (artifact kind) to
`ArtifactKind`.

### Worked example

```json
{
  "id": "example-crate",
  "kind": "rust_crate",
  "path": "crates/example-crate",
  "standing": [
    "BUILDS",
    "TESTED",
    "RECEIPT_VERIFIED",
    "OCEL_PROVEN",
    "WASM4PM_PROVEN",
    "PUBLISH_READY"
  ],
  "scope": "local release validation and crates.io dry-run",
  "ladder_level": 7,
  "evidence": [
    {
      "kind": "command",
      "command": "cargo test -p example-crate",
      "exit_code": 0,
      "utc": "2026-07-06T19:00:00Z",
      "artifact": null
    },
    {
      "kind": "ocel_event",
      "event_id": "evt_18",
      "path": "target/cargo-cicd/evidence/example-crate.xes"
    },
    {
      "kind": "receipt",
      "chain_hash": "blake3:9f8e1e18…5f1d91",
      "path": "receipts/example-crate.json"
    },
    {
      "kind": "command",
      "command": "cargo publish --dry-run --allow-dirty -p example-crate",
      "exit_code": 0,
      "utc": "2026-07-06T19:44:59Z",
      "artifact": null
    }
  ],
  "external_operator_side_effects": [
    "real crates.io publish requires operator credentials"
  ]
}
```

`ladder_level` is 7 (`PUBLISH_READY`) because `PRODUCTION_READY`/`PILOT_READY`
are absent; `RECEIPT_VERIFIED` and `WASM4PM_PROVEN` are both present but the
ladder position is governed by the highest ladder-listed status
(`PUBLISH_READY`, rung 7), not by the count of statuses held.

## Top-level document shape

```json
{
  "schema_id": "cicd-standing.v1",
  "release_id": "string, e.g. v26.7.6",
  "generated_at_utc": "RFC3339",
  "generator": "string, tool/command that produced this document",
  "standing_version": "1",
  "artifacts": [/* StandingArtifact, ... */]
}
```

This is `StandingDocument` in `model.rs`. `schema_id` defaults to the
canonical id via `#[serde(default = "default_schema_id")]`, so documents
predating the field still parse.

## Validation rule (scoped-readiness)

Any artifact whose `standing` list contains one or more of
`PRODUCTION_READY`, `PILOT_READY`, `PUBLISH_READY`, `PUBLICATION_READY`
**must** carry a non-empty `scope` string. An artifact with any of these
four statuses and a missing or empty `scope` is invalid: `StandingArtifact::validate`
returns a typed `StandingError::MissingScope` rather than panicking or
silently defaulting. A readiness claim without a stated scope is not a
claim at all — a "production ready" or "publish ready" tag only carries
meaning when it says ready *for what*.

`NON_STANDING` and `QUARANTINED` are exempt from this rule and from the
ladder: they mark an artifact as out of scope for readiness tracking
entirely (`NON_STANDING`) or as known-bad and held out of normal gates
pending repair (`QUARANTINED`), not as claims requiring evidence.

## Determinism rule

Standing compilation must be deterministic: given the same workspace
state and evidence inputs, re-running the compiler must produce a
byte-for-byte-equivalent `standing.json` (modulo `generated_at_utc`).
Non-deterministic inputs into the compiled document — including
ontology-derived data whose serialization order was not fixed — must be
sorted or otherwise stabilized in the generating pipeline before this
schema's constraints (e.g. `ladder_level` reproducibility, stable
`artifacts` ordering) can be relied on by downstream consumers. See the
ordering fix applied to the ontology TTL feeding the manufacturing
pipeline (`ontology/cargo-cicd-capabilities.ttl` → `ggen`) for the
concrete instance of this rule in this workspace.

## OCEL 2.0 relation

`EvidenceRef::OcelEvent { event_id, path }` entries point at OCEL 2.0
event log files (the same XES/OCEL evidence emitted per
`src/evidence.rs`'s `start`/`complete` event pattern). `OCEL_PROVEN`
standing requires at least one such reference resolving to a real event
in a log that has itself been validated (not merely present) — an
`event_id` that does not resolve, or that resolves in an unvalidated
log, does not justify `OCEL_PROVEN`.

## wasm4pm relation

`WASM4PM_PROVEN` requires that the artifact's process conduct has been
validated by the `wpm` oracle (`wpm audit <file.xes>` returning
`Accept`), per invariant **E1** in `src/evidence.rs`: cargo-cicd never
adjudicates itself; only `wpm` issues verdicts. A `WASM4PM_PROVEN` tag
backed only by an internal check and no corresponding `wpm` verdict is
not a valid claim under this schema. When `wpm` is unavailable, the
correct standing is the absence of `WASM4PM_PROVEN` (or an explicit
note under `evidence`/`external_operator_side_effects`), not a
provisional grant of the status — mirroring invariant **E7**, where a
blocked oracle is a first-class expectation rather than an error to be
papered over.

## Consumer responsibilities

Any tool or agent (Claude Code, `praxis`, CI, a dashboard) that reads a
`standing.json` and treats it as truth about release readiness must,
before doing so:

1. **Check `schema_id`.** Reject or flag documents whose `schema_id`
   fails `is_recognized_schema_id` — an unrecognized id may carry a
   status vocabulary this schema does not describe.
2. **Never infer readiness from absence.** A missing status is not
   evidence of failure and is not evidence of success; it means
   "not yet run/recorded." Only explicit `standing` entries and their
   `evidence` count.
3. **Follow the evidence, not just the tag.** A status string alone is
   a claim; the `evidence` array is what backs it. A consumer deciding
   whether to trust `OCEL_PROVEN` or `WASM4PM_PROVEN` should confirm the
   referenced event/log/receipt actually exists and validates, not just
   that the string is present in `standing`.
4. **Respect the scoped-readiness rule.** Never treat `PRODUCTION_READY`,
   `PILOT_READY`, `PUBLISH_READY`, or `PUBLICATION_READY` as actionable
   without reading the accompanying `scope` string — readiness is only
   ever readiness *for* that stated scope, not unconditional readiness.
5. **Treat `QUARANTINED` as a hard stop for gates**, and `NON_STANDING`
   as out-of-scope, not as a failing or passing grade respectively.
6. **Never generate or forward a promise of an operator-side action.**
   `external_operator_side_effects` entries describe steps that require
   a human with real credentials (e.g. an actual `crates.io` publish);
   a consumer must surface these to the human rather than attempt them
   or imply they have already happened.
7. **Recompute, don't trust, `ladder_level` when in doubt.** It is a
   derived field; if a consumer needs to gate on it, recomputing from
   `standing` (and `scope` for the rung-9 case) via the same rule this
   document specifies is safer than trusting an externally-supplied
   integer verbatim.
