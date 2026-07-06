# Claim Rules

Every prose claim of readiness in a workspace (a doc, a commit message, a PR
description, an agent's status report) should be checked against that
workspace's compiled `cicd-standing.v1` index (produced by
`cargo cicd standing refresh`, typically written to
`target/<workspace>-standing/standing.json`). A consumer such as
anti-llm-cheat-lsp can enforce these rules as diagnostics; this document
states each rule in plain language with an example violation and fix so a
human or an agent can apply the same discipline even without that tooling
wired up.

See `docs/reference/standing-schema.md` for the schema these rules check
against.

## The rules

### 1. Index missing, unparseable, or internally invalid

**Rule**: the standing index must exist, parse as `cicd-standing.v1` JSON,
and every entry must pass the scoped-readiness validation rule (any of
`PRODUCTION_READY` / `PILOT_READY` / `PUBLISH_READY` / `PUBLICATION_READY`
requires a non-empty `scope`).

- **Violation example**: `cargo cicd standing refresh` was never run in this
  checkout, so the standing index does not exist, but a doc claims
  "the exporter is PUBLISH_READY."
- **Fix**: run `cargo cicd standing refresh` to produce a fresh, valid index
  before making the claim.

### 2. Unscoped readiness claim

**Rule**: any claim using "production-ready", "pilot-ready",
"publish-ready", or "publication-ready" language must carry an explicit
scope phrase ("for `<scope>`" / "scoped to `<scope>`") in the same sentence.

- **Violation example**: "the exporter is production-ready." (no scope
  stated)
- **Fix**: "the exporter is PRODUCTION_READY for local release validation
  and crates.io dry-run."

### 3. Claimed status outruns the index

**Rule**: a claim naming a specific standing status (`PRODUCTION_READY`,
`PILOT_READY`, `PUBLISH_READY`, `PUBLICATION_READY`, `OCEL_PROVEN`,
`WASM4PM_PROVEN`) for a named artifact must find that exact status in the
artifact's `standing` list in the index.

- **Violation example**: "the platform client is OCEL_PROVEN" when the
  index lists that artifact's `standing` as `["BUILDS", "TESTED"]` only.
- **Fix**: either run the OCEL process-validation pass and re-refresh the
  index so `OCEL_PROVEN` actually appears, or downgrade the claim to what
  the index supports (`TESTED`).

### 4. "Published" claimed without an operator-side-effect record

**Rule**: `cicd-standing.v1` has no `PUBLISHED` status — only
`PUBLISH_READY` (dry-run verified) and `EXTERNAL_OPERATOR_SIDE_EFFECT` (an
operator actually completed the external action) exist. A claim that
something is "published" must find a non-empty
`external_operator_side_effects` entry on that artifact.

- **Violation example**: "the crate is published to crates.io" when the
  standing entry only carries `PUBLISH_READY` and the
  `external_operator_side_effects` list still reads "real crates.io publish
  requires operator credentials" (i.e. it has not happened).
- **Fix**: say "the crate is PUBLISH_READY (dry-run verified); real publish
  is pending operator action" until an operator runs `cargo publish` and the
  side-effect entry is updated to reflect completion.

### 5. "Operational"/"verified" claimed without receipt or OCEL backing

**Rule**: a claim that an artifact is running, verified, or otherwise
observably real must find `RECEIPT_VERIFIED` or `OCEL_PROVEN` in that
artifact's `standing` list.

- **Violation example**: "the release is confirmed operational" with no
  receipt-verify or OCEL evidence attached to the release artifact in the
  index.
- **Fix**: attach the receipt-verify run or OCEL log as evidence, refresh
  the index, then make the claim — or state the actual status (`TESTED`,
  `BUILDS`) instead.

### 6. "Benchmarked" claimed without evidence

**Rule**: a benchmarked claim must find either the `BENCHMARKED` status or
at least one evidence entry on that artifact.

- **Violation example**: "the graph engine is benchmarked" with an empty
  evidence array and no `BENCHMARKED` status.
- **Fix**: run the benchmark, attach the raw output path as a `command` or
  `artifact` evidence entry, refresh, then claim it.

### 7. Stale index

**Rule**: the index's generation timestamp must be no older than a
configured freshness window (a sensible default is 24h) relative to the
current time.

- **Violation example**: the standing index was generated three days ago; a
  claim is made today without re-refreshing it.
- **Fix**: re-run `cargo cicd standing refresh` before relying on the index
  for any claim older than the configured freshness window.

## Non-blocking exemption

Consumers that enforce these rules as diagnostics may exempt certain paths
(vision/roadmap docs, archived releases) from gate-failing while still
reporting the finding. This is a consumer-side configuration concern, not a
property of the standing index itself — do not invent a parallel exemption
mechanism per consumer.
