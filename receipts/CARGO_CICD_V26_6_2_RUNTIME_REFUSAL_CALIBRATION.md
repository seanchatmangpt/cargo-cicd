# Receipt: RUNTIME_REFUSAL_CALIBRATION
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

The `Wasm4pmShell` adapter in `src/integrations/` and the test suite in `tests/wasm4pm_refusal_cases.rs` establish and verify refusal calibration: wpm correctly distinguishes parseable XES (accepted, exit=0, quality verdict returned) from garbage binary input (refused, exit=1, UTF-8 error). Tests cover: missing file → refuse, empty XES → refuse, corrupted XML → refuse, binary garbage → refuse (UTF-8 error). Evidence invariants E1 (no self-certification), E2 (evidence required before adjudication), and E3 (blocked is first-class) are enforced at the structural level. All 7 refusal tests pass.

## wasm4pm adjudication

Gates session observations:
- `wpm audit <valid_xes>`: exit=0, output contains "DECEPTIVE" (quality judgment, not parse error)
- `wpm audit <binary_garbage>`: exit=1, stderr: "stream did not contain valid UTF-8"
- `wpm audit <missing_file>`: exit=1 (file not found)

Refusal calibration confirmed: wpm refuses binary content unconditionally; accepts parseable XES and returns quality verdict.
