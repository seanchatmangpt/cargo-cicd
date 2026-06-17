# Tutorial: Your First OCEL Evidence Record

By the end of this tutorial you will have emitted an OCEL 2.0 evidence file to disk and know where to find it.

**Prerequisites:**

- Completed [Tutorial 1](01-first-clean-workspace.md) — you have cargo-cicd installed
- The cargo-cicd source checked out (for the library example)

---

## Step 1 — What is OCEL evidence?

When cargo-cicd runs a command it emits a structured record of what happened. That record is written in **OCEL 2.0** (Object-Centric Event Log) format — a JSON file at `target/cargo-cicd/evidence/`. An external oracle (`wpm`) can then adjudicate whether the process conformed to policy.

You do not need `wpm` for this tutorial. When the oracle is absent the verdict is `Blocked`, which is a first-class expectation, not an error.

---

## Step 2 — Run the evidence example

From the cargo-cicd workspace root:

```sh
cargo run --example 02_ocel_evidence
```

You will see:

```
OCEL 2.0 evidence written to:
  /path/to/your/workspace/target/cargo-cicd/evidence/events.ocel.json

To verify with the oracle (if wpm is on PATH):
  wpm receipt verify-ocel2 .../events.ocel.json

Without wpm, the verdict is: Blocked (expected — oracle is optional)
timestamp: 2026-06-17T...Z
```

---

## Step 3 — Inspect the evidence file

```sh
cat target/cargo-cicd/evidence/events.ocel.json
```

You will see a JSON document with this structure:

```json
{
  "ocel:version": "2.0",
  "ocel:events": {
    "evt-status-show-...Z": {
      "ocel:activity": "status:show",
      "ocel:timestamp": "2026-06-17T...Z",
      "ocel:vmap": {},
      "ocel:typedOmap": []
    }
  },
  "ocel:objects": {},
  "ocel:event-types": [],
  "ocel:object-types": []
}
```

Every command cargo-cicd runs produces one or more events in this format.

---

## Step 4 — Understand the event fields

| Field | Meaning |
|-------|---------|
| `ocel:activity` | The command that ran, e.g. `"status:show"` |
| `ocel:timestamp` | When it ran, in ISO-8601 UTC |
| `ocel:vmap` | Attributes (verdict, duration, etc.) |
| `ocel:typedOmap` | Object relationships (workspace, crate, etc.) |

---

## Step 5 — Run cargo cicd evidence audit

```sh
cargo cicd evidence audit
```

This reads the evidence directory and reports what was found. Without `wpm` installed the verdict column shows `Blocked`.

---

## What you have learned

- Every cargo-cicd command emits evidence to `target/cargo-cicd/evidence/`
- Evidence is OCEL 2.0 JSON — machine-readable and oracle-ready
- `Blocked` is a valid verdict meaning "oracle not present"
- The oracle (`wpm`) is optional for development; required only for release gates

**Next:** [Tutorial 3 — Run the full pipeline with all features enabled](03-full-pipeline.md)
