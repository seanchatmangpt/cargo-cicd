---
description: Run the evidence doctor and status audit, explain the verdict, and point at the evidence directory.
allowed-tools: Bash, Read
---

You are auditing the process-evidence produced by cargo-cicd. Work through the steps below, show all command output verbatim, then explain what the verdict means.

---

## Step 1 — Evidence doctor

Run the built-in evidence health check:

```
cargo cicd evidence doctor
```

This command inspects receipts and XES event streams under `target/cargo-cicd/evidence/`. Record the full output including any Accept / Refuse / Warn lines.

---

## Step 2 — Status audit

Run the audit sub-verb of the status noun:

```
cargo cicd status audit
```

This surfaces any policy-level issues the engine has flagged since the last run. Note every diagnostic at ERROR or WARN severity.

---

## Step 3 — Inspect the evidence directory

List the contents of the evidence directory so the user can see what was emitted:

```bash
ls -lh target/cargo-cicd/evidence/ 2>/dev/null || echo "directory not found — run at least one cargo cicd command first"
```

For each `.xes` file found, show its name, size, and modification time. XES (XML Event Stream) files are the canonical process-evidence format.

For each `.json` receipt file found, show a one-line summary of its `verdict` field:

```bash
for f in target/cargo-cicd/evidence/*.json; do
  [ -f "$f" ] || continue
  verdict=$(grep -o '"verdict":"[^"]*"' "$f" | head -1)
  echo "$f  →  ${verdict:-verdict field not found}"
done
```

---

## Step 4 — Explain the verdict

Based on the output from steps 1–3, explain:

1. **What the doctor found** — whether receipts are well-formed, whether required fields (`trace_id`, `case_id`, `concept:name`, timestamps) are present.
2. **What "Accept" means** — the evidence satisfies the process-data contract; the gate is open.
3. **What "Refuse" means** — one or more receipts are malformed, missing required fields, or the XES stream is not parseable. List which receipts failed and why.
4. **Where to look next** — if a refusal was issued, point at the specific file(s) under `target/cargo-cicd/evidence/` that need attention, and suggest which `cargo cicd` command should be re-run to regenerate clean evidence.

---

## Step 5 — Remediation tips (if needed)

If any step produced an error or refusal:

- Re-run the command that should have emitted the receipt (e.g. `cargo cicd workspace doctor`, `cargo cicd status show`).
- Check that the `process-data` feature flag is compiled in: `cargo build --features process-data`.
- Verify the `[evidence]` section in `cicd.toml` points at `target/cargo-cicd/evidence/`.
- If `.xes` files are present but empty, the adapter may have failed to flush — check the `[[events]]` table in `cicd.toml` for the last emitted event.
