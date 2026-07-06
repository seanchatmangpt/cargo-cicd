# Diagnostic Lifecycle

A cargo-cicd-lsp diagnostic moves through four states:
**raised** → **routed** → **pending repair** → **cleared by evidence**

A fifth outcome, **residual preserved**, applies when a diagnostic cannot be cleared
because its observed surface is no longer accessible (e.g., the file was deleted).

---

## States

### 1. Raised

A diagnostic is raised when an evaluator detects a condition that maps to a
`CICD-{FAMILY}-{NNN}` code.

Evaluation is triggered by:
- An LSP `textDocument/didOpen` or `textDocument/didSave` notification on a watched file
- A `workspace/didChangeWatchedFiles` notification when `cicd.toml` or a `Cargo.toml` is modified
- A manual re-evaluation request from the client

Each evaluator is stateless. It reads current workspace state and either produces a
diagnostic or produces nothing. There is no diagnostic object that persists between
evaluations independent of its evidence.

### 2. Routed

A raised diagnostic is routed to the file and range that best represents its source location.

Routing is by family:

| Family | Primary file | Fallback |
|--------|-------------|---------|
| GIT | the file containing the conflict or block | workspace root |
| EVIDENCE | the evidence log file | workspace root |
| WPM | `cicd.toml` wpm configuration key | workspace root `cicd.toml` line 1 |
| TEST | the changed source file with missing coverage | `cicd.toml` |
| TARGET | the `Cargo.toml` declaring the target | workspace root manifest |
| PUBLISH | `cicd.toml` at the offending key | workspace root `cicd.toml` line 1 |
| PUBLIC | the source file containing the undocumented or leaking item | workspace root |
| GGEN | the stale generated file | `cicd.toml` template declaration |
| CLOSE | `cicd.toml` | workspace root |

If the primary file does not exist, the diagnostic is routed to the workspace root
directory entry, column 0.

### 3. Pending Repair

A diagnostic remains in the pending repair state for as long as the condition that
produced it persists. The server re-evaluates on each relevant file change notification.

The server does not accumulate or deduplicate across evaluations. Each evaluation produces
a complete, current set of diagnostics for the affected URIs. The previous set for each
URI is replaced, not merged.

### 4. Cleared by Evidence

A diagnostic is cleared when the evaluator that raised it runs again and produces no
output for the same code and location.

**Clearing is always by evidence, never by timer or user acknowledgement.**

This means:
- Dismissing a diagnostic in the editor UI does not clear it
- Saving an unrelated file does not clear it
- Restarting the editor does not clear it if the condition persists
- Only fixing the underlying condition and triggering re-evaluation clears it

### 5. Residual Preserved

When a diagnostic's observed surface becomes inaccessible after the diagnostic was raised
(for example, a file is deleted before the condition is resolved), the diagnostic is
preserved as a residual attached to the workspace root URI.

Residuals are cleared when:
- The observed surface becomes accessible again and the condition is resolved, or
- The workspace is re-initialised and the evaluator finds no condition

---

## Keyed-Subtraction Law

The server maintains the current diagnostic set as a map keyed by `(uri, code)`.

On each evaluation for a URI:

```
new_set = evaluate(workspace_state, uri)
publish(uri, new_set)
```

The LSP `textDocument/publishDiagnostics` notification for a URI always carries the
**complete current set** for that URI. Any code previously published for that URI that
does not appear in the new set is treated by the LSP client as cleared.

**Key law: clearing one diagnostic for a URI does not erase others.**

Clearing is keyed subtraction — only the specific `(uri, code)` pair is removed from the
published set. Other diagnostics for the same URI that are still valid remain in the set
and continue to be published.

Consequences:
- A diagnostic cannot be stuck if the condition is resolved
- A diagnostic cannot disappear if the condition persists
- There is no separate "clear" message; absence from the published set is the clearing signal
- Fixing one problem in a file does not accidentally suppress other active diagnostics in that file

---

## Multi-File Diagnostics

Some conditions affect multiple files. The server publishes one diagnostic per affected
URI, each with the same code but scoped to the relevant range in that file.

Example: a dirty working tree may produce CICD-GIT-001 routed to each modified file,
not as a single workspace-level diagnostic. This lets the editor show the annotation
on the specific file that needs attention.

---

## Evaluation Order and Suppression

Evaluators run in family order: GIT → EVIDENCE → WPM → TEST → TARGET → PUBLISH → PUBLIC → GGEN → CLOSE.

A family-level error in an upstream family suppresses downstream evaluators that depend
on its output. For example, a CICD-PUBLISH-002 (cicd.toml parse failure) suppresses
CLOSE-family evaluators that depend on parsed publish configuration. This prevents a
single root cause from producing a cascade of secondary codes.

Suppression is explicit: each evaluator declares its upstream dependencies. If a
dependency's evaluator produced an Error-severity diagnostic, the dependent evaluator
is skipped and its previous diagnostics for affected URIs are cleared.

---

## See Also

- [Diagnostics](diagnostics.md) — Full code catalog by family
- [LSP overview](../../lsp/README.md) — What the server does and does not do
