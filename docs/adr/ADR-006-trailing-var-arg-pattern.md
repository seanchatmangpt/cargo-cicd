# ADR-006: Trailing Var-Arg Pattern for Noun-Verb Routing

**Status:** Accepted
**Date:** 2026-06-03

## Context

The `clap-noun-verb` crate routes positional arguments through a noun-verb hierarchy. When a verb needs to accept multiple positional arguments (e.g., a list of targets, file paths, or test names), there are several candidate patterns: multiple named args, a comma-separated string, repeated flags, or a trailing var-arg. Each has different behavior under the clap-noun-verb parser.

## Decision

The trailing var-arg is the canonical pattern for all verbs that accept open-ended positional arguments. All positional arguments after the verb are collected as a `Vec<String>` via `args.trailing_vararg("name")`.

```rust
fn run(&self, args: &VerbArgs) -> anyhow::Result<()> {
    let targets: Vec<String> = args.trailing_vararg("targets")?;
    // targets contains all positional args after the verb name
}
```

No other pattern is used for open-ended positional arguments.

## Rationale

Under clap-noun-verb routing, the noun and verb names consume the first two positional slots. Any pattern that tries to name additional positional arguments by position (third, fourth, etc.) produces ambiguous parse results when the verb name is elided via default verb injection. The trailing var-arg pattern is the only one that remains unambiguous after default verb injection.

## Consequences

- All verbs accepting lists of items use `trailing_vararg`.
- `inject_default_verbs()` in `main.rs` can safely inject defaults without disturbing positional argument order.
- Tests that pass positional arguments to verbs use the trailing position without naming intermediate slots.
- Documentation for each verb describes its trailing var-arg semantics explicitly.

## Violation

If positional arguments are indexed by position (e.g., "third arg is the target"), default verb injection breaks the indexing. Verbs that use named multi-value flags instead of trailing var-arg produce non-idiomatic CLI surfaces that differ from all other nouns in the system.
