# Affidavit Integration — Cryptographic Provenance Receipts

cargo-cicd integrates [**affidavit**](https://github.com/seanchatmangpt/affidavit),
a cryptographic provenance engine that *"assembles, seals, and certifies
provenance receipts — append-only, content-addressed BLAKE3 chains of
operation-events."* affidavit becomes cargo-cicd's **second external witness**,
sitting alongside the wasm4pm process-conformance oracle.

| Witness  | Question it answers                         | Mechanism                          |
|----------|---------------------------------------------|------------------------------------|
| wasm4pm  | *Does the process conform?* (fitness)       | XES/OCEL token-replay adjudication |
| affidavit| *Is the evidence intact?* (integrity)       | rolling BLAKE3 receipt chain       |

Both honour cargo-cicd invariant **E1** — the engine never grades itself, only
an external authority issues a verdict. affidavit's doctrine, *"Certify, Don't
Decide,"* is that same principle expressed cryptographically.

---

## Why shell-out, not a library dependency

affidavit's mandatory `core` feature pulls in `wasm4pm-compat`, which requires
unstable rustc features (`generic_const_exprs`, `unsized_const_params`,
`min_specialization`, …). cargo-cicd is a **stable-toolchain** tool
(`rust-version = 1.86`); linking affidavit in-process would force the entire
project — and every downstream user — onto nightly.

So affidavit is integrated exactly like wasm4pm: by invoking the installed
`affi` binary at runtime (`src/integrations/affidavit_shell.rs`, mirroring
`wasm4pm_shell.rs`). This keeps the default build on stable, keeps the certifier
at arm's length, and degrades gracefully when `affi` is absent.

```
default build           cargo-cicd (stable)            ← no affidavit code compiled
--features affidavit    + affidavit_shell + noun        ← still stable; talks to `affi` over a pipe
runtime, affi present   affi receipt emit/assemble/verify
runtime, affi absent    WARN + install hint, exit 0     ← never blocks the workspace
```

---

## Enabling it

The integration is gated behind the opt-in `affidavit` feature (implies
`process-data`). There is **no compile-time crate dependency** — only the
runtime binary.

```sh
# Build with the affidavit noun available
cargo build --features affidavit

# Point cargo-cicd at the affi binary (or put it on PATH)
export AFFI_PATH=/path/to/affi      # falls back to `which affi`
```

---

## Commands

```sh
# 1. Seal: replay the evidence journal into affidavit and assemble a receipt
cargo cicd affidavit seal

# 2. Verify: certify the sealed receipt through affidavit's pipeline
cargo cicd affidavit verify
```

### `affidavit seal`

Reads the accumulated process-event journal
(`target/cargo-cicd/evidence/events.jsonl`, via `evidence::read_journal`) and,
for each event, shells out to:

```
affi receipt emit --type <command:lifecycle> --object <workspace:workspace:verdict> --payload <event.json>
```

then seals the chain with:

```
affi receipt assemble --out target/cargo-cicd/evidence/affidavit/receipt.json
```

All affi working state and the final `receipt.json` live under
`target/cargo-cicd/evidence/affidavit/`.

### `affidavit verify`

Certifies the sealed receipt:

```
affi receipt verify target/cargo-cicd/evidence/affidavit/receipt.json
```

The verdict is conveyed by **exit code** — `0` = `ACCEPT`, non-zero = `REJECT`.
A single bit-flip anywhere in the chain flips the verdict to `REJECT`. The verb
claims `PASS` on ACCEPT, `FAIL` on REJECT, and `WARN` when `affi` (or the
receipt) is absent.

---

## Event mapping

Each cargo-cicd `ProcessEvent` becomes one affidavit operation-event. The pure,
unit-tested mapping helpers live in `affidavit_shell.rs`:

| cargo-cicd `ProcessEvent`        | affi `receipt emit` argument                          |
|----------------------------------|-------------------------------------------------------|
| `command` + `lifecycle_transition` | `--type status:show:complete` (`event_type_for`)    |
| `workspace_id` + `verdict_claimed` | `--object <ws>:workspace:<verdict>` (`object_ref_for`)|
| canonical JSON of the event      | `--payload payload-<n>.json` (BLAKE3-committed by affi)|

`object_ref_for` sanitizes `:` and whitespace so the `ID:TYPE:QUALIFIER` token
always parses with exactly two separators.

---

## The `affi` CLI contract

Confirmed from affidavit's `examples/golden_run.sh`:

| Command                                                        | Purpose                    |
|----------------------------------------------------------------|----------------------------|
| `affi receipt emit --type T --object ID:TYPE:QUAL --payload F` | record one operation-event |
| `affi receipt assemble --out receipt.json`                     | seal accumulated events    |
| `affi receipt verify receipt.json`                             | certify (exit 0 = ACCEPT)  |
| `affi receipt show receipt.json`                               | display a sealed receipt   |

affidavit's seven-stage decidable pipeline (Decode → Format → Chain Integrity →
Continuity → Commitment → Profile → Verdict) runs inside `affi receipt verify`.

---

## Tests

`tests/affidavit_integration.rs` (run with `--features affidavit`) covers the
pure mapping helpers and the graceful-degradation path when `affi` is absent —
the realistic CI state, mirroring how the wasm4pm evidence-gate tests treat a
missing oracle (`Blocked`). The verbs exit 0 with or without `affi` installed,
so the workspace is never blocked by a missing certifier.

```sh
cargo test --features affidavit --test affidavit_integration
cargo test --features affidavit --lib affidavit          # pure-helper unit tests
```

---

## Files

| File                                   | Role                                             |
|----------------------------------------|--------------------------------------------------|
| `src/integrations/affidavit_shell.rs`  | `AffidavitShell` shell-out adapter + mapping helpers |
| `src/nouns/affidavit.rs`               | `affidavit seal` / `affidavit verify` verbs      |
| `src/evidence.rs`                      | `read_journal()` — load the accumulated journal  |
| `tests/affidavit_integration.rs`       | integration + graceful-degradation tests         |
