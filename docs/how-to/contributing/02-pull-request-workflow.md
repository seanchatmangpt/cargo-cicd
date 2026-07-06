# Pull Request Workflow

This guide covers branch naming, commit messages, code review expectations, and the merge process.

## Branch Naming

Use descriptive branch names that reflect the type and scope of work:

```
feat/noun-verb-action         # New feature: e.g., feat/target-prune
fix/scope-description          # Bug fix: e.g., fix/git-dirty-detection
refactor/scope-change          # Refactoring: e.g., refactor/adapter-patterns
docs/section-improvement       # Documentation: e.g., docs/setup-guide
test/feature-coverage          # Tests: e.g., test/policy-scenarios
ci/build-improvement           # CI/build: e.g., ci/github-actions
```

Examples:
- `feat/status-audit-policy` — new status audit policy
- `fix/target-size-calculation` — fix how target size is computed
- `refactor/engine-state-layout` — reorganize EngineState
- `docs/adapter-lifecycle` — document adapter patterns

## Commit Message Format

Follow the format from `CLAUDE.md`:

```
<type>(<scope>): <description>

<detailed explanation (optional)>

<link to Claude Code session (optional)>
```

### Types

- **feat** — new feature or capability
- **fix** — bug fix
- **refactor** — code refactoring (no behavior change)
- **test** — test additions or fixes
- **docs** — documentation updates
- **ci** — CI/build configuration changes

### Scopes

Valid scopes (from CLAUDE.md):
- **core** — core engine logic (EngineState, adapters, policies)
- **cli** — CLI commands and nouns/verbs
- **target** — target/ directory scanning and pruning
- **test** — test infrastructure and test detection
- **git** — git integration and phase closure
- **autonomic** — autonomic policies and suggest mode
- **docs** — documentation and guides
- **receipts** — evidence emission and XES handling

### Examples

```
feat(core): add WorkspaceState::member_crates()

Extract crate membership logic to a dedicated method
to reduce duplication in ChangedFileDetector and TestPlanState.
```

```
fix(git): handle detached HEAD in git phase state

When HEAD is detached, GitPhaseState.branch should return
"HEAD" instead of panicking.
```

```
refactor(adapters): consolidate metadata queries

Use cargo metadata once per invocation, cache in a thread-local,
and have all adapters query the cache. Reduces subprocess calls
by 60% on large monorepos.
```

```
docs(docs): add feature flag gating matrix

Document which features gate which dimensions, with examples
of how to feature-gate new code.

https://claude.ai/code/session_01GTNEZYe16QF5TzVZUZSvsA
```

## Before Opening a PR

1. **Update and build:**
   ```bash
   git fetch origin
   git rebase origin/main
   cargo build
   ```

2. **Run all tests:**
   ```bash
   cargo test
   ```

3. **Format code:**
   ```bash
   cargo fmt
   ```

4. **Lint code:**
   ```bash
   cargo clippy -- -D warnings
   ```

5. **Check for forbidden terms** (if modifying public-facing code):
   ```bash
   cargo test --test invariants invariant_public_boundary
   ```

## Opening a PR

1. **Push your branch:**
   ```bash
   git push -u origin feat/my-feature
   ```

2. **Create PR on GitHub:**
   - Title: brief description (under 70 chars)
   - Description: what, why, how; reference any issues
   - Link to CLAUDE.md if architectural questions

3. **PR title format:**
   - Start with the commit type and scope: `feat(core): ...`, `fix(git): ...`
   - Keep it short and scannable

4. **PR description template:**
   ```markdown
   ## Summary
   - Brief one-liner of what this PR does
   - Why it's needed (user impact or tech debt)
   - How it solves the problem

   ## Changes
   - [ ] New feature: describe the API/behavior
   - [ ] Bug fix: describe the symptom and root cause
   - [ ] Refactoring: describe what moved/changed
   - [ ] Tests: describe test coverage added

   ## Testing
   - [ ] All integration tests pass (`cargo test`)
   - [ ] No forbidden terms in help text (if public-facing)
   - [ ] Existing fixtures cover new code paths
   - [ ] New fixtures added if needed

   ## Checklist
   - [ ] Commit message follows `feat(scope): description` format
   - [ ] Code formatted with `cargo fmt`
   - [ ] Lints pass with `cargo clippy -- -D warnings`
   - [ ] Feature flag containment verified (if adding features)
   - [ ] CLAUDE.md updated (if architectural changes)

   Closes #<issue_number> (if applicable)
   ```

## Code Review Expectations

### What Reviewers Will Check

1. **Correctness**
   - Does the code do what it claims?
   - Are edge cases handled?
   - Is error handling adequate?

2. **Architecture**
   - Does it follow adapter → noun pattern?
   - Is EngineState mutation restricted to adapters?
   - Is state immutable where it should be?
   - Are feature flags correctly gated?

3. **Testing**
   - Is there test coverage for the main paths?
   - Are fixtures used appropriately?
   - Do tests isolate from external state?

4. **Documentation**
   - Are public APIs documented (doc comments)?
   - Is the purpose clear from code structure?
   - Does CLAUDE.md need updates?

5. **Style**
   - Does code follow Rust conventions?
   - Are variable names clear and consistent?
   - Are comments helpful (not redundant)?

6. **Forbidden Terms**
   - Does the PR introduce any forbidden terms in public output?
   - (List: ALIVE, Nehemiah, CONSTRUCT8, Instinct8, Inspection Gate, Cargo Court, AGI, Truex, Field8, wall)

### Common Review Comments

**"Adapter pattern"**
- This logic should move to an adapter in `src/adapters/`
- Adapters own external data translation; nouns consume state

**"State mutation"**
- EngineState fields are immutable snapshots
- Mutations flow through adapters → CicdTomlWriter
- Nouns should read state, not modify it

**"Feature gating"**
- New level-5 engine code should gate behind `#[cfg(feature = "process-data")]`
- Autonomic policies should gate behind `#[cfg(feature = "autonomic")]`

**"Public boundary"**
- This string is user-visible; check it against the forbidden term list
- Help text should be clear without internal jargon

**"Test isolation"**
- Tests must not depend on external state (previous test run, git commits, etc.)
- Use `FixtureWorkspace` to create isolated test environments

## After Review

1. **Address feedback:**
   - Push new commits (don't force-push during review)
   - Reply to each comment
   - Suggest re-review if major changes

2. **Approvals:**
   - Two approvals required (or maintainer discretion for small changes)
   - All CI checks must pass

3. **Merge:**
   - Maintainer will squash & rebase or fast-forward merge
   - Commit message will be validated against format

## Release Considerations

If your PR includes:
- **New public verbs** → update help text, add to invariants test
- **New EngineState dimension** → document in CLAUDE.md, add tests
- **New adapter** → document source and invariants in CLAUDE.md
- **Policy changes** → test against wasm4pm evidence gate
- **Evidence format changes** → coordinate with wasm4pm team

## Revert Policy

If a PR is merged and later causes issues:
- Maintainer will revert with a message: `revert: <original title> (#<PR>)`
- Reopened PR must address the root cause before re-merge
- Failures in release validation are grounds for immediate revert

## Further Reading

- [CLAUDE.md](../../../CLAUDE.md) — architecture details referenced in reviews
- [03-adding-features.md](./03-adding-features.md) — patterns for common changes
- [04-code-style.md](./04-code-style.md) — style expectations in detail
