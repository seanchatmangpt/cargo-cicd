# Convergence Finding: `wpm audit events.xes` DECEPTIVE Verdict

**Date:** 2026-07-06
**Trigger:** A prior convergence pass ran `wpm audit target/cargo-cicd/evidence/events.xes -v`
against cargo-cicd's own freshly-emitted evidence and got:

```
Audit Verdict:            DECEPTIVE
Fitness Score:            0.6457
Precision Score:          0.2331
Total Traces Audited:     1
Fitting Traces:           0
Deviating Traces:         1
```

This document records the root-cause investigation, what turned out to be genuinely
wrong (a cargo-cicd bug, now fixed), what turned out to be **not** wrong (both
cargo-cicd's emission vocabulary and wasm4pm's conformance-checking logic), and the
honest before/after state.

## TL;DR

- **cargo-cicd's XES emission is correct.** The activity vocabulary, ordering, and
  filtering (`emit_xes_filtered` / `append_events`) match wasm4pm's expectations
  exactly. Proven empirically below: a clean trace from `cargo cicd pipeline run`
  audits as **TRUTHFUL, fitness 1.0000, precision 1.0000** with the unmodified `wpm`
  binary.
- **wasm4pm's conformance-checking logic (`simd_token_replay`) is not weakened or
  wrong.** It correctly reports low fitness for a highly-interleaved, ad-hoc command
  history — that is precisely what it is designed to do.
- **The DECEPTIVE verdict came from auditing the wrong artifact.** `events.xes` is,
  by cargo-cicd's own documented design (`ProcessEvent::trace_class`,
  `process/cicd-process.powl.json` → `trace_classes.live_workspace`), the
  **ambient, accumulated history of individual ad-hoc command invocations** across an
  entire dev/chat session — explicitly documented as expected to show
  `"target_verdict": "VARIANCE"`, "expected and honest," not `TRUTHFUL`. Running the
  strict admission-gate check against that ambient log will essentially always score
  low, because `wpm`'s Petri net is derived from the log's own directly-follows
  relations (see "Why ambient traces score low" below) and a real dev session
  re-invokes `status:show`, `target:show`, etc. in many different orders.
- **A genuine cargo-cicd bug was found and fixed**, and it is more serious than the
  artifact-selection issue above: cargo-cicd's own interpretation of `wpm audit`
  output was silently **blind to the DECEPTIVE verdict** — it would have reported
  `Accept` for the exact 0.6457/DECEPTIVE trace in the trigger report. See
  "Root cause: `infer_verdict` never recognized `wpm audit`'s vocabulary" below.

## Investigation

### 1. What cargo-cicd actually emits

`src/evidence.rs::append_events` (the real path invoked by every noun/verb) writes
`events.xes` via `emit_xes_filtered`, which deliberately:

- drops `"start"` lifecycle events (only `"complete"` is written — mixing both
  duplicates activity names in the DFG and corrupts token-replay fitness, per the
  function's own doc comment), and
- drops any activity not in `DECLARED_ACTIVITIES` (10 activities declared in
  `process/cicd-process.powl.json`; e.g. `git:status` is dropped as "noise").

Verified directly against the on-disk evidence: the ambient `events.xes` that
produced the DECEPTIVE verdict contained exactly `228 complete − 33 git:status =
195` events — precisely matching what the filter is documented to do. **This is
working as designed, not a bug.**

### 2. What wasm4pm's `audit` command actually checks

`wpm audit <file>` (`wasm4pm-cli/src/commands/audit.rs` → `simd_token_replay`) does
**not** load `process/cicd-process.powl.json` or any other external reference model.
It is self-referential: it builds a directly-follows graph (DFG) from the *same* log
being audited (aggregating over all events/traces in the file), converts that DFG
into a Petri net, and token-replays each trace against it. Conformance here really
measures "how internally consistent is the ordering in this log," not "does this log
match a declared reference process."

A consequence of this design (confirmed by reading
`wasm4pm/src/simd_token_replay.rs::SimdPetriNet::from_dfg` /
`replay_trace`): each Petri-net transition is labeled only by its **source**
activity, not by the `(source, destination)` pair. When an activity has more than
one distinct successor across the log (e.g. `status:show` followed at different
points by `target:show`, `test:changed`, `target:prune`, ...), replay can only
"guess" one candidate destination per firing. Wrong guesses cascade into `missing`/
`remaining` token counts. This means **any log with real branching/interleaving —
which is exactly what an ambient, multi-command dev session looks like — will score
below the ambient-log's own documented `VARIANCE` expectation, sometimes landing in
the `DECEPTIVE` band.** This is a known modeling limitation of DFG-derived
single-Petri-net replay, not something to "fix" by weakening the checker — it is
functioning as an honest (if blunt) process-conformance instrument.

### 3. Confirming which side is right: run a clean trace through the exact same tools

`cargo cicd pipeline run` (`src/nouns/pipeline.rs` → `legacy_nouns/pipeline.rs`,
the currently-registered implementation) already exists specifically to produce a
single, cleanly-ordered trace tagged `trace_class = "pipeline_run"` (as opposed to
the default `"live_workspace"`). Running it fresh and then running the **identical**
command from the trigger report against its output:

```
$ WPM_BIN=/Users/sac/wasm4pm/target/release/wpm cargo-cicd pipeline run
$ wpm audit target/cargo-cicd/evidence/events.xes -v

Audit Verdict:            TRUTHFUL
Fitness Score:            1.0000
Precision Score:          1.0000
Total Traces Audited:     1
Fitting Traces:           1
Deviating Traces:         0
```

Full conformance. This directly confirms cargo-cicd's activity vocabulary
(`status:show`, `target:show`, `test:changed`, `trybuild:changed`,
`workspace:doctor`, `publish:run`, `status:audit`, `evidence:audit`) and ordering
are exactly what wasm4pm's model expects — **the vocabulary/ordering side of this
was never broken.**

For contrast, replaying the ambient scenario at small scale (9 ad-hoc commands run
in realistic, non-strict order) reproduces the same phenomenon as the original
report, just smaller:

```
Audit Verdict:            DECEPTIVE
Fitness Score:            0.6136
Precision Score:          0.5000
```

So: neither cargo-cicd's emission nor wasm4pm's checker is defective. The trigger
report's finding was real (DECEPTIVE is what you get auditing `events.xes`) but the
framing — "cargo-cicd's evidence doesn't conform" — was imprecise, because
`events.xes` in ambient/live_workspace form was never meant to be the admission-gate
artifact; that role is `pipeline run`'s dedicated clean trace.

## Root cause of the actual bug: `infer_verdict` never recognized `wpm audit`'s vocabulary

This is the part that matters most, and it is a genuine cargo-cicd defect, now
fixed.

`wpm audit` always **exits 0** and reports one of three fitness bands on its own
`"Audit Verdict:"` line — `TRUTHFUL` (≥0.95), `VARIANCE` (0.70–0.95), or
`DECEPTIVE` (<0.70) — regardless of which band the trace lands in
(`wasm4pm-cli/src/commands/audit.rs::print_audit_report`). None of those words
contain the substrings `"fail"`, `"error"`, or `"warn"`.

`src/integrations/wasm4pm_shell.rs::Wasm4pmShell::audit()` (called from
`WpmEvidenceOracle::audit_xes` in `src/evidence.rs`, the function that produced the
DECEPTIVE report in the trigger) previously routed through the **generic**
`infer_verdict(stdout, stderr, exit_ok)` heuristic shared by every other wpm
shell-out (`lean`, `doctor`, `spc status`, ...):

```rust
fn infer_verdict(stdout: &str, _stderr: &str, exit_ok: bool) -> WpmVerdict {
    if !exit_ok { return WpmVerdict::Fail; }
    let lower = stdout.to_lowercase();
    if lower.contains("fail") || lower.contains("error") || lower.contains("warn") {
        WpmVerdict::Warn
    } else {
        WpmVerdict::Pass // exit 0 with neutral output = pass
    }
}
```

Since `wpm audit` exits 0 and its report text never contains "fail"/"error"/"warn"
— **including for a DECEPTIVE result** — this heuristic classified *every* audit
result as `WpmVerdict::Pass`, which `WpmEvidenceOracle::audit_xes` then maps to
`ExpectedWpmVerdict::Accept`. In other words: **the exact 0.6457/DECEPTIVE trace
from the trigger report would have been silently reported as `Accept` by
cargo-cicd's own evidence gate**, had anything in cargo-cicd actually called
`audit_xes` on it, completely defeating the purpose of having wasm4pm as an
external, non-self-adjudicating oracle (violates the spirit of invariant **E1** —
"cargo-cicd never adjudicates itself" — because a hard-coded interpretation bug
made it functionally toothless).

## Fix

`src/integrations/wasm4pm_shell.rs`:

- Added `infer_audit_verdict(stdout, exit_ok)`, a dedicated parser for `wpm audit`'s
  specific three-tier vocabulary: `"deceptive"` → `Fail`, `"variance"` → `Warn`
  (matches cargo-cicd's own documented expectation that ambient `live_workspace`
  traces show honest variance, not full conformance), `"truthful"` → `Pass`, with a
  fallback to the old generic heuristic only if none of those words appear (protects
  against a future wpm report-format change silently going unrecognized again).
- `Wasm4pmShell::audit()` now uses `infer_audit_verdict` instead of the generic
  `invoke()`/`infer_verdict` path.
- Corrected stale doc comments in the same file claiming `wpm audit` accepts an
  OCEL 2.0 JSON input — the current `wpm audit` CLI only accepts XES and explicitly
  bails (with a redirect message) if it detects an OCEL file.
- Added 5 unit tests (`src/integrations/wasm4pm_shell.rs::tests`) pinning
  TRUTHFUL→Pass, VARIANCE→Warn, DECEPTIVE→Fail (using the literal 0.6457 DECEPTIVE
  report text from the trigger), non-zero-exit→Fail, and the unrecognized-format
  fallback.
- Updated `tests/wasm4pm_evidence_gate.rs::evidence_gate_status_show_accepted`: its
  single-event fixture had no directly-follows edges for wpm's Petri net to
  replay against, which — now that the verdict is parsed honestly — surfaces a
  separate, genuine wasm4pm edge case (a lone event with zero edges scores
  `DECEPTIVE`/`0.0000`, since `consumed`/`produced` end up asymmetric rather than
  both zero). This test was previously "passing" only because of the `infer_verdict`
  bug above, not because the case was actually conformant. Fixed by using a minimal
  *linear* 3-activity trace, which is the smallest input that gives the Petri net
  real edges to replay and correctly audits `TRUTHFUL`/`1.0000`.

## Before / after

| Scenario | Command | Verdict (raw `wpm audit -v` text — unaffected by the fix, wasm4pm's own algorithm is unchanged) | cargo-cicd's own interpretation before fix | cargo-cicd's own interpretation after fix |
|---|---|---|---|---|
| Ambient `live_workspace` trace (original trigger, and reproduced at small scale) | `wpm audit events.xes -v` | `DECEPTIVE`, fitness 0.6136–0.6457 | `WpmVerdict::Pass` → `ExpectedWpmVerdict::Accept` (bug: silently wrong) | `WpmVerdict::Fail` → `ExpectedWpmVerdict::Refuse` (correct) |
| Clean `pipeline_run` trace (`cargo cicd pipeline run`) | `wpm audit events.xes -v` | `TRUTHFUL`, fitness 1.0000, precision 1.0000 | `WpmVerdict::Pass` → `Accept` (happened to be correct here) | `WpmVerdict::Pass` → `Accept` (unchanged, still correct) |

The raw wasm4pm oracle output for the ambient trace is **unchanged and should stay
unchanged** — auditing an intentionally noisy ambient log and getting a low score is
the checker doing its job. What changed is that cargo-cicd's own code now actually
listens to that result instead of overriding it to a pass.

## Secondary finding (documented, not fixed — separate command path, out of scope for this pass)

`cargo cicd status audit` / `evidence audit` (`src/nouns/status.rs::run_audit`)
audits `events.ocel.json` via `Wasm4pmShell::audit()`, but the current `wpm audit`
CLI **only accepts XES** and explicitly bails for OCEL input:

```
OCEL 2.0 format detected (...).
The wpm audit command currently supports XES event logs (IEEE 1849).
To audit an OCEL log, flatten it first: wpm run --algorithm dfg --format json ...
```

Separately, `legacy_nouns/pipeline.rs`'s own self-adjudication step calls
`wpm receipt verify-ocel2` against a bare OCEL 2.0 log built by `emit_ocel_fresh` —
but `receipt verify-ocel2` expects a *receipt document* with embedded
expected/observed OCEL logs, not a plain event log, so it currently fails
structurally every time a real `wpm` binary is present:

```
=== wpm receipt verify-ocel2 ===
  [FAIL] Receipt has missing or invalid OCEL 2.0 structures.
```

This means `cargo cicd pipeline run`'s built-in self-check currently **always
REFUSEs** (and the command exits non-zero) whenever a real `wpm` binary is
available — confirmed by running it with `WPM_BIN` set. `process/cicd-process.powl.json`'s
`admission_gate.command` field (`"wpm audit audit-events.xes"`) is also stale: the
code actually writes `audit-events.ocel.json`, a different format, at a different
path. None of this affects the XES-based `wpm audit` root cause investigated above,
so it's recorded here rather than fixed in this pass — it needs its own decision
(either wire `pipeline run`'s self-check to `wpm audit` against a filtered XES
export, matching what's proven to work above, or fix `receipt verify-ocel2`'s input
contract) rather than a quick patch.

## Files changed

- `src/integrations/wasm4pm_shell.rs` — `infer_audit_verdict`, `audit()` rewire, doc fixes, unit tests.
- `tests/wasm4pm_evidence_gate.rs` — realistic multi-event fixture for `evidence_gate_status_show_accepted`.
- `docs/convergence/FLEET_STANDING_CONVERGENCE.md` — this document.

---

## Final closure

**Date:** 2026-07-06

This section closes out the session: untracked-item cleanup, the XES conformance
investigation above, and a dead-code sweep, verified together and committed.

### What was resolved in this pass

1. **Untracked scratch artifacts** — `clippy_output.txt` (raw scratch `cargo clippy`
   stderr from an abandoned attempt), `crates/cargo-cicd-bench-utils/` (an orphaned
   WIP crate exporting `SequenceGenerator`/`TempWorkspace` with zero consumers
   anywhere in the workspace), and `ocel/` (a stray directory duplicating the real
   evidence locations under `target/cargo-cicd/evidence/` and `.cargo-cicd/ocel/`)
   were confirmed to be genuinely untracked, unreferenced files and deleted directly
   from the working tree. None of the three was ever committed to git, so there is
   no corresponding deletion commit — `git log` has no record of them to begin with.

2. **XES conformance investigation** — see the full writeup above. Root cause: `wpm
   audit`'s `DECEPTIVE` verdict was correct wasm4pm behavior on a genuinely noisy
   ambient trace; the actual bug was in cargo-cicd's own `infer_audit_verdict`,
   which didn't recognize `wpm audit`'s verdict vocabulary and silently mapped it to
   `Pass`. Fixed in `src/integrations/wasm4pm_shell.rs`. A secondary, unrelated
   finding (the `receipt verify-ocel2` / OCEL-vs-XES input-contract mismatch on
   `pipeline run`'s self-check) was documented but explicitly left unfixed — it is
   a separate command path requiring its own design decision.

3. **Dead-code sweep** — with `#![allow(dead_code, unused_imports)]` already
   removed from `src/main.rs` (ERRC "First wave" item 1), a systematic sweep of the
   warnings it had been hiding: deleted `src/adapters/cicd_toml_writer.rs`,
   `src/adapters/fs.rs`, `src/adapters/manifest_parser.rs`,
   `src/autonomic/signals.rs`, `src/policies/diagnostics_bridge.rs`, and the entire
   `src/state/` module (10 files) — all first-generation duplicates or orphaned
   scaffolding with no consumer once `EngineState`/the second-generation adapters
   were confirmed as the only live path. Trimmed dead functions/fields/imports
   across `certification/`, `ui/`, `policies/`, `autonomic/`, `engine/`,
   `cicd_toml.rs`, `integrations/mod.rs`, and two test files. Dead-code warnings on
   the default build went from 115 to 0 (the one remaining warning,
   `unused_mut` in `src/legacy_nouns/pipeline.rs:63`, predates this session and is
   unrelated to dead-code work). This also directly addresses ERRC "First wave"
   items 1–2 (`docs/vision/ERRC_REVIEW.md`).

### Final verification matrix

| Check | Result |
|---|---|
| `cargo build` | Pass — 1 pre-existing `unused_mut` warning only (`src/legacy_nouns/pipeline.rs:63`), 0 errors |
| `cargo build --all-features` | Pass — 63 warnings, all feature-gated dead code outside this pass's scope (confirmed pre-existing at 177 warnings on the same all-features build before this session's changes; not a regression, and not fully addressed since the dead-code sweep targeted the default-feature build) |
| `cargo test` (full suite) | 175 passed, 1 pre-existing failure, 0 regressions (see below) |
| `cargo test --test invariants -- --nocapture` | 4/4 pass, including `invariant_public_boundary_no_forbidden_terms_in_all_help` (CLI `--help` output, not docs content) |
| `cargo run -- --version` | `cargo-cicd 26.6.30` |
| `cargo run -- --help` | Renders full noun list correctly |
| `cargo run -- standing refresh` | `standing refresh: 10 artifact(s) -> ./target/praxis-standing/standing.json`, exit 0 |
| `cargo run -- standing verify` | `standing verify: 0 drifted artifact(s)`, exit 0 |

**Pre-existing failure (confirmed, not a regression):** `tests/ggen_customization_guard.rs::no_forbidden_terms_in_public_docs`
fails because several files under `docs/how-to/` and `docs/reference/` (created by
an earlier "docs: reorganize into Diataxis structure" / "docs: rewrite README"
commit already on `main` before this session started) contain the forbidden terms
(`ALIVE`, `Nehemiah`, `Inspection Gate`, `Field8`, `Instinct8`, `Cargo Court`,
`Truex`, `CONSTRUCT8`) as narrative/example text. Confirmed via `git stash` that
this failure is present identically with none of this session's changes applied.
This is a separate, pre-existing docs-content problem, distinct from the invariant
test (which only scans CLI `--help` output and passes). Left unfixed — it needs a
human decision on rewriting or exempting the specific example passages in those
docs files, out of scope for this closure pass.

### Overall project state

**Genuinely done in this pass:**
- Untracked scratch cleanup, verified with zero remaining references.
- The XES/`wpm audit` conformance bug: root-caused, fixed, and covered by a
  realistic-fixture regression test.
- Dead-code elimination on the default build surface (115 → 0 warnings), including
  removal of an entire duplicate `src/state/` module.
- Full verification matrix run clean (build, all-features build, full test suite,
  invariants, version/help, `standing refresh`/`standing verify`) with the one
  pre-existing docs-content test failure identified as out-of-scope and unrelated.

**Left for a human to decide:**
- The `no_forbidden_terms_in_public_docs` failure — whether to rewrite the
  offending doc passages, exempt them, or relax the test's scope to match
  `invariant_public_boundary_no_forbidden_terms_in_all_help`'s (CLI-output-only)
  intent.
- The secondary XES finding: `pipeline run`'s self-check calling
  `wpm receipt verify-ocel2` against a bare OCEL log it structurally rejects, and
  `process/cicd-process.powl.json`'s stale `admission_gate.command` path — needs a
  decision on which command path (`wpm audit` against filtered XES, or fixing
  `receipt verify-ocel2`'s contract) is the intended one going forward.
- ~37 remaining lower-priority findings in `docs/vision/ERRC_REVIEW.md` (5
  Eliminate, 12 Reduce, 16 Raise, 4 Create) beyond the "First wave" items already
  addressed by this session's dead-code sweep — notably the public API surface
  reduction (`src/lib.rs`), defaulting the parallel target-scan path, and adding
  one-line `--help` descriptions for every noun. None of these are regressions or
  blocking; they are prioritized improvement candidates for a future pass.
- The 63 remaining `--all-features` dead-code warnings (feature-gated code paths
  not exercised by the default build) — a natural follow-on to this session's
  default-build dead-code sweep, using the same `cargo build --all-features`
  signal now that it's been established as a working detection method.
