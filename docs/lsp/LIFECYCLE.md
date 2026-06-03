# Diagnostic Lifecycle

A cargo-cicd-lsp diagnostic moves through four states: **raised**, **routed**, **pending repair**, and **cleared by evidence**.

---

## States

### 1. Raised

A diagnostic is raised when an evaluator detects a condition that maps to a CICD-XXX-NNN code.

Evaluation is triggered by:
- An LSP `didOpen` or `didSave` notification on a watched file
- A `didChangeWatchedFiles` notification when `cicd.toml` is modified
- A manual re-evaluation request from the client

Each evaluator is stateless. It reads current workspace state and either produces a diagnostic or produces nothing. There is no "diagnostic object" that persists between evaluations independent of its evidence.

### 2. Routed

A raised diagnostic is routed to the file and range that best represents its source location.

Routing rules by domain:

| Domain | Primary file | Fallback |
|--------|-------------|---------|
| CFG | `cicd.toml` at the offending key | workspace root `cicd.toml` line 1 |
| TGT | The `Cargo.toml` declaring the target | workspace root |
| TCH | `rust-toolchain.toml` | workspace root |
| CHG | The changed source file | `cicd.toml` |
| GIT | The file with the conflict or block | workspace root |
| WRK | The `Cargo.toml` with the problem | workspace root manifest |

If the primary file does not exist (e.g., `cicd.toml` is missing), the diagnostic is routed to the workspace root directory entry, column 0.

### 3. Pending Repair

A diagnostic remains in the pending repair state for as long as the condition that produced it persists. The server re-evaluates on each relevant file change notification.

The server does not aggregate or deduplicate across evaluations. Each evaluation produces a complete, current set of diagnostics. The previous set is replaced, not merged.

### 4. Cleared by Evidence

A diagnostic is cleared when the evaluator that raised it runs again and produces no output for the same code and location.

**Clearing is always by evidence, never by timer or user acknowledgement.**

This means:
- Dismissing a diagnostic in the editor UI does not clear it
- Saving an unrelated file does not clear it
- Restarting the editor does not clear it if the condition persists
- Only fixing the underlying condition and triggering re-evaluation clears it

---

## Keyed-Subtraction Law

The server maintains the current diagnostic set as a map keyed by `(uri, code)`.

On each evaluation:

```
new_set = evaluate(workspace_state)
published = new_set

# Diagnostics not present in new_set are implicitly cleared
# by publishing new_set as the complete set for each URI
```

This is the **keyed-subtraction law**: the LSP `publishDiagnostics` notification for a URI always carries the complete current set for that URI. Any code previously published for that URI that does not appear in the new set is treated by the client as cleared.

Consequences:
- A diagnostic cannot be "stuck" if the condition is resolved
- A diagnostic cannot disappear if the condition persists
- There is no separate "clear" message; absence from the published set is the clearing signal

---

## Multi-File Diagnostics

Some conditions affect multiple files. In these cases, the server publishes one diagnostic per affected URI, each with the same code but scoped to the relevant range in that file.

Example: CICD-GIT-005 (merge conflict markers) is raised once per file containing conflict markers, not as a single workspace-level diagnostic.

---

## Evaluation Order

Evaluators run in domain order: CFG → WRK → TCH → TGT → CHG → GIT.

A CFG-domain error (e.g., `cicd.toml` parse failure) will suppress downstream evaluators that depend on parsed config. This prevents diagnostic noise from a single root cause producing a cascade of secondary codes.

The suppression is explicit: each evaluator declares its dependencies. If a dependency's evaluator produced an error-severity diagnostic, the dependent evaluator is skipped and its previous diagnostics are cleared.

---

## See Also

- [DIAGNOSTICS.md](DIAGNOSTICS.md) — Full code table
- [README.md](README.md) — What the server does and does not do
