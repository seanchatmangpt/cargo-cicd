# Known Gotchas

Common pitfalls and how to avoid them.

## Forbidden Terms in Public Output

**The Rule:** The following terms must **never** appear in user-visible output (help text, stdout, error messages):

```
internal engine codename
internal scoring codename
internal build-tag codename
internal build-tag alias
evidence gate / oracle adjudication (internal review-step codename)
internal workspace codename
policy engine (internal automation codename)
internal release-track codename
internal capability-tag codename
policy boundary (internal pressure-mechanism codename)
```

These are internal implementation details and architectural references. Users should see only: "CI/CD helper", "workspace cleaner", "test runner", etc.

### How to Catch This

The `invariants` test enforces this:

```bash
cargo test --test invariants invariant_public_boundary
```

This test runs every `--help` command and checks for forbidden terms.

### Example: The Bug

Do not include an internal engine codename in output. Search and remove it:

```bash
grep -r "<forbidden-term>" src/
# ... remove the offending line ...
```

If you see this test fail, search for the term and remove it.

## State Mutation Patterns

**The Rule:** `EngineState` is an immutable snapshot. Mutations happen only through adapters to `CicdTomlWriter`.

### Anti-Pattern 1: Mutating EngineState in a Verb

Verbs are read-only consumers. Never mutate EngineState. Have the adapter populate the state, then read it.

### Anti-Pattern 2: Adapter Returns Mutable State

State should be built immutable directly. Do not mutate state after construction. Build immutable values in one place.

### Anti-Pattern 3: Circular Dependency in State

Never embed full state within another state struct. Reference by ID or path instead. Keep state dimensions independent.

## Test Isolation Failures

**The Rule:** Tests must not depend on external state: previous test runs, git commits, filesystem files, environment variables.

### Anti-Pattern 1: Test Depends on External Git State

Never assume the current git repo state in tests. Use `FixtureWorkspace` to create isolated test environments. The fixture is a fresh git repo that you control.

### Anti-Pattern 2: Test Leaves Behind Temp Files

Always use `tempfile::TempDir` which cleans up automatically. Or use `FixtureWorkspace` which handles this internally.

### Anti-Pattern 3: Test Assumes Environment Variable

Either make the test not depend on environment, or mock it via files. Do not assume MY_CONFIG or other env vars are set.

## Feature Flag Gating Mistakes

**The Rule:** Use `#[cfg(feature = "...")]` at compile time, not runtime checks.

### Anti-Pattern 1: Runtime Feature Check

Checking `cfg!(feature = "...")` at runtime compiles the code regardless of feature. Use compile-time `#[cfg(...)]` guards instead.

### Anti-Pattern 2: Feature-Gated Code Without Stub

Always provide a stub implementation for when the feature is off. Otherwise the code doesn't compile without the feature.

## Adapter Query Mistakes

**The Rule:** Adapters query external sources once and return immutable results. No caching, no side effects.

### Anti-Pattern 1: Adapter Modifies External State

Never have an adapter call destructive git commands like reset or clean. Adapters only read, they never write or modify.

### Anti-Pattern 2: Adapter Caches Incorrectly

If caching is needed, validate freshness. Query fresh data every time in the adapter. Cache at a higher level (cicd.toml) if needed.

## Common cicd.toml Mistakes

**The Rule:** cicd.toml is a state carrier. It should be written by adapters only, read by nouns.

### Anti-Pattern 1: Manually Editing cicd.toml

Never hand-edit cicd.toml. Have the adapter write it via CicdTomlWriter to ensure consistency.

### Anti-Pattern 2: Stale cicd.toml Not Invalidated

Always validate cache freshness against current state. If git changed since the cache was written, regenerate it.

## Evidence Emission Mistakes

**The Rule:** Process events are emitted to XES format. They must be valid and complete.

### Anti-Pattern 1: Missing Event Timestamp

Always include a timestamp in ProcessEvent. wasm4pm will reject events without timestamps.

### Anti-Pattern 2: Invalid Event Type

Use standardized event types (see evidence.rs). Do not use arbitrary event_type strings. Examples: "noun_verb_invoked", "test_started", "policy_evaluated".

## Troubleshooting Quick Reference

| Symptom | Cause | Fix |
|---------|-------|-----|
| Test fails with forbidden term | internal codename leaked into help text | grep and remove the term |
| Test passes alone but fails in CI | Test depends on git state | Use FixtureWorkspace::clean() |
| Adapter returns different results on second call | Adapter mutates external state | Ensure adapter only reads |
| cicd.toml becomes inconsistent | Manually edited or stale cache | Always regenerate via adapter |
| wasm4pm evidence-gate fails | Event missing timestamp | Check evidence.rs for standard types |
| Feature code compiles without feature | Missing #[cfg(...)] guard | Wrap with #[cfg(feature = "...")] |

## Prevention Checklist

Before opening a PR:

- [ ] No forbidden terms in public output (run invariants test)
- [ ] All state mutations go through adapters (code review check)
- [ ] Tests use FixtureWorkspace, not real git state
- [ ] Adapters do not modify external sources (read-only)
- [ ] Feature gates use #[cfg(...)], not runtime checks
- [ ] cicd.toml is written by CicdTomlWriter, not manually
- [ ] Process events have timestamps and standard types
- [ ] No circular state dependencies in EngineState
