# Verdict Schemas

## wpm-verdict-v1.json

Contract for the JSON emitted by `wpm audit`.

**CICD-WPM-004 invariant:** No court verdict may silently degrade to zero through key mismatch.

All consumers of `wpm audit` output MUST read the keys specified in this schema.
Do NOT read `fitness` (top-level) — read `overall_fitness`.
Do NOT read `missing_tokens` — read `missing`.
Do NOT read `trace_id` — traces are indexed by position.
Precision is not computed — display as UNSUPPORTED, never as 0.0000.
