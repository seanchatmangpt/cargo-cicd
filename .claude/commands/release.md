# /release — cargo-cicd Release Gate

Trigger: user says "release", "ship", or asks to publish v26.6.2.
Action: execute all steps in order; stop on first failure.

---

## Step 1 — Git clean

```bash
git status
```

FAIL if: any uncommitted, staged, or untracked files exist.
Fix: commit, stash, or `git clean -fd` (confirm before destructive).

---

## Step 2 — Full test suite

```bash
cargo make test
```

FAIL if: any test exits non-zero.
Fix: `cargo test --test <suite> <function>` → fix code → re-run `cargo make test`.

---

## Step 3 — Invariants

```bash
cargo test --test invariants
```

FAIL if: any of the 7 invariants fail, especially `invariant_public_boundary_no_forbidden_terms_in_all_help`.

Forbidden terms (never in CLI output):
`ALIVE` · `Inspection Gate` · `wall` · `Nehemiah` · `Field8` · `Instinct8` · `Cargo Court` · `AGI` · `Truex` · `CONSTRUCT8`

Fix:
```bash
cargo run -- <noun> <verb> --help | grep -E "ALIVE|wall|Nehemiah|Field8|Instinct8|AGI|Truex|CONSTRUCT8"
rg "<term>" src/
# replace with approved public term
cargo test --test invariants
```

---

## Step 4 — Feature flag compilation

```bash
cargo build --features autonomic,wasm4pm,contrib
```

FAIL if: any compile error under feature-gated code.
Fix: verify `#[cfg(feature = "...")]` gates and optional dep wiring in `Cargo.toml`.

---

## Step 5 — Evidence gate

```bash
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
```

| Outcome | Meaning | Action |
|---------|---------|--------|
| `Accept` | oracle accepted evidence | continue |
| `Blocked` | wpm unavailable | install wpm, re-run |
| `Refuse` on happy-path | OCEL format wrong | inspect `target/cargo-cicd/evidence/`, fix serialization |
| `Accept` on mutated evidence | oracle not enforcing integrity | investigate wpm config |

Evidence uses OCEL 2.0 (not XES). Import from `wasm4pm_compat`, never hand-roll structs.
Do NOT call `wpm audit` on `.xes` files for new code.

```bash
which wpm && wpm --version   # verify oracle available
wpm audit target/cargo-cicd/evidence/<file>.ocel.json
```

---

## Step 6 — Receipt validation

```bash
wpm receipt doctor --format json --strict receipts/*.json
```

FAIL if: any receipt is malformed or rejected.
Fix: re-run the emitting verb to regenerate the receipt. Do not delete receipts.
If `receipts/` empty: run `status show`, `workspace doctor`, `publish run` first.

---

## Step 7 — README currency

```bash
ls -la README.md ontology/cargo-cicd-capabilities.ttl
git log --oneline -5 -- README.md
git log --oneline -5 -- ontology/cargo-cicd-capabilities.ttl
```

FAIL if: ontology has commits newer than README.
Fix:
```bash
ggen
git diff README.md
# if changed: stage and commit before proceeding
```

---

## Step 8 — CHANGELOG

```bash
head -40 CHANGELOG.md
```

FAIL if: no entry for v26.6.2 exists.
Fix: prepend to `CHANGELOG.md`:
```markdown
## v26.6.2 — 2026-06-21

### Added
- ...

### Fixed
- ...
```

---

## Step 9 — Version check

```bash
grep '^version' Cargo.toml | head -3
grep '26\.6\.2' src/main.rs
```

FAIL if: either location does not read `26.6.2`.
Fix: update `Cargo.toml` and `src/main.rs`, then `cargo build` to confirm propagation.

---

## Step 10 — Release commit

```bash
git add CHANGELOG.md README.md
git commit -m "chore(release): v26.6.2 evidence gate pass"
git status   # must be clean
```

Do NOT `--amend` previous commits.

---

## Step 11 — Tag

```bash
git tag -a v26.6.2 -m "Release v26.6.2 — evidence adjudicated by wasm4pm"
git show v26.6.2
```

FAIL if: tag is lightweight (missing `-a`).
Fix:
```bash
git tag -d v26.6.2
git tag -a v26.6.2 -m "Release v26.6.2 — evidence adjudicated by wasm4pm"
```

---

## Step 12 — Push

```bash
git push origin main --tags
```

| Failure | Action |
|---------|--------|
| main behind remote | `git pull --rebase origin main` → re-run all tests → push |
| tag already on remote | investigate duplicate run; do not force-push tags |
| force-push to main | BLOCKED — never allowed |

---

## Verify

```bash
git log --oneline -3
git tag -l | grep v26
```
