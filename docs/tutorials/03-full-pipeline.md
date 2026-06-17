# Tutorial: Run the Full Pipeline

By the end of this tutorial you will have run `examples/03_max_pipeline` — a program that exercises every capability in cargo-cicd simultaneously — and seen it complete with a `PASS` verdict and an OCEL evidence record on disk.

**Prerequisites:**

- Completed [Tutorial 2](02-ocel-evidence.md)
- The cargo-cicd source checked out
- Rust 1.85 or later

---

## Step 1 — Enable all features

The maximalist pipeline requires these feature flags:

| Flag | What it adds |
|------|--------------|
| `process-data` | `EngineState` and all adapters |
| `autonomic` | Policy evaluation (implies `process-data`) |
| `advanced` | All 10 best-of-breed modules (implies `process-data`) |

Optional (not required for this tutorial):

| Flag | What it adds |
|------|--------------|
| `wasm4pm` | Live oracle verdict instead of `Blocked` |
| `contrib` | Maintainer diagnostics |

---

## Step 2 — Run the maximalist pipeline example

```sh
cargo run --example 03_max_pipeline \
    --features process-data,autonomic,advanced
```

You will see ten lines like:

```
[1/10]  workspace: cargo-cicd
[2/10]  parallel_scan: 847 files, 1234567 reclaimable bytes
[3/10]  fingerprint: 3a9f2b...
[4/10]  cache: 64 bytes cached
[5/10]  dep_graph: 3 members, build order: [".", "cargo-cicd-core", "cargo-cicd-lsp"]
[6/10]  timeline: 3 events recorded
[7/10]  pattern: 2 governance matches
[8/10]  diagnostics: Error — 312 chars
[9/10]  histogram: workspace_scan p99 = 45000µs
[10/10] snapshot: 128 bytes (bitcode)

pipeline complete — all 10 advanced modules exercised
  total span : 0.003s
  evidence   : target/cargo-cicd/evidence/max_pipeline.ocel.json
  verdict    : Blocked (wpm not required — oracle is optional)
```

---

## Step 3 — Inspect the evidence

```sh
cat target/cargo-cicd/evidence/max_pipeline.ocel.json
```

You will see four events — `status show`, `target show`, `workspace doctor`, `evidence audit` — all grouped under the `maximalist_pipeline` case.

---

## Step 4 — Run with the oracle (optional)

If `wpm` is on your `PATH`:

```sh
wpm receipt verify-ocel2 target/cargo-cicd/evidence/max_pipeline.ocel.json
```

The verdict should be `Accept`.

---

## What you have learned

- All five feature tiers compose without conflict
- All ten advanced modules (`observability`, `parallel_scan`, `fingerprint`, `cache`, `dep_graph`, `timeline`, `pattern`, `diagnostics`, `histogram`, `snapshot`) run in a single pipeline
- Evidence is always emitted regardless of oracle availability
- `Blocked` degrades gracefully to `Accept` when `wpm` is present

**For the rationale behind why these capabilities compose the way they do, see:**  
[Explanation: Combinatorial Maximalism](../explanation/combinatorial-maximalism.md)
