---
artifact: INVARIANTS
date: 2026-06-02
---

# cargo-cicd Invariants

These must hold everywhere, regardless of command, feature flag combination, or workspace shape.
Every test in this repository must trace to at least one invariant below.

---

## I1 — PublicBoundary

**Assertion:** No public-facing output surface (stdout, stderr, `--help`, README, generated
`cicd.toml` comments) contains any forbidden internal term.

**Forbidden terms:** any project-internal codename, covenant label, or working-name — such as
internal names for the evidence gate, code review process, policy engine, or other internal
subsystems — that is not part of the `cargo-cicd` public API.

**Test method:** Capture full stdout and stderr for every command under all feature flag
combinations. Scan output bytes for each forbidden term. Assert zero matches.

**Forbidden violation:** Leaking an internal term in any help text, diagnostic, or generated
file is a defect regardless of how it arrived there (templated string, dependency message,
panic text).

---

## I2 — PublishDeterminism

**Assertion:** If workspace state (source files, `Cargo.toml`, toolchain, feature flags) does
not change between two consecutive `publish run` invocations, the produced `cicd.toml` is
byte-identical.

**Test method:** Run `publish run` in a clean, unchanging TempDir. Capture `cicd.toml`. Run
again. Compare SHA-256 of both outputs. Assert equal.

**Forbidden violation:** Embedding a timestamp, random seed, or non-deterministic ordering in
`cicd.toml` without a stable sort or normalization pass.

---

## I3 — NoFalseClose

**Assertion:** `git close` must not report a successful close if the working tree contains any
dirty state (modified tracked files, untracked files, staged changes).

**Test method:** Create a workspace with at least one dirty file. Invoke `git close`. Assert
exit non-0 and that the output names the dirty state as the reason.

**Forbidden violation:** Reporting success from `git close` when `git status --porcelain`
produces any non-empty output.

---

## I4 — NoDestructiveDefault

**Assertion:** `target prune` must not delete any file from `target/` without explicit
confirmation or a `--yes` / `--force` flag. The default invocation must only report what would
be pruned and suggest a plan.

**Test method:** Invoke `target prune` without any confirmation flag on a workspace with an
over-limit `target/`. Assert exit 0, no files deleted (compare directory listing before and
after), and that stdout contains a suggestion.

**Forbidden violation:** Deleting any file during a default `target prune` invocation, even
incremental artifacts.

---

## I5 — NoFullTrybuildByDefault

**Assertion:** `trybuild changed` must not run the full fixture estate when only a subset of
fixtures has changed. It must run only the changed fixtures.

**Test method:** Construct a workspace with N fixtures (N >= 10). Mark exactly one as changed
(modify its `.rs` source). Invoke `trybuild changed`. Assert that only one fixture name appears
in the output plan and that N-1 fixture names do not appear.

**Forbidden violation:** Running all fixtures when `trybuild changed` is invoked, regardless of
how many changed. "Running all just to be safe" is a law violation.

---

## I6 — NoAssumedWasm4pmCapability

**Assertion:** When the `wasm4pm` feature is enabled, `cargo-cicd` must not exercise any
wasm4pm integration path unless the wasm4pm binary is present on PATH **and** the specific
capability (scan, file-exchange, shell-out) has been discovered and classified as available.

**Test method (binary absent):** Enable `wasm4pm` feature. Remove wasm4pm binary from PATH.
Invoke any command with wasm4pm integration. Assert PARTIAL signal, no capability assumed.

**Test method (binary present, exchange absent):** Provide mock wasm4pm binary that responds
to scan but not to file-exchange. Invoke command requiring exchange. Assert PARTIAL with
exchange capability named as absent.

**Forbidden violation:** Entering any wasm4pm code path when the corresponding capability has
not been positively classified as available.

---

## I7 — FeatureProjectionConsistency

**Assertion:** Enabling `process-data`, `autonomic`, or `wasm4pm` may add records, events, or
fields to command output, but must not change or contradict any fact that appears in the default
(no-feature) output.

**Test method:** Run a command with no features. Capture output facts (exit code, key status
lines). Run the same command with each feature flag enabled. Assert that every fact present in
the no-feature output is also present and identical in the feature-enabled output. Assert that
no feature flag causes a previously-passing command to fail or a previously-reported fact to
disappear.

**Forbidden violation:** A feature flag that changes the exit code, removes a status line, or
inverts a reported fact compared to the default run on the same workspace state.
