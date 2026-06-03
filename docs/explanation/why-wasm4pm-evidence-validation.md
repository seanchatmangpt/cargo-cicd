# Why wasm4pm Evidence Validation

## The adjudication problem

A CI/CD tool that only checks its own output has a blind spot: it cannot
verify that the process it claims to run actually happened in the order it
claims. Code can report success while silently skipping steps.

This is not a theoretical concern. Real systems drift: a prune step is
skipped when disk space is plentiful, a trybuild run is short-circuited when
time is short, a publish check is relaxed for a "trivial" fix. Over time,
the tool's self-reported state diverges from what actually happened.

## What wasm4pm provides

wasm4pm is an external process-mining oracle. When the `wasm4pm` feature flag
is enabled, `cargo-cicd` emits structured XES events after each command. wasm4pm
can read these events and verify conformance against a declared process model:

1. Did the commands run in a lawful order?
2. Were any mandatory steps skipped?
3. Does the event log show any impossible state transitions?

This is _external adjudication_: a process outside `cargo-cicd` verifies
`cargo-cicd`'s claims. The oracle cannot be fooled by the tool's own state —
it reads the raw event stream.

## The graceful absence design

wasm4pm integration is opt-in and degrades gracefully. If the `wasm4pm`
binary is not in `PATH`:

- No error is reported.
- All commands run normally.
- Events are still emitted to `.cicd/events.xes` for later analysis.

This means you can add wasm4pm to your workflow incrementally — start with
evidence emission, then add conformance checking when you need it.

## What the oracle checks

The declared process model for a healthy workspace session is:

```
status show → (test changed | trybuild changed)* → git close → publish run
```

The oracle verifies:

- `publish run` is never called without a preceding `test changed` that passed.
- `git close` is never called with dirty files present.
- Events are temporally consistent (no event precedes its prerequisites).

Model-vs-log mismatch is a defect, not a warning. If the oracle reports
non-conformance, the workspace is not in a lawful state.

## Relationship to process-data feature

wasm4pm integration requires the `process-data` feature, which enables XES
event emission. Without `process-data`, no events are written and the oracle
has nothing to check.

The dependency chain is:

```
default ← process-data ← wasm4pm
```

Enable `wasm4pm` and `process-data` is automatically included.
