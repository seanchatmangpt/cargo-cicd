# Pull Request Workflow

How to create, structure, and get your changes merged.

## Branch Naming Convention

Use one of these patterns:

```
feat/noun-description          # New feature
fix/issue-number-description   # Bug fix
refactor/scope-description     # Code refactoring
test/scenario-description      # Test additions or fixes
docs/topic-description         # Documentation updates
chore/task-description         # Build, CI, deps
```

### Examples

- `feat/status-show-detailed-output`
- `fix/1234-handle-missing-toolchain`
- `refactor/engine-state-composition`
- `test/invariants-boundary-case`
- `docs/cli-noun-verb-grammar`
- `chore/bump-clap-version`

## Commit Message Format

All commits must follow the format defined in [CLAUDE.md](../../CLAUDE.md):

```
<type>(<scope>): <subject>

<body>
<blank line>
<optional footer>
```

### Type
One of: `feat`, `fix`, `refactor`, `test`, `docs`, `ci`, `chore`

### Scope
One of: `core`, `cli`, `target`, `test`, `git`, `autonomic`, `docs`, `receipts`

### Subject
- Imperative mood ("add" not "added" or "adds")
- No period at the end
- Under 50 characters
- Lowercase

### Body
- Explain **what** and **why**, not how
- Wrap at 72 characters
- Blank line between subject and body
- Reference issues: `Closes #123`

### Example

```
feat(core): add workspace diagnostics to engine state

Extend EngineState with WorkspaceDiagnostics struct to capture
structural health checks (duplicate dependencies, version skew,
toolchain mismatch). This enables the `workspace doctor` noun to
read diagnosis data from the engine rather than computing it
inline.

Includes new WorkspaceDiagnosticAdapter to populate the field
during engine initialization.

Closes #456
```

## Creating a Pull Request

### Step 1: Create and Push Your Branch

```bash
# Create a feature branch from main
git checkout -b feat/your-feature-name

# Make your changes, test locally
cargo test

# Stage and commit
git add src/your_file.rs tests/your_test.rs
git commit -m "feat(core): your feature description

Description of what and why.

Closes #issue-number"

# Push to your fork
git push -u origin feat/your-feature-name
```

### Step 2: Open the PR on GitHub

Use the GitHub web interface or `gh` CLI:

```bash
gh pr create --title "Brief description of your change" \
  --body "$(cat <<'EOF'
## Summary
- What this change does
- Why it's needed
- Any relevant context

## Testing
- [ ] Added unit tests for new code
- [ ] Ran `cargo test` successfully
- [ ] Tested the feature manually (if applicable)
- [ ] Updated CLAUDE.md if architecture changed (if applicable)

## Checklist
- [ ] Commit message follows format: `type(scope): subject`
- [ ] No forbidden terms in public docs/help text
- [ ] All tests pass
- [ ] Code follows project style guidelines
EOF
)"
```

### Step 3: Code Review

Expect feedback on:
- **Correctness** — Does it actually work?
- **Architecture** — Does it fit the design?
- **Tests** — Are there adequate tests?
- **Docs** — Is it documented?
- **Style** — Does it follow conventions?

Address feedback by:
1. Making changes locally
2. Committing with a new commit (don't amend previous commits; the reviewer sees the history)
3. Pushing: `git push`
4. Replying to comments in the PR

### Step 4: Merge

Once approved:
- Maintainers will merge using **squash** or **rebase** strategy
- Your PR branch will be deleted automatically
- Your changes are live on main

## Common PR Patterns

### Feature PR

```
Title: "Add workspace diagnostics noun"

Body:
## Summary
- Adds `cargo cicd workspace doctor` command
- Displays duplicate dependencies, version skew, toolchain mismatch

## Testing
- [x] Tests pass: `cargo test --test cli`
- [x] Manual test on fixture workspace

## Related
- Closes #456
```

### Bug Fix PR

```
Title: "Fix panic when cicd.toml is corrupted"

Body:
## Summary
- Handles malformed TOML gracefully
- Returns helpful error message instead of panicking

## Testing
- [x] Regression test added: `tests/corrupted_cicd_toml_handling.rs`
- [x] All tests pass

## Validation
- Tested with fixture: `tests/fixtures/corrupted_cicd_toml/`
```

### Refactoring PR

```
Title: "Simplify EngineState initialization"

Body:
## Summary
- Consolidates initialization logic into a builder
- Reduces duplication across adapters

## Testing
- [x] All existing tests pass
- [x] No behavioral changes

## Notes
- Pure refactor, no new features
- Tests verify no regressions
```

## Before You Push

### Checklist

- [ ] Tests pass: `cargo test`
- [ ] Build is clean: `cargo build`
- [ ] Lint is clean: `cargo clippy -- -D warnings` (if using stable Rust)
- [ ] Formatting is correct: `cargo fmt -- --check`
- [ ] Commit message follows format (see above)
- [ ] No forbidden terms in public-facing code or docs
- [ ] Feature flag changes documented (if applicable)
- [ ] CLAUDE.md updated if architecture changed (if applicable)

### Run Full Checks Locally

```bash
# Check all at once
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Or with cargo-make:

```bash
cargo make check
cargo make test
cargo make build
```

## Review Expectations

### As an Author
- Respond to feedback promptly
- Explain your reasoning, don't just accept suggestions blindly
- Push new commits for review; don't rewrite history
- Keep PRs focused: one feature or fix per PR

### As a Reviewer (if you review others' PRs)
- Check for correctness, not just style
- Ask questions if logic is unclear
- Suggest, don't dictate (use "consider" language)
- Approve when confident; request changes when needed

## Merging Strategy

The project uses:
- **Squash merges** for single-commit features
- **Rebase merges** for multi-commit features where history matters
- **Merge commits** are avoided to keep history clean

You don't need to do anything; maintainers choose the strategy when merging.

## After Your PR is Merged

1. Delete your local branch: `git branch -d feat/your-feature-name`
2. Sync main: `git checkout main && git pull`
3. Start your next contribution!

## Getting Help with a PR

If you're stuck:
- Ask in the PR comments for guidance
- Reference related issues or discussions
- Look at similar PRs in the history for patterns
- Check [CLAUDE.md](../../CLAUDE.md) for architecture clarification

## Related Guides

- [Code Style & Patterns](./04-code-style.md) — conventions to follow
- [Documentation Standards](./05-documentation-standards.md) — when to update docs
- [Adding Features](./03-adding-features.md) — how to structure new capabilities
