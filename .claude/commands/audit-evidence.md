---
description: Run the evidence doctor and status audit, explain the verdict, and point at the evidence directory.
allowed-tools: Bash, Read
---

Trigger: user asks to audit evidence, check receipts, adjudicate, or verify process evidence was accepted.

## Step 1 — Evidence doctor

```bash
cargo cicd evidence doctor
```

Record full output including Accept / Refuse / Warn lines.

## Step 2 — Status audit

```bash
cargo cicd status audit
```

Note every ERROR or WARN diagnostic.

## Step 3 — Inspect evidence directory

```bash
ls -lh target/cargo-cicd/evidence/ 2>/dev/null || echo "directory not found — run at least one cargo cicd command first"
```

For each `.json` receipt:
```bash
for f in target/cargo-cicd/evidence/*.json; do
  [ -f "$f" ] || continue
  verdict=$(grep -o '"verdict":"[^"]*"' "$f" | head -1)
  echo "$f  →  ${verdict:-verdict field not found}"
done
```

## Step 4 — Verdict interpretation

| Verdict | Meaning | Action |
|---------|---------|--------|
| `Accept` | Evidence satisfies process-data contract; gate is open | None |
| `Refuse` | Receipt malformed, missing required fields, or XES unparseable | List failing files, re-run emitting command |
| `Blocked` | `wpm` binary unavailable | Expected in local dev; not an error |

Required receipt fields: `trace_id`, `case_id`, `concept:name`, timestamps.

## Step 5 — Remediation (on Refuse)

1. Re-run the emitting command (e.g. `cargo cicd workspace doctor`, `cargo cicd status show`)
2. Verify feature flag: `cargo build --features process-data`
3. Check `[evidence]` section in `cicd.toml` points at `target/cargo-cicd/evidence/`
4. If `.xes` files are empty: check `[[events]]` table in `cicd.toml` for last emitted event
