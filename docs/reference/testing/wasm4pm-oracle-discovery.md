# wasm4pm Oracle Discovery

**Date:** 2026-06-02
**Binary:** `/Users/sac/wasm4pm/target/release/wpm`
**Version:** `wpm 26.5.29`

---

## Confirmed Commands

| Command | Exit Code | Verdict / Summary |
|---|---|---|
| `wpm --version` | 0 | `wpm 26.5.29` |
| `wpm doctor` | 0 | Checks rustc, wasm-pack, Cargo.toml, src/; WARN if `.wasm4pm` dir absent |
| `wpm audit <xes>` | 0 | Returns conformance verdict: DECEPTIVE / PASS / WARN (see below) |
| `wpm receipt doctor <file>` | 0 (per --help) | Audits receipt JSON against Adversarial Ingress Gates |
| `wpm receipt verify-ocel2` | 0 (per --help) | Validates embedded OCEL 2.0 logs |
| `wpm receipt detect-fixture-mutation` | 0 (per --help) | Structural similarity + temporal variance |
| `wpm receipt verify-boundary-evidence` | 0 (per --help) | Checks boundary_evidence block |
| `wpm receipt verify-proof-class` | 0 (per --help) | Validates proof_class vs evidence level |
| `wpm receipt verify-challenge` | 0 (per --help) | Checks challenge nonce cryptographic binding |
| `wpm receipt canonicalize-ocel2` | 0 (per --help) | Canonical sorted/minified OCEL representation |
| `wpm receipt producer-safe-report` | 0 (per --help) | Sanitized external integration report |
| `wpm receipt operator-private-report` | 0 (per --help) | Internal forensics report with raw hash comparisons |
| `wpm lean` | 0 | Lean Audit / Value Stream Mapping; identifies process wastes |
| `wpm spc status` | 0 | SPC cycle count, history length, sufficiency check |

---

## Raw Command Outputs

### `wpm doctor`
```
Running wpm doctor...
  [PASS] rustc: rustc 1.95.0 (59807616e 2026-04-14)
  [PASS] wasm-pack: wasm-pack 0.14.0
  [PASS] Cargo.toml found
  [PASS] src/ directory found
  [WARN] .wasm4pm directory not found

Summary:
All checks passed! Your environment is healthy.
```

### `wpm audit --help`
```
Run a Vision 2030 conformance audit on an event log

Usage: wpm audit [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to XES event log file

Options:
  -a, --activity-key <ACTIVITY_KEY>  Attribute key for activity names [default: concept:name]
  -v, --verbose                      Show verbose output
  -h, --help                         Print help
  -V, --version                      Print version
```

### `wpm lean`
```
Lean Audit: Value Stream Mapping
1. Overproduction (Artifact Bloat)
   [LEAN] No results directory found.

2. Motion (WASM Loading Latency)
   [WASTE] WASM server not running. CLI must cold-boot WASM (2.3s waste).

3. Defects (DoD Conformance)
   [LEAN] System is DoD-sealed (100 0.000000e+00st coverage verified).

======================================
Lean Audit: 1 process wastes identified.
```

### `wpm spc status`
```
SPC System Status
Cycle Count:              0
History Length:           0
Sufficient Data:          NO (Need 9 cycles)
```

---

## Audit Probe — Full Output

**Input file:** `/tmp/cargo_cicd_discovery.xes`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="">
  <extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>
  <trace>
    <string key="concept:name" value="discovery-probe"/>
    <event>
      <string key="concept:name" value="status show"/>
      <string key="cargo_cicd:verdict" value="PASS"/>
      <date key="time:timestamp" value="2026-06-02T00:00:00.000Z"/>
      <string key="lifecycle:transition" value="complete"/>
    </event>
  </trace>
</log>
```

**stdout:**
```
Vision 2030 Conformance Audit Report

Audit Verdict:            DECEPTIVE
Fitness Score:            0.0000
Precision Score:          0.0000

Total Traces Audited:     1
Fitting Traces:           1
Deviating Traces:         0

Doctrine: If the code says it worked but the event log cannot prove a lawful process happened, then it did not work.
```

**Exit code:** `0`

---

## Audit Verdict Analysis

`wpm audit <xes>` returns one of these verdicts in the `Audit Verdict` field:

| Verdict | Meaning |
|---|---|
| `DECEPTIVE` | Fitness/precision scores are 0 — log fits traces but model is undefined or empty; event log cannot prove lawful process |
| (expected) `PASS` | Fitness and precision > threshold against a declared process model |
| (expected) `WARN` | Partial conformance |
| (expected) `FAIL` / `REJECT` | Non-conforming log |

The minimal XES probe above returned `DECEPTIVE` with exit code 0. This is because the XES log contains no reference process model for comparison — fitness is structural (traces found) but precision is 0. The binary always exits 0; verdict discrimination is text-parsed from stdout.

---

## Stub / Deferred Commands

Based on `--help` output only (not exercised with real inputs in this probe):

- `wpm receipt doctor <file>` — requires a receipt JSON; stub behavior unknown
- `wpm receipt verify-ocel2`, `detect-fixture-mutation`, `verify-boundary-evidence`, `verify-proof-class`, `verify-challenge`, `canonicalize-ocel2`, `producer-safe-report`, `operator-private-report` — all declared subcommands; real behavior requires actual receipt artifacts
- `wpm receipt truthforge` — no help text; likely stub

---

## Conclusion: Evidence Gate Usage

The following wpm commands are confirmed operational for use in cargo-cicd evidence gates:

1. **`wpm audit <xes>`** — primary gate; parses XES event log and emits conformance verdict. Exit code is always 0; gate logic must parse stdout for `Audit Verdict:` field. Verdicts: `DECEPTIVE`, and presumably `PASS`/`WARN`/`FAIL`.

2. **`wpm doctor`** — environment health check; safe to run as precondition gate. Exits 0 even with WARNs.

3. **`wpm spc status`** — statistical process control readout; useful for cycle-count gating once SPC history accumulates (needs 9 cycles minimum).

4. **`wpm lean`** — value stream mapping; identifies process waste; exit 0 always; waste count parseable from final summary line.

5. **`wpm receipt doctor <receipt.json>`** — adversarial receipt gate; requires receipt JSON artifact; confirmed interface via `--help`. Supports `--strict`, `--format json`, `--audience ci` flags — suitable for CI pipeline integration.

**Recommended evidence gate pattern for cargo-cicd:**
```
wpm audit <xes_file>          # parse Audit Verdict from stdout
wpm receipt doctor <receipt>  # --format json --audience ci --strict
```

Both commands exit 0 regardless of verdict; gate scripts must parse output text or JSON.
