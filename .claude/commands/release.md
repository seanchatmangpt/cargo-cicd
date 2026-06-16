# /release — cargo-cicd Release Gate

Run the full release workflow for cargo-cicd. Current version: **v26.6.2**.

Execute every step in order. If any step fails, stop and fix before continuing. Do not tag or push until all gates are green.

---

## Step 1 — Pre-flight: Git must be clean

```bash
git status
```

**What to check:** The working tree must be completely clean — no uncommitted changes, no untracked files, no staged modifications.

**Failure means:** You have unsaved work. Either commit it, stash it, or discard it before releasing.

**How to fix:**
- Commit outstanding changes: `git add <files> && git commit -m "..."`
- Stash if temporary: `git stash`
- Discard untracked files: `git clean -fd` (destructive — confirm first)

---

## Step 2 — Run all tests

```bash
cargo make test
```

**What to check:** All test suites must exit 0. Watch for failures in any tier — unit tests, integration tests, evidence gate, or autonomic policies.

**Failure means:** A regression exists. The release must not proceed.

**How to fix:**
- Read the failing test output carefully — it will name the test file and function
- Run the specific failing test in isolation: `cargo test --test <suite> <function_name>`
- Fix the underlying code, not the test (unless the test itself is wrong)
- Re-run `cargo make test` to confirm all pass

---

## Step 3 — Invariants: no forbidden terms in public output

```bash
cargo test --test invariants
```

**What to check:** All 7 public boundary invariants must pass. The critical one is `invariant_public_boundary_no_forbidden_terms_in_all_help`, which scans every `--help` output for forbidden terms.

Forbidden terms (must never appear in any CLI output):
`ALIVE`, `Inspection Gate`, `wall`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`

**Failure means:** A forbidden internal term leaked into user-facing output. This is a hard public boundary violation.

**How to fix:**
1. Identify which noun/verb leaked the term:
   ```bash
   cargo run -- <noun> <verb> --help | grep -E "ALIVE|Inspection Gate|wall|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8"
   ```
2. Search source for the term: `rg "<term>" src/`
3. Replace with the approved public alternative
4. Re-run: `cargo test --test invariants`

---

## Step 4 — Feature flag compilation

```bash
cargo build --features autonomic,wasm4pm,contrib
```

**What to check:** All optional feature combinations must compile without errors. This catches feature-gated code that silently broke.

**Failure means:** A feature-gated module has a compile error. The release binary would be broken for users who enable those features.

**How to fix:**
1. Note the exact compiler error and which feature it implicates
2. Check that the failing code is correctly gated: `#[cfg(feature = "autonomic")]`
3. Check that any new dependencies are declared as optional and wired to the feature:
   ```toml
   [dependencies]
   some_crate = { version = "1.0", optional = true }
   [features]
   autonomic = ["process-data", "some_crate"]
   ```
4. Fix the compile error and re-run this step

---

## Step 5 — Evidence gate

```bash
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
```

**What to check:**
- `wasm4pm_evidence_gate`: Happy-path evidence flows must produce `Accept` verdicts from the wpm oracle
- `wasm4pm_evidence_mutation`: Corrupted or mutated evidence must produce `Refuse` verdicts

**Failure means:**
- If wpm is unavailable, tests will report `Blocked` — this is expected offline but blocks a release
- If wpm is available and returns `Refuse` on happy-path evidence, the XES format is wrong
- If wpm accepts mutated evidence, the oracle is not enforcing integrity

**How to fix (oracle unavailable):**
```bash
which wpm
wpm --version
```
If not found, install wasm4pm and add it to PATH, then re-run.

**How to fix (format mismatch):**
1. Inspect a failing XES file: `ls -la target/cargo-cicd/evidence/`
2. Manually audit: `wpm audit target/cargo-cicd/evidence/evt-*.xes`
3. Compare XES structure against the expected format in `src/evidence.rs`
4. Fix the serialization and re-run

---

## Step 6 — wpm receipt validation

```bash
wpm receipt doctor --format json --strict receipts/*.json
```

**What to check:** All receipt artifacts in `receipts/` must be valid and accepted by the oracle under strict mode.

**Failure means:** A receipt is malformed, tampered, or references evidence that no longer matches. Releases require clean receipts.

**How to fix:**
1. Identify which receipt failed: `wpm receipt doctor --format json --strict receipts/<name>.json`
2. If the receipt is stale, regenerate it by re-running the relevant verb
3. If the receipt is corrupt, investigate whether evidence was mutated between emission and signing
4. Do not delete receipts — fix the underlying cause

**If `receipts/` is empty or missing:** This means no receipts have been generated yet. Run the evidence-emitting verbs (`status show`, `workspace doctor`, `publish run`) and re-audit.

---

## Step 7 — README currency check

**What to check:** Confirm `ggen` has been run and `README.md` reflects the current ontology. The README is generated — manual edits to generated sections will be overwritten.

```bash
# Check when README was last modified vs the ontology
ls -la README.md ontology/cargo-cicd-capabilities.ttl
git log --oneline -5 -- README.md
git log --oneline -5 -- ontology/cargo-cicd-capabilities.ttl
```

**If the ontology has commits more recent than the README:**
```bash
ggen
git diff README.md
```

Review the diff. If content changed, the README was stale. Stage and commit the update before proceeding.

**Failure means:** Users will see outdated command references. Generated docs must match the live ontology.

---

## Step 8 — CHANGELOG update

**What to check:** `CHANGELOG.md` must include an entry for v26.6.2 with a summary of changes in this release.

```bash
head -40 CHANGELOG.md
```

**Failure means:** The release has no documented history. Downstream users and auditors cannot determine what changed.

**How to fix:** Add an entry at the top of `CHANGELOG.md`:
```markdown
## v26.6.2 — 2026-06-16

### Added
- ...

### Fixed
- ...

### Changed
- ...
```

Commit the CHANGELOG update as part of the release commit in Step 10.

---

## Step 9 — Version check

**What to check:** The version string must read `26.6.2` in both locations.

```bash
grep '^version' Cargo.toml | head -3
grep '26\.6\.2' src/main.rs
```

**Failure means:** The binary will report the wrong version. Users and oracle receipts will reference a mismatched version.

**How to fix:**
- In `Cargo.toml`: `version = "26.6.2"`
- In `src/main.rs`: find the version constant or `.version("26.6.2")` call and update it
- After changing `Cargo.toml`, run `cargo build` to verify the version propagates

---

## Step 10 — Final release commit

Stage any outstanding release artifacts (CHANGELOG, README if regenerated):

```bash
git add CHANGELOG.md README.md
git commit -m "chore(release): v26.6.2 evidence gate pass"
```

**What to check:** `git status` after the commit must be clean again.

**Do not use `--amend`** on any previous commit. Create a new commit.

---

## Step 11 — Tag the release

```bash
git tag -a v26.6.2 -m "Release v26.6.2 — evidence adjudicated by wasm4pm"
```

**What to check:** The tag must be annotated (`-a`), not lightweight. Annotated tags carry the tagger identity and timestamp, which is required for release provenance.

Verify: `git show v26.6.2`

**Failure means:** If you forget `-a`, delete the lightweight tag and recreate:
```bash
git tag -d v26.6.2
git tag -a v26.6.2 -m "Release v26.6.2 — evidence adjudicated by wasm4pm"
```

---

## Step 12 — Push to origin

```bash
git push origin main --tags
```

**What to check:** Both the commit and the tag must push without error. `--tags` pushes all local tags that aren't on the remote.

**Failure means:**
- If main is behind remote: `git pull --rebase origin main`, re-run all tests, then push
- If the tag already exists on remote: do not force-push tags. Investigate why — a duplicate tag means the release process was run twice
- Never `git push --force` on main

---

## Release Complete

After a successful push, verify on the remote:
```bash
git log --oneline -3
git tag -l | grep v26
```

The evidence gate has passed. wasm4pm has adjudicated the process artifacts. v26.6.2 is live.
