# Git Hooks — cargo-cicd

Automated quality gates enforced by POSIX-compatible shell scripts installed
into `.git/hooks/`. All hooks print colored `✓`/`✗` output with per-check
timing. Color is suppressed automatically when stdout is not a terminal.

---

## Installation

Run from the repository root:

```sh
sh scripts/install-hooks.sh
```

This copies `scripts/hooks/*` into `.git/hooks/` and sets executable
permissions on each file.

To overwrite hooks that already exist (e.g. after updating this repo):

```sh
sh scripts/install-hooks.sh --force
```

Verify installation:

```sh
ls -la .git/hooks/
```

You should see `pre-commit`, `pre-push`, and `commit-msg` listed as
executable files.

---

## Hooks Reference

### `pre-commit`

Runs automatically on every `git commit`. Must complete in under 30 seconds.

| Check | Command | Failure action |
|---|---|---|
| Formatting | `cargo fmt --check` | Reject; run `cargo fmt` to fix |
| Lint | `cargo clippy --quiet -- -D warnings` | Reject; fix all warnings |
| Invariants | `cargo test --test invariants --quiet` | Reject; see `tests/invariants.rs` |
| Forbidden terms | grep on staged files | Reject; remove flagged terms |

**Forbidden terms** scanned in every staged file:

```
ALIVE  Nehemiah  CONSTRUCT8  Inspection Gate  Cargo Court
AGI  Truex  wall  Field8  Instinct8
```

These terms must not appear in any committed file. See `CLAUDE.md §FORBIDDEN`
for the rationale.

The forbidden-term scan reads staged file contents via `git show :$file` so
it checks what will actually be committed, not the working-tree version.

---

### `pre-push`

Runs automatically on every `git push`. Performs three phases:

**Phase 1 — pre-commit checks (repeated)**

All four pre-commit checks are re-run against the full tracked file set
(not just staged files) to catch anything that slipped through.

**Phase 2 — full test suite**

| Check | Command |
|---|---|
| Full test suite | `cargo test --quiet` |
| Feature projection | `cargo test --test feature_projection --quiet` |

The wasm4pm evidence-gate tests (`wasm4pm_evidence_gate`,
`wasm4pm_evidence_mutation`, `wasm4pm_refusal_cases`) are **not** a push
gate. They are a release-closing gate. Those tests self-skip when the `wpm`
binary is absent; no special exclusion is needed here.

**Phase 3 — source hygiene**

| Check | What it looks for |
|---|---|
| No release blockers | `TODO(release-block)` in `*.rs`, `*.toml`, `*.md` |
| Docs non-empty | `TESTING_GUIDE.md` and `CONTRIBUTING.md` must exist and be non-empty |

---

### `commit-msg`

Runs automatically after you type a commit message. Enforces the
conventional-commit format documented in `CLAUDE.md`.

**Required format:**

```
<type>(<scope>): <description>
```

**Valid types:**

```
feat  fix  docs  refactor  test  ci  chore  perf
```

**Valid scopes:**

```
core  cli  target  test  git  autonomic  docs  receipts
```

**Additional rules:**

- First line must not be empty.
- First line must not exceed 72 characters.
- Breaking-change suffix `!` is accepted: `feat(core)!: redesign engine API`.
- Scope may be omitted (warns but passes): `fix: correct typo in README`.

**Examples of valid messages:**

```
feat(core): add workspace scan adapter
fix(cli): handle missing cicd.toml gracefully
docs(receipts): document receipt schema fields
refactor(git): simplify phase closure detection
test(autonomic): add policy suggestion regression test
ci(docs): add GIT_HOOKS documentation
chore(target): remove stale build artifact cache
perf(core): cache toolchain detection result
```

---

## Troubleshooting

### `cargo fmt --check` fails

Your staged code is not formatted. Fix it:

```sh
cargo fmt
git add -u
git commit
```

### `cargo clippy` fails

View the warnings with context:

```sh
cargo clippy -- -D warnings
```

Fix each warning. Clippy warnings are errors in this project by policy.

### Invariants test fails

The seven public-boundary invariants defined in `tests/invariants.rs` are
broken. Run the test directly for the full failure output:

```sh
cargo test --test invariants
```

### Forbidden term found in staged file

Remove the flagged term. If it appears in documentation context that is
legitimately referencing the term (e.g. quoting this very list), consider
whether the file should be staged at all. The scan checks only staged content
in pre-commit, and all tracked files in pre-push.

### commit-msg hook rejects my message

Ensure your message matches `<type>(<scope>): <description>`. Common mistakes:

- Wrong type: `update` is not valid — use `feat` or `refactor`.
- Missing colon-space: `feat(core) add thing` → `feat(core): add thing`.
- First line too long: split at 72 characters; put detail in the body.

### Hook not running

Check that the hook file is executable:

```sh
ls -la .git/hooks/pre-commit
```

If not executable:

```sh
chmod +x .git/hooks/pre-commit .git/hooks/pre-push .git/hooks/commit-msg
```

Or re-run the installer:

```sh
sh scripts/install-hooks.sh --force
```

### Hook runs but produces no color

Color output requires a terminal (`-t 1` check). When running in a CI
environment or piped context, output is plain text. This is intentional.

---

## Emergency Bypass

> **WARNING:** Bypassing hooks skips quality gates that protect the shared
> history. Only use these escapes when you have a clear, specific reason and
> you will fix the underlying issue immediately afterward.

Skip pre-commit and commit-msg hooks:

```sh
git commit --no-verify -m "chore(ci): emergency fix — hooks broken by toolchain upgrade"
```

Skip pre-push hooks:

```sh
git push --no-verify
```

**Acceptable reasons to bypass:**

- Toolchain is broken (e.g. `cargo fmt` crashes on a Rust nightly regression)
  and you need to commit a fix for the toolchain issue itself.
- You are committing a work-in-progress branch that will never be merged
  without a passing CI run.
- CI has independently validated the change and a hook check is failing due
  to a hook bug, not a code bug.

**Not acceptable:**

- Bypassing because a check is "probably fine".
- Bypassing to meet a deadline without fixing the underlying issue.
- Bypassing the forbidden-term scan on files that genuinely contain
  forbidden terms.
