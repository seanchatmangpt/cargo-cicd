# MCP Servers & Plugin Bundle — cargo-cicd

Covers: `.claude/mcp-servers.json` (active + hypothetical) and `.claude/plugins/cargo-advanced-tools.yaml`.

---

## Active MCP Servers

Declared in `.claude/mcp-servers.json`. Active in every Claude Code session.

### `github` (builtin)

Scope: `seanchatmangpt/cargo-cicd` only. Cross-repo operations require explicit permission.

**Use for:** PR lifecycle, code search, Actions monitoring, release tagging.

| Tool | Signature | When to call |
|------|-----------|-------------|
| `create_pull_request` | `(repo, title, body, head, base)` | After new noun/verb implementation |
| `pull_request_read` | `(repo, pr_number)` | Before review pass |
| `pull_request_review_write` | `(repo, pr_number, event, comments)` | Submit review |
| `search_code` | `(query, repo?)` | Find forbidden terms; find existing patterns |
| `actions_list` | `(repo, workflow_id?)` | Monitor CI after push |
| `get_job_logs` | `(repo, job_id)` | Diagnose CI failure |
| `get_file_contents` | `(repo, path, ref?)` | Read historical file at commit |
| `push_files` | `(repo, branch, files, message)` | Multi-file commit via API |
| `get_latest_release` | `(repo)` | Pre-tag check |
| `list_releases` | `(repo)` | Release history |
| `add_issue_comment` | `(repo, issue_number, body)` | Comment on issue/PR |
| `resolve_review_thread` | `(repo, pr_number, thread_id)` | Mark thread resolved |
| `actions_run_trigger` | `(repo, workflow_id, ref, inputs?)` | Trigger workflow dispatch |

Full tool list also includes: `list_branches`, `create_branch`, `list_pull_requests`, `update_pull_request`, `merge_pull_request`, `add_reply_to_pull_request_comment`, `unresolve_review_thread`, `issue_read`, `issue_write`, `list_issues`, `search_issues`, `search_pull_requests`, `create_or_update_file`, `delete_file`, `get_commit`, `list_commits`, `actions_get`, `list_tags`, `get_tag`, `get_label`, `list_repository_collaborators`, `fork_repository`, `create_repository`, `search_repositories`, `search_users`, `search_commits`, `get_teams`, `get_team_members`, `sub_issue_write`, `enable_pr_auto_merge`, `disable_pr_auto_merge`, `subscribe_pr_activity`, `unsubscribe_pr_activity`, `add_comment_to_pending_review`, `request_copilot_review`, `run_secret_scanning`, `get_me`.

```
# Canonical usage — new noun PR
create_pull_request(
  repo="seanchatmangpt/cargo-cicd",
  title="feat(cli): add workspace validate verb",
  body="...",
  head="feat/workspace-validate",
  base="main"
)

# Search for forbidden terms
search_code(query="ALIVE OR Instinct8 OR Truex repo:seanchatmangpt/cargo-cicd")
```

---

### `cargo-cicd-evidence` (filesystem, read-only)

Roots: `${workspaceRoot}/target/cargo-cicd/evidence`, `${workspaceRoot}/receipts`

**FORBIDDEN: write=false. Never manually edit evidence files — oracle detects tampering → Refuse.**

File patterns:
- `target/cargo-cicd/evidence/evt-*.ocel.json` — OCEL 2.0 evidence (canonical, new code only)
- `target/cargo-cicd/evidence/evt-*.xes` — XES legacy (do not extend)
- `target/cargo-cicd/evidence/evt-*.jsonl` — JSONL companions
- `receipts/*.json` — wpm receipt artifacts

```
# Read evidence file
Read("target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.ocel.json")

# Find FAIL verdicts
Grep(pattern="verdict_claimed.*FAIL", path="target/cargo-cicd/evidence/", glob="*.jsonl")

# List all evidence
Glob(pattern="target/cargo-cicd/evidence/evt-*.ocel.json")

# Shell-out to oracle after locating file
wpm audit target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.ocel.json
# → Accept | Refuse | Blocked
```

**Evidence naming convention:**
```
evt-<command-slug>-<timestamp-compact>Z.<ext>
evt-status-show-20260614134507123Z.ocel.json
evt-publish-run-20260614140023456Z.ocel.json
```
- `<command-slug>`: noun-verb joined by `-`
- `<timestamp-compact>`: ISO 8601 without separators, ms included, always UTC

---

### `cargo-cicd-workspace` (filesystem, read-only)

Roots: `${workspaceRoot}`, `${workspaceRoot}/crates`
Include filter: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `cicd.toml`

```
# Read workspace manifest
Read("Cargo.toml")

# Read state carrier (last-run snapshot, not live state)
Read("cicd.toml")

# Read sub-crate manifest
Read("crates/cargo-cicd-core/Cargo.toml")

# Check for missing license fields
Grep(pattern="^license", path=".", glob="**/Cargo.toml", output_mode="files_with_matches")

# Read toolchain pin
Read("rust-toolchain.toml")
```

`cicd.toml` = last-completed-run snapshot. For live state: `cargo cicd status show` or `cargo cicd workspace doctor`.

---

## Hypothetical MCP Servers (not yet deployed)

Functionality currently via Bash + filesystem servers above. Documented as design spec.

### 1. `cargo-workspace` (process — hypothetical)

Binary: `cargo-workspace-mcp` (`crates/cargo-workspace-mcp/`)
Does NOT invoke `cargo` — pure TOML parsing. Faster than `cargo metadata`.

| Tool | Returns | Bash equivalent |
|------|---------|----------------|
| `list_members()` | `Vec<{name, version, path, manifest_path}>` | `grep -A20 '\[workspace\]' Cargo.toml` |
| `get_manifest(path)` | `{package, dependencies, features, workspace}` | `cat Cargo.toml` |
| `check_dependencies(manifest_path, check_type)` | `{pass, issues, recommendations}` | `cat Cargo.toml` + manual check |
| `find_workspace_root(start_path)` | `{found, root_path, discovery_method}` | `git rev-parse --show-toplevel` |

`check_type` values: `publish_required` | `duplicates` | `version_constraints`

---

### 2. `xes-evidence` (process — hypothetical)

Binary: `xes-evidence-mcp` (`crates/xes-evidence-mcp/`)

**OCEL 2.0 is canonical. XES is legacy. This server still covers XES for backward compat.**

| Tool | Returns | Bash equivalent |
|------|---------|----------------|
| `list_evidence_files(dir, filter?)` | `Vec<EvidenceFileSummary>` | `ls -la target/cargo-cicd/evidence/` |
| `parse_xes(file_path)` | `{traces, event_count, case_ids}` | `cat *.xes` + XML parse |
| `get_verdict(file_path)` | `{verdict_claimed, verdict_adjudicated, lifecycle_complete}` | `grep verdict *.xes` |
| `find_traces_by_case_id(dir, case_id)` | `Vec<TraceMatch>` | `grep -l 'value="pipeline_run_phase"' *.xes` |

Filter fields: `format: "xes"|"jsonl"|"all"`, `command`, `since_iso`, `verdict: "PASS"|"WARN"|"FAIL"`

---

### 3. `wasm4pm-oracle` (process — hypothetical)

Binary: `wasm4pm-mcp` (wraps `wpm` binary). `wpm` must be on PATH.
Oracle unavailable → all responses return `oracle_available: false`, verdict `Blocked` — not an error.

**OCEL emission pattern (all noun handlers):**
```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only — invariant E1: never adjudicate inside cargo-cicd)
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

OCEL 2.0 JSON shape wpm expects:
```json
{ "eventTypes": [...], "objectTypes": [...], "events": [...], "objects": [...] }
```
- `OCELEvent.relationships: Vec<OCELRelationship { objectId, qualifier }>`
- `OCELObject.relationships: Vec<OCELObjectRelationship { objectId, qualifier }>`

Object types in cargo-cicd domain: `Workspace`, `Crate`, `TestRun`, `GitCommit`, `Release`, `Receipt`, `EvidenceFile`, `Policy`, `Toolchain`

| Tool | Bash equivalent |
|------|----------------|
| `audit_xes(xes_file_path, options?)` → `OracleVerdict` | `wpm audit <file>` |
| `validate_receipt(receipt_path, options?)` → `ReceiptValidationResult` | `wpm receipt doctor --format json --strict <file>` |
| `get_oracle_version()` → `OracleVersionInfo` | `wpm --version` |
| `batch_audit(evidence_dir, pattern?, options?)` → `BatchAuditResult` | `for f in *.ocel.json; do wpm audit "$f"; done` |

wpm exit code contract:
| Exit code | Verdict | Meaning |
|-----------|---------|--------|
| `0` | `Accept` | Conformant |
| `1` | `Refuse` | Non-conformant — violations present |
| `2` | `Blocked` | Oracle error / malformed file |
| not found | `Blocked` | `wpm` not on PATH |

Tests assert `ExpectedWpmVerdict` enum values — never raw exit codes.

**FORBIDDEN:**
- `hand-rolling OcelLog`, `OcelEvent`, `OcelObject` structs — import from `wasm4pm-compat`
- Calling `wpm` on `.xes` files for new code — OCEL only
- Adjudicating inside cargo-cicd (invariant E1)
- Extending `evidence_xes_v2.rs` — legacy, do not touch
- `src/ocel.rs` — DELETE if present; replace with `wasm4pm-compat` imports

Cargo.toml dependency:
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

---

### 4. `git-phase` (process — hypothetical)

Binary: `git-phase-mcp` (wraps git commands, returns structured data).

Git phase lifecycle: `clean` | `dirty` | `staged` | `committed` | `pushed` | `behind`

| Tool | Returns | Bash equivalent |
|------|---------|----------------|
| `get_phase(workspace_root, base_ref?)` | `{phase, branch, ahead_count, behind_count, dirty_file_count, ...}` | `git status --porcelain && git rev-list --count HEAD ^origin/main` |
| `list_dirty_files(workspace_root, filter?)` | `Vec<{path, status, staged, is_rust_source, is_test_file, is_trybuild_fixture}>` | `git status --porcelain` |
| `ahead_behind_main(workspace_root, base_ref?)` | `{ahead, behind, diverged, needs_pull, needs_push}` | `git rev-list --count HEAD ^origin/main` |
| `list_changed_rs_files(workspace_root, base_ref?, options?)` | `{source_files, test_files, trybuild_fixtures, deleted_files}` | `git diff origin/main --name-only \| grep '\.rs$'` |

`list_dirty_files` filter: `status: "M"|"A"|"D"|"R"|"?"|"all"`, `include_untracked: bool`

---

## YAML Plugin Bundle: `cargo-advanced-tools.yaml`

**File:** `.claude/plugins/cargo-advanced-tools.yaml`
**Type:** Static YAML definition — NOT an MCP server process.
**Version:** 1.0.0 (matches cargo-cicd 26.6.2)
**MCP Namespace:** `cargo`
**Distributable toolkit:** `plugins/cargo-cicd-kit/`

| Aspect | MCP Servers | YAML Plugin Bundle |
|--------|-------------|-------------------|
| Runtime | Active process/builtin | Static definition |
| Protocol | MCP over stdio/HTTP | YAML parsed by plugin loader |
| Invocation | Claude calls via MCP | Named wrappers around `cargo` CLI |
| Scope | Data access, external systems, oracle | Direct cargo command execution |

### Tools (namespace: `cargo`)

| Tool | Key params | Bash equivalent |
|------|-----------|----------------|
| `build_with_features` | `features` (required), `release`, `all_targets`, `workspace` | `cargo build --features <f>` |
| `test_with_filter` | `scope` (required: `all\|lib\|integration\|doc\|unit`), `test_name`, `features`, `nocapture` | `cargo test [--test <name>]` |
| `check_all` | `features`, `all_targets`, `workspace`, `fix` | `cargo check --workspace --all-targets` |
| `analyze_workspace` | (none) | `cargo metadata --format-version 1` |
| `validate_features` | `feature_combo` (required), `check_conflicts` | `cargo check --features <combo>` |
| `clippy_suggestions` | `features`, `all_targets`, `all_features`, `fix` | `cargo clippy --all-targets` |
| `doc_generation` | `features`, `all_features`, `document_private_items`, `open_browser` | `cargo doc` |
| `metadata_extraction` | `format: json\|tree\|compact`, `include_dependencies` | `cargo metadata --format-version 1` |

Feature implication map (transitive — `validate_features` resolves these):
- `autonomic` → `process-data`
- `wasm4pm` → `process-data`
- `contrib` → `process-data`
- `advanced` → `process-data`

Bundle roles:
- Schema registry for `plugins/cargo-cicd-kit/` type-safe wrapper generation
- Feature compatibility manifest (`features.available`, `features.recommended_combinations`)
- Integration declaration (`integration.adapters`, `integration.nouns`)
- Performance config source (rate limits, timeouts, caching TTLs, parallelism)

---

## Configuration

### `mcp-servers.json` schema

```json
{
  "mcpServers": {
    "<name>": {
      "type": "builtin" | "filesystem" | "process",
      "description": "<string>",

      // filesystem only:
      "roots": ["${workspaceRoot}/path"],
      "include": ["*.toml"],
      "capabilities": { "read": true, "write": false, "search": true },

      // process only:
      "command": "/path/to/binary",
      "args": ["--mode", "mcp"],
      "env": { "KEY": "${workspaceRoot}/.config" }
    }
  }
}
```

Variable substitution:
| Variable | Expands to |
|----------|----------|
| `${workspaceRoot}` | Absolute path to workspace root (where `.claude/` lives) |
| `${userHome}` | User home directory |
| `${env:VAR_NAME}` | Environment variable at session start |

After editing `mcp-servers.json`: restart Claude Code session (not hot-reloaded).

Validate JSON: `python3 -m json.tool .claude/mcp-servers.json`

---

## Security

- All configured filesystem servers: `write: false`. Do not change without explicit need.
- Validate all file paths before passing to MCP tools:
  - Must be within expected directory (no `../` sequences)
  - Must match expected extension (`.ocel.json`, `.xes`, `.jsonl`, `.json`, `.toml`)
  - Must be absolute paths
- FORBIDDEN terms list applies to all MCP tool arguments, PR titles, issue bodies, commit messages.
- Do not search for forbidden terms except to eliminate them.
- Evidence files are write-once. Oracle detects tampering → `Refuse`.
- GitHub MCP: never push directly to `main`; always use `create_pull_request`.
- GitHub MCP: never force-push or trigger workflows on third-party repos.

---

## Troubleshooting

| Symptom | Diagnosis | Fix |
|---------|-----------|-----|
| MCP server not responding | Invalid JSON in config | `python3 -m json.tool .claude/mcp-servers.json` |
| Process server missing | Binary not on PATH | `which <binary>` |
| Evidence server empty | No commands run yet | `cargo cicd status show` to emit first evidence |
| GitHub 403/404 | Auth or wrong repo | `get_me` to verify auth; check branch exists |
| `oracle_available: false` | `wpm` not on PATH | `export PATH="/path/to/wasm4pm/target/release:$PATH"` then `wpm --version` |
| `target/cargo-cicd/evidence/` missing | First run | `mkdir -p target/cargo-cicd/evidence` |

Oracle unavailable = `ExpectedWpmVerdict::Blocked` in tests — not an error state.

---

*cargo-cicd 26.6.2 — updated 2026-06-21*
