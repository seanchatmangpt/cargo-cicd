---
name: evidence-audit
description: Explains and runs the process-evidence adjudication pipeline for cargo-cicd. Checks that XES event files exist under target/cargo-cicd/evidence/, invokes the receipt doctor via `cargo cicd evidence doctor`, runs `cargo cicd status audit`, and interprets the Accept or Refuse verdict from the wpm oracle. Notes that a missing wpm oracle exits non-zero with a BLOCKED diagnostic (expected in local dev environments). Use when the user says "audit evidence", "check receipts", "adjudicate", or asks whether process evidence was accepted.
---

# Evidence Audit — Process-Evidence Adjudication

cargo-cicd emits process events in XES (XML Event Stream) format to
`target/cargo-cicd/evidence/`. Release closure requires the wpm oracle to
adjudicate that evidence and return an **Accept** verdict. This skill walks
through each step.

---

## Step 1 — Confirm Evidence Files Exist

Check that the evidence directory is present and contains at least one XES file.

```
target/cargo-cicd/evidence/
```

Use the Glob tool to list files matching `target/cargo-cicd/evidence/**/*.xes`.

- If the directory is empty or absent: the commands that should have emitted evidence were never run. Report **BLOCKED — no evidence files found** and advise the user to run `cargo cicd status show` or another command that emits process events, then re-audit.
- If XES files are present: list them with their sizes and proceed to Step 2.

---

## Step 2 — Receipt Doctor (wpm oracle adjudication)

Run the receipt doctor verb, which internally calls:

```
wpm receipt doctor --format json --strict <receipt.json>
```

via the CLI surface:

```
cargo cicd evidence doctor
```

Capture the full output. Look for:

- `"verdict": "Accept"` — the receipt conforms to the process model.
- `"verdict": "Refuse"` — the receipt has structural or semantic violations. Show the `reasons` array from the JSON output.
- Non-zero exit with a `BLOCKED` or `oracle unavailable` diagnostic — this means the wpm binary at `/Users/sac/wasm4pm/target/release/wpm` was not found or returned an unexpected error. **This is expected in local development environments where wpm is not installed.** Note the diagnostic and continue to Step 3.

---

## Step 3 — Status Audit (XES health check)

Run the audit verb, which internally calls `wpm audit <file.xes>` on each
evidence file:

```
cargo cicd status audit
```

Capture the full output. Interpret the result:

- All files report `OK` or `Accept` — evidence is structurally sound.
- Any file reports an error or `Refuse` — show the file path and the error detail.
- Non-zero exit with a `BLOCKED` or `oracle unavailable` diagnostic — same as Step 2: expected locally when wpm is absent. Note the diagnostic.

---

## Step 4 — Interpret the Combined Verdict

| Condition | Interpretation |
|-----------|---------------|
| Both `evidence doctor` and `status audit` return Accept | **Release gate: PASSED** — evidence adjudicated. |
| Either returns Refuse | **Release gate: FAILED** — show the refusal reasons; the emitting command or receipt schema needs correction. |
| Either exits non-zero with BLOCKED / oracle unavailable | **Release gate: DEFERRED (local)** — wpm oracle is absent. This is normal locally. CI must have wpm present for a closing verdict. |
| Evidence directory empty or absent | **Release gate: BLOCKED** — no evidence was emitted. Run a command that emits process events first. |

---

## Step 5 — Action Guidance

**If Accept:** The evidence gate is satisfied. Proceed with the release checklist
(`/release-checklist`) if you have not already.

**If Refuse:** Examine the `reasons` array from the receipt doctor output. Common
causes:

- Missing required XES trace attributes.
- Event timestamps out of causal order.
- Receipt schema version mismatch.

Fix the emitting code in `src/evidence.rs` or the adapter that writes receipts,
re-run the command that emits evidence, then re-run this skill.

**If BLOCKED (oracle absent):** Verify `wpm` is built and present at
`/Users/sac/wasm4pm/target/release/wpm`. If running in CI, ensure the wpm
binary is installed in the CI environment before the evidence-gate test step.
A non-zero exit with a BLOCKED diagnostic is expected and non-fatal locally,
but the CI pipeline must resolve it before a release is cut.
