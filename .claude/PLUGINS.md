# MCP Server Documentation for cargo-cicd

> **Scope note:** This document covers **MCP servers** declared in `.claude/mcp-servers.json`
> (both active and hypothetical). It is *not* the plugin bundle reference. For the YAML
> plugin bundle (`.claude/plugins/cargo-advanced-tools.yaml`), see the dedicated section
> [YAML Plugin Bundle](#yaml-plugin-bundle-cargo-advanced-toolsyaml) near the end of this
> file, or consult the distributable toolkit at `plugins/cargo-cicd-kit/`.

This document is the authoritative reference for all Model Context Protocol (MCP) server
integrations used by cargo-cicd. It covers configured servers, hypothetical domain-specific
servers, installation procedures, security constraints, and tool invocation examples.

---

## Overview

cargo-cicd uses MCP (Model Context Protocol) servers to extend Claude Code's capabilities
for Rust workspace management, evidence adjudication, and CI/CD orchestration. MCP servers
allow Claude Code to call structured tools — beyond raw Bash — to read, query, and interact
with external systems in a controlled, typed, and auditable way.

MCP servers in cargo-cicd serve three primary roles:

1. **Data Access** — Reading Cargo manifests, XES evidence files, and workspace state
   without spawning heavyweight processes like `cargo metadata`.
2. **External Integration** — Communicating with GitHub for PRs, issues, and code review
   automation within the cargo-cicd release workflow.
3. **Oracle Interfacing** — Querying the wasm4pm evidence adjudication system for process
   conformance verification (currently via Bash; see `wasm4pm-oracle` hypothetical server
   below for the MCP-native alternative).

MCP servers are configured in `.claude/mcp-servers.json`. Each server declares a type,
description, optional roots (for filesystem servers), and capability flags.

**Key principle:** MCP tools are preferred over raw Bash tool calls when:
- Structured, typed output is needed (JSON rather than text parsing)
- Access to specific file subtrees must be scoped and auditable
- Tool calls should be visible in Claude's reasoning trace

---

## Configured MCP Servers

The following servers are declared in `.claude/mcp-servers.json` and are active in every
Claude Code session scoped to this repository.

---

### `github`

**Type:** `builtin`

**Purpose:**
Provides full GitHub API access for repository management, pull request lifecycle, issue
tracking, code search, and Actions workflow control. Scoped to the
`seanchatmangpt/cargo-cicd` repository.

Within cargo-cicd, the GitHub MCP server is the primary tool for:
- Creating and reviewing pull requests for new nouns, verbs, or policy modules
- Searching code across the repository without local filesystem access
- Triggering and monitoring GitHub Actions CI runs
- Managing release tags and changelogs
- Adding inline PR review comments during code review passes

**Tools Exposed:**

| Tool | Signature | Description |
|------|-----------|-------------|
| `get_me` | `()` | Return the authenticated GitHub user identity |
| `list_branches` | `(repo)` | List all branches in the repository |
| `create_branch` | `(repo, branch, sha)` | Create a new branch from a given SHA |
| `list_pull_requests` | `(repo, state?, base?)` | List open or closed PRs |
| `pull_request_read` | `(repo, pr_number)` | Fetch full PR metadata, body, and diff summary |
| `create_pull_request` | `(repo, title, body, head, base)` | Open a new PR |
| `update_pull_request` | `(repo, pr_number, ...)` | Update PR title, body, or state |
| `merge_pull_request` | `(repo, pr_number, method)` | Merge with squash, merge, or rebase |
| `pull_request_review_write` | `(repo, pr_number, event, comments)` | Submit an approval, request-changes, or comment review |
| `add_issue_comment` | `(repo, issue_number, body)` | Add a comment to an issue or PR |
| `add_reply_to_pull_request_comment` | `(repo, pr_number, comment_id, body)` | Reply inline to a specific PR comment |
| `resolve_review_thread` | `(repo, pr_number, thread_id)` | Mark a review thread as resolved |
| `unresolve_review_thread` | `(repo, pr_number, thread_id)` | Reopen a resolved review thread |
| `issue_read` | `(repo, issue_number)` | Fetch issue metadata and comments |
| `issue_write` | `(repo, title, body, labels?)` | Create a new issue |
| `list_issues` | `(repo, state?, labels?)` | List issues with optional filters |
| `search_code` | `(query, repo?)` | Search code within the repository |
| `search_issues` | `(query, repo?)` | Search issues and PRs by keyword |
| `search_pull_requests` | `(query, repo?)` | Search PRs by filters |
| `get_file_contents` | `(repo, path, ref?)` | Fetch raw file content at a given ref |
| `create_or_update_file` | `(repo, path, message, content, sha?)` | Create or update a file via API |
| `delete_file` | `(repo, path, message, sha)` | Delete a file via API |
| `push_files` | `(repo, branch, files, message)` | Push multiple files in a single commit |
| `get_commit` | `(repo, sha)` | Fetch commit metadata and changed files |
| `list_commits` | `(repo, sha?, path?)` | List commits on a branch or path |
| `actions_list` | `(repo, workflow_id?)` | List workflow runs |
| `actions_get` | `(repo, run_id)` | Get a specific workflow run status |
| `actions_run_trigger` | `(repo, workflow_id, ref, inputs?)` | Trigger a workflow dispatch |
| `get_job_logs` | `(repo, job_id)` | Fetch logs from a specific Actions job |
| `list_releases` | `(repo)` | List published releases |
| `get_latest_release` | `(repo)` | Get the most recent release |
| `get_release_by_tag` | `(repo, tag)` | Get release by version tag |
| `list_tags` | `(repo)` | List git tags |
| `get_tag` | `(repo, tag)` | Get a specific tag with SHA |
| `get_label` | `(repo, name)` | Get label metadata |
| `list_repository_collaborators` | `(repo)` | List all collaborators |
| `fork_repository` | `(repo)` | Fork the repository |
| `create_repository` | `(name, description?, private?)` | Create a new repository |
| `search_repositories` | `(query)` | Search GitHub repositories |
| `search_users` | `(query)` | Search GitHub users |
| `search_commits` | `(query, repo?)` | Search commits |
| `get_teams` | `(org)` | List teams in an organization |
| `get_team_members` | `(org, team_slug)` | List members of a team |
| `sub_issue_write` | `(repo, parent_issue, title, body)` | Create a sub-issue |
| `enable_pr_auto_merge` | `(repo, pr_number, method)` | Enable auto-merge for a PR |
| `disable_pr_auto_merge` | `(repo, pr_number)` | Disable auto-merge |
| `subscribe_pr_activity` | `(repo, pr_number)` | Subscribe to PR notifications |
| `unsubscribe_pr_activity` | `(repo, pr_number)` | Unsubscribe from PR notifications |
| `add_comment_to_pending_review` | `(repo, pr_number, body, path, line)` | Add a line comment to an in-progress review |
| `request_copilot_review` | `(repo, pr_number)` | Request a Copilot AI review |
| `run_secret_scanning` | `(repo)` | Trigger secret scanning on the repository |

**When to Use:**
- Opening a PR after implementing a new noun or verb: `create_pull_request`
- Reviewing existing PR changes before merging: `pull_request_read` + `pull_request_review_write`
- Searching for existing implementations of a pattern: `search_code`
- Monitoring CI after a push: `actions_list` + `get_job_logs`
- Tagging a release after evidence gate passes: `get_latest_release` + push tag via Bash

**Example Invocations:**

```
# Open a PR for a new noun implementation
create_pull_request(
  repo="seanchatmangpt/cargo-cicd",
  title="feat(cli): add workspace validate verb",
  body="Implements workspace validate as a new verb under the workspace noun...",
  head="feat/workspace-validate",
  base="main"
)

# Search for forbidden terms across source files
search_code(
  query="ALIVE OR Instinct8 OR Truex repo:seanchatmangpt/cargo-cicd"
)

# Get the latest Actions run status
actions_list(repo="seanchatmangpt/cargo-cicd", workflow_id="ci.yml")

# Fetch a specific XES evidence file at a historical commit
get_file_contents(
  repo="seanchatmangpt/cargo-cicd",
  path="target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes",
  ref="v26.6.2"
)
```

**Scope Restriction:**
The GitHub MCP server is scoped to `seanchatmangpt/cargo-cicd`. Cross-repository
operations (e.g., opening PRs against `wasm4pm` or `clap-noun-verb`) require explicit
permission grants or switching to a different session scope.

---

### `cargo-cicd-evidence`

**Type:** `filesystem`

**Roots:**
- `${workspaceRoot}/target/cargo-cicd/evidence`
- `${workspaceRoot}/receipts`

**Capabilities:** read, search (no write)

**Purpose:**
Provides scoped read-only access to all XES (XML Event Stream) evidence files and JSONL
process event companions emitted during cargo-cicd runs, plus the wasm4pm receipt artifacts
stored in `receipts/`.

This server allows Claude Code to inspect evidence without invoking raw Bash file reads.
Files are scoped strictly to the evidence output directory and the receipts directory —
Claude cannot accidentally read source code or credentials through this server.

**Typical File Patterns:**
- `target/cargo-cicd/evidence/evt-*.xes` — XML Event Stream files for wpm audit
- `target/cargo-cicd/evidence/evt-*.jsonl` — JSONL companions for machine parsing
- `receipts/*.json` — wpm receipt artifacts after adjudication

**When to Use:**
- Inspecting which events were emitted during a pipeline run
- Comparing `verdict_claimed` vs `verdict_adjudicated` to diagnose gate failures
- Reading receipt JSON before passing to `wpm receipt doctor`
- Searching evidence files for a specific `case_id` or `command` value
- Counting emitted events to verify completeness of an evidence set

**Example Invocations:**

```
# Read the most recently emitted XES file
Read("target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes")

# Search all evidence files for FAIL verdicts
Grep(pattern="verdict_claimed.*FAIL", path="target/cargo-cicd/evidence/", glob="*.jsonl")

# Read a receipt for validation
Read("receipts/receipt-publish-run-20260614.json")

# List all evidence files from the current session
Glob(pattern="target/cargo-cicd/evidence/evt-*.xes")
```

**Integration with wasm4pm:**
This server provides the input files consumed by the wasm4pm oracle. After using this
server to locate an XES file, invoke the oracle via Bash:

```bash
wpm audit target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes
# Output: Accept / Refuse / Blocked
```

**Important:** This server has `write: false`. Evidence files are written exclusively by
cargo-cicd's `ProcessEvent` serialization path (`src/evidence.rs`). Never modify evidence
files manually — the oracle will detect tampering and issue a `Refuse` verdict.

---

### `cargo-cicd-workspace`

**Type:** `filesystem`

**Roots:**
- `${workspaceRoot}` (workspace root)
- `${workspaceRoot}/crates` (sub-crate directory)

**Include Filter:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `cicd.toml`

**Capabilities:** read, search (no write)

**Purpose:**
Provides scoped read-only access to Cargo manifest files, the lockfile, toolchain
configuration, and the `cicd.toml` state carrier. This server is the primary interface
for workspace-level metadata queries — reading workspace members, dependency versions,
toolchain pins, and persisted engine state — without spawning `cargo metadata` or
invoking the full adapter stack.

**Typical File Patterns:**
- `Cargo.toml` — Root workspace manifest with `[workspace]` section
- `Cargo.lock` — Dependency lock file
- `rust-toolchain.toml` — Active toolchain pin (`[toolchain] channel = "stable"`)
- `cicd.toml` — Persisted EngineState (written by `CicdTomlWriter`)
- `crates/*/Cargo.toml` — Sub-crate manifests

**When to Use:**
- Reading workspace member names without invoking `cargo metadata`
- Checking the pinned toolchain channel before running `ToolchainDetector`
- Inspecting `cicd.toml` to understand last-known workspace state
- Verifying that all sub-crate `Cargo.toml` files have required publish metadata
  (name, version, description, license, readme)
- Comparing `Cargo.lock` dependency versions during a security review

**Example Invocations:**

```
# Read the workspace root manifest
Read("Cargo.toml")

# Read the cicd.toml state carrier
Read("cicd.toml")

# Read a specific sub-crate manifest
Read("crates/cargo-cicd-core/Cargo.toml")

# Search for missing license fields across all manifests
Grep(pattern="^license", path=".", glob="**/Cargo.toml", output_mode="files_with_matches")

# Find all workspace members declared
Grep(pattern="members\s*=", path="Cargo.toml", output_mode="content")

# Read toolchain pin
Read("rust-toolchain.toml")
```

**Integration with EngineState:**
The `cargo-cicd-workspace` MCP server is the read-side counterpart to the
`CargoMetadataAdapter` and `ManifestParser` adapters in `src/adapters/`. Prefer this
server for read-only inspection; the adapters are invoked at runtime by `EngineState::from_workspace()`.

**Note on `cicd.toml`:**
The `cicd.toml` file is a runtime artifact written by `CicdTomlWriter`. Reading it via
this server gives a snapshot of the last completed run — not live workspace state. For
live state, run `cargo cicd status show` or `cargo cicd workspace doctor`.

---

## Custom MCP Servers for cargo-cicd

The following four MCP servers are **hypothetical but highly useful** additions to the
cargo-cicd toolchain. They are not yet implemented as standalone MCP server processes;
their functionality is currently provided through a combination of Bash invocations and
the filesystem MCP servers above. This section documents them as a design specification
for future implementation or for contributors who want to build these integrations.

Each server is described with its full tool surface, invocation examples, and guidance
on when it is the right tool to reach for.

---

### 1. `cargo-workspace` MCP Server

**Type:** `process` (hypothetical — not yet deployed)

**Binary:** `cargo-workspace-mcp` (to be implemented in `crates/cargo-workspace-mcp/`)

**Purpose:**
Query Cargo workspace metadata without spawning the full `cargo metadata` command.
`cargo metadata` takes 1-3 seconds on large workspaces because it resolves all
transitive dependencies. The `cargo-workspace` MCP server instead performs targeted
line-by-line Cargo.toml parsing (matching the `CargoMetadataAdapter` approach) to
answer structural questions instantly.

This server is the MCP-native equivalent of the `CargoMetadataAdapter` and
`ManifestParser` adapters combined.

**Tools Exposed:**

#### `list_members`

```
list_members() -> Vec<WorkspaceMember>

WorkspaceMember {
  name: String,         // Package name from [package].name
  version: String,      // Package version
  path: String,         // Absolute path to crate directory
  manifest_path: String // Absolute path to Cargo.toml
}
```

Reads the root `Cargo.toml` `[workspace]` section and enumerates all declared members
by parsing their individual `Cargo.toml` files. Does not invoke `cargo` — pure TOML parsing.

**Example:**
```
list_members()
// Returns:
// [
//   { name: "cargo-cicd", version: "26.6.2", path: "/home/user/cargo-cicd", ... },
//   { name: "cargo-cicd-core", version: "0.1.0", path: "/home/user/cargo-cicd/crates/cargo-cicd-core", ... },
//   { name: "cargo-cicd-lsp", version: "0.1.0", path: "/home/user/cargo-cicd/crates/cargo-cicd-lsp", ... }
// ]
```

#### `get_manifest`

```
get_manifest(path: String) -> CargoManifest

CargoManifest {
  package: PackageMetadata,
  dependencies: HashMap<String, DependencySpec>,
  dev_dependencies: HashMap<String, DependencySpec>,
  build_dependencies: HashMap<String, DependencySpec>,
  features: HashMap<String, Vec<String>>,
  workspace: Option<WorkspaceSection>
}
```

Parses the `Cargo.toml` at the given path and returns structured metadata. Faster than
`cargo metadata --manifest-path` because it skips dependency resolution.

**Example:**
```
get_manifest("/home/user/cargo-cicd/Cargo.toml")
// Returns parsed manifest with package name, version, features, etc.

get_manifest("/home/user/cargo-cicd/crates/cargo-cicd-core/Cargo.toml")
// Returns core crate manifest
```

#### `check_dependencies`

```
check_dependencies(
  manifest_path: String,
  check_type: "publish_required" | "duplicates" | "version_constraints"
) -> DependencyCheckResult

DependencyCheckResult {
  pass: bool,
  issues: Vec<DependencyIssue>,
  recommendations: Vec<String>
}
```

Validates dependency declarations for common issues:
- `publish_required`: Checks that name, version, description, license, and readme are
  all present for publishable crates
- `duplicates`: Detects the same crate appearing in both `[dependencies]` and
  `[dev-dependencies]`
- `version_constraints`: Flags overly broad version constraints (e.g., `*` or
  `>=0.0.0`)

**Example:**
```
check_dependencies(
  manifest_path="/home/user/cargo-cicd/crates/cargo-cicd-lsp/Cargo.toml",
  check_type="publish_required"
)
// Returns { pass: false, issues: [{ field: "license", message: "missing required field" }] }
```

#### `find_workspace_root`

```
find_workspace_root(start_path: String) -> WorkspaceRootResult

WorkspaceRootResult {
  found: bool,
  root_path: Option<String>,
  manifest_path: Option<String>,
  discovery_method: "workspace_field" | "package_only" | "not_found"
}
```

Walks up the directory tree from `start_path` to locate the Cargo workspace root,
mimicking the algorithm Cargo itself uses. Useful when running from within a sub-crate
directory or a subdirectory of the workspace.

**Example:**
```
find_workspace_root("/home/user/cargo-cicd/crates/cargo-cicd-core/src")
// Returns { found: true, root_path: "/home/user/cargo-cicd", ... }
```

**When to Use `cargo-workspace` Server:**
- Enumerating workspace members faster than `cargo metadata`
- Validating publish metadata before running the publish gate
- Locating the workspace root when running from an unknown directory
- Checking for dependency declaration issues without a full `cargo check`
- Generating workspace topology for documentation or visualization

**Current Workaround (without this server):**
```bash
# list_members equivalent
grep -A 20 '\[workspace\]' Cargo.toml

# get_manifest equivalent
cat Cargo.toml

# find_workspace_root equivalent
git rev-parse --show-toplevel
```

---

### 2. `xes-evidence` MCP Server

**Type:** `process` (hypothetical — not yet deployed)

**Binary:** `xes-evidence-mcp` (to be implemented in `crates/xes-evidence-mcp/`)

**Purpose:**
Parse and query XES (XML Event Stream) evidence files using structured tools rather than
raw XML parsing via Bash. The `xes-evidence` MCP server provides a typed interface to
the process evidence emitted by cargo-cicd, enabling Claude Code to reason about
evidence content without regex-heavy shell commands.

This server is the MCP-native counterpart to reading evidence files through the
`cargo-cicd-evidence` filesystem server + Bash.

**XES File Structure (for reference):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <string key="case_id" value="status_show_phase"/>
    <event>
      <string key="event_id" value="evt-status-show-20260614134507123Z"/>
      <string key="timestamp" value="2026-06-14T13:45:07.123Z"/>
      <string key="lifecycle_transition" value="complete"/>
      <string key="verdict_claimed" value="PASS"/>
      <string key="trace_class" value="live_workspace"/>
      <string key="command" value="status show"/>
      <string key="workspace_id" value="cargo-cicd"/>
      <string key="duration_ms" value="42"/>
    </event>
  </trace>
</log>
```

**Tools Exposed:**

#### `list_evidence_files`

```
list_evidence_files(
  evidence_dir: String,
  filter: Optional<{
    format: "xes" | "jsonl" | "all",
    command: Optional<String>,
    since_iso: Optional<String>,
    verdict: Optional<"PASS" | "WARN" | "FAIL">
  }>
) -> Vec<EvidenceFileSummary>

EvidenceFileSummary {
  path: String,
  format: "xes" | "jsonl",
  event_id: String,
  command: String,
  timestamp_iso: String,
  size_bytes: u64,
  verdict_claimed: String
}
```

Lists evidence files with optional filtering by format, command, time range, or verdict.
Returns a structured summary without reading the full file content.

**Example:**
```
list_evidence_files(
  evidence_dir="/home/user/cargo-cicd/target/cargo-cicd/evidence",
  filter={ format: "xes", verdict: "FAIL" }
)
// Returns all XES files where verdict_claimed is FAIL
```

#### `parse_xes`

```
parse_xes(file_path: String) -> ParsedXes

ParsedXes {
  traces: Vec<XesTrace>,
  event_count: u64,
  case_ids: Vec<String>
}

XesTrace {
  case_id: String,
  events: Vec<XesEvent>
}

XesEvent {
  event_id: String,
  timestamp_iso: String,
  lifecycle_transition: "start" | "complete",
  verdict_claimed: String,
  verdict_adjudicated: Option<String>,
  command: String,
  workspace_id: String,
  duration_ms: Option<u64>,
  trace_class: String
}
```

Fully parses an XES file and returns all traces and events as structured data. Handles
multi-trace XES files (multiple `<trace>` elements with different `case_id` values).

**Example:**
```
parse_xes("/home/user/cargo-cicd/target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes")
// Returns: { traces: [{ case_id: "status_show_phase", events: [...] }], event_count: 2 }
```

#### `get_verdict`

```
get_verdict(file_path: String) -> VerdictSummary

VerdictSummary {
  file_path: String,
  event_id: String,
  command: String,
  verdict_claimed: String,
  verdict_adjudicated: Option<String>,
  adjudicated_at: Option<String>,
  adjudication_delta_ms: Option<u64>,
  lifecycle_complete: bool  // true if both start and complete events are present
}
```

Extracts just the verdict information from an XES file without loading the full trace
structure. Fast path for verdict checks.

**Example:**
```
get_verdict("/home/user/cargo-cicd/target/cargo-cicd/evidence/evt-publish-run-20260614.xes")
// Returns { verdict_claimed: "PASS", verdict_adjudicated: "Accept", lifecycle_complete: true }
```

#### `find_traces_by_case_id`

```
find_traces_by_case_id(
  evidence_dir: String,
  case_id: String
) -> Vec<TraceMatch>

TraceMatch {
  file_path: String,
  case_id: String,
  event_count: u64,
  first_event_at: String,
  last_event_at: String,
  verdicts: Vec<String>
}
```

Scans all XES files in the evidence directory and returns all traces matching the given
`case_id`. Useful for tracing a logical operation (e.g., `pipeline_run_phase`) across
multiple evidence files emitted during a single pipeline execution.

**Example:**
```
find_traces_by_case_id(
  evidence_dir="/home/user/cargo-cicd/target/cargo-cicd/evidence",
  case_id="pipeline_run_phase"
)
// Returns all XES files containing a <trace> with case_id="pipeline_run_phase"
```

**When to Use `xes-evidence` Server:**
- Diagnosing evidence gate failures by inspecting which events have mismatched verdicts
- Verifying that `start` and `complete` events are both present for a command
- Searching for evidence of a specific command across historical runs
- Checking `case_id` groupings are correct before oracle adjudication
- Building evidence summaries for human review without parsing XML manually

**Current Workaround (without this server):**
```bash
# list_evidence_files equivalent
ls -la target/cargo-cicd/evidence/

# parse_xes / get_verdict equivalent
cat target/cargo-cicd/evidence/evt-status-show-*.xes | grep -E "verdict_claimed|verdict_adjudicated"

# find_traces_by_case_id equivalent
grep -l 'value="pipeline_run_phase"' target/cargo-cicd/evidence/*.xes
```

---

### 3. `wasm4pm-oracle` MCP Server

**Type:** `process` (hypothetical — not yet deployed)

**Binary:** `wasm4pm-mcp` (wrapper around the `wpm` binary)

**Purpose:**
Provide a structured MCP interface to the wasm4pm evidence adjudication oracle, replacing
raw Bash invocations of `wpm audit` and `wpm receipt doctor`. This server wraps the `wpm`
binary and returns typed, structured results rather than raw text output that must be
parsed.

This is the highest-value hypothetical server for cargo-cicd: the oracle is the final
gate for every release, and a structured interface dramatically reduces the surface area
for misinterpretation of verdict output.

**Prerequisite:** The `wpm` binary must be on PATH. The server detects oracle availability
at startup and reports `oracle_available: false` in all responses when `wpm` is not found,
rather than erroring — matching the `Blocked` expected verdict pattern in tests.

**Tools Exposed:**

#### `audit_xes`

```
audit_xes(
  xes_file_path: String,
  options: Optional<{
    strict: bool,        // Fail on any warning (default: false)
    timeout_ms: u64,     // Oracle call timeout (default: 30000)
    trace_filter: Optional<String>  // Filter to specific case_id
  }>
) -> OracleVerdict

OracleVerdict {
  verdict: "Accept" | "Refuse" | "Blocked",
  xes_file: String,
  oracle_command: String,          // "wpm audit /path/to/file.xes"
  oracle_version: String,          // wpm --version output
  adjudicated_at: String,          // ISO 8601 timestamp
  duration_ms: u64,
  oracle_available: bool,
  refusal_reason: Option<String>,  // Set if verdict is Refuse
  raw_output: String               // Full wpm stdout for debugging
}
```

Invokes `wpm audit <xes_file_path>` and parses the result into a structured response.
Maps raw oracle output to the three canonical verdicts.

**Example:**
```
audit_xes(
  xes_file_path="/home/user/cargo-cicd/target/cargo-cicd/evidence/evt-publish-run-20260614.xes"
)
// Returns: { verdict: "Accept", oracle_available: true, duration_ms: 18, ... }

audit_xes(
  xes_file_path="/home/user/cargo-cicd/target/cargo-cicd/evidence/evt-corrupt-20260614.xes",
  options={ strict: true }
)
// Returns: { verdict: "Refuse", refusal_reason: "Invalid trace structure: missing complete event", ... }
```

#### `validate_receipt`

```
validate_receipt(
  receipt_path: String,
  options: Optional<{
    strict: bool,   // Enforce all required fields (default: true)
    format: "json"  // Receipt format (only "json" supported currently)
  }>
) -> ReceiptValidationResult

ReceiptValidationResult {
  verdict: "Accept" | "Refuse",
  receipt_path: String,
  oracle_command: String,    // "wpm receipt doctor --format json --strict /path"
  issues: Vec<String>,       // List of validation failures (empty if Accept)
  adjudicated_at: String,
  duration_ms: u64,
  oracle_available: bool
}
```

Invokes `wpm receipt doctor --format json --strict <receipt_path>` and returns a
structured validation result. Corresponds to the receipt validation step in the release
checklist.

**Example:**
```
validate_receipt(
  receipt_path="/home/user/cargo-cicd/receipts/receipt-publish-run-20260614.json",
  options={ strict: true, format: "json" }
)
// Returns: { verdict: "Accept", issues: [], duration_ms: 12 }

validate_receipt(
  receipt_path="/home/user/cargo-cicd/receipts/receipt-incomplete-20260614.json"
)
// Returns: { verdict: "Refuse", issues: ["missing required field: adjudicated_at"] }
```

#### `get_oracle_version`

```
get_oracle_version() -> OracleVersionInfo

OracleVersionInfo {
  available: bool,
  version: Option<String>,   // e.g., "wasm4pm 2.1.0"
  binary_path: Option<String>,
  capabilities: Vec<String>  // e.g., ["audit", "receipt doctor", "batch"]
}
```

Checks oracle availability and version without performing adjudication. Safe to call as
a pre-flight check before running the evidence gate.

**Example:**
```
get_oracle_version()
// Returns: { available: true, version: "wasm4pm 2.1.0", binary_path: "/usr/local/bin/wpm", capabilities: ["audit", "receipt doctor"] }
// Or if not installed:
// Returns: { available: false, version: null, binary_path: null }
```

#### `batch_audit`

```
batch_audit(
  evidence_dir: String,
  pattern: Optional<String>,   // Glob pattern (default: "*.xes")
  options: Optional<{
    strict: bool,
    fail_fast: bool,            // Stop on first Refuse (default: false)
    timeout_ms: u64
  }>
) -> BatchAuditResult

BatchAuditResult {
  total_files: u64,
  accepted: u64,
  refused: u64,
  blocked: u64,
  overall_verdict: "Accept" | "Refuse" | "Blocked",
  per_file_verdicts: Vec<OracleVerdict>,
  duration_ms: u64
}
```

Runs `wpm audit` against all XES files matching the pattern in the evidence directory
and aggregates results. If any file yields `Refuse`, the overall verdict is `Refuse`.
If the oracle is unavailable, the overall verdict is `Blocked`.

**Example:**
```
batch_audit(
  evidence_dir="/home/user/cargo-cicd/target/cargo-cicd/evidence",
  pattern="*.xes",
  options={ fail_fast: false }
)
// Returns: { total_files: 6, accepted: 6, refused: 0, blocked: 0, overall_verdict: "Accept" }

batch_audit(
  evidence_dir="/home/user/cargo-cicd/target/cargo-cicd/evidence",
  options={ fail_fast: true }
)
// Returns early on first Refuse:
// { total_files: 3, accepted: 2, refused: 1, overall_verdict: "Refuse", per_file_verdicts: [...] }
```

**When to Use `wasm4pm-oracle` Server:**
- Running the evidence gate before a release tag: `batch_audit` over all evidence files
- Diagnosing a specific gate failure: `audit_xes` on the failing XES file
- Validating receipts after `cargo cicd evidence doctor`: `validate_receipt`
- Pre-flight oracle availability check in CI: `get_oracle_version`
- Mutation test verification (confirming corrupt evidence is Refused): `audit_xes`

**Current Workaround (without this server):**
```bash
# audit_xes equivalent
wpm audit target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes

# validate_receipt equivalent
wpm receipt doctor --format json --strict receipts/receipt-publish-run-20260614.json

# get_oracle_version equivalent
wpm --version 2>/dev/null || echo "oracle unavailable"

# batch_audit equivalent
for f in target/cargo-cicd/evidence/*.xes; do wpm audit "$f"; done
```

---

### 4. `git-phase` MCP Server

**Type:** `process` (hypothetical — not yet deployed)

**Binary:** `git-phase-mcp` (thin wrapper around git commands)

**Purpose:**
Provide rich, typed git state queries beyond what `git status` and `git diff` return as
raw text. The `git-phase` MCP server encapsulates the logic in `GitStatusAdapter` and
`ChangedFileDetector`, returning structured data rather than porcelain output that must
be string-parsed.

This server models git state in terms meaningful to cargo-cicd: phases, file
classifications, and change sets — not raw git object identifiers.

**Git Phase Model:**
cargo-cicd tracks git state as a phase lifecycle:
- `clean` — No dirty, staged, or untracked files; branch is at or ahead of origin
- `dirty` — Unstaged changes present; not ready to push
- `staged` — Changes staged but not committed
- `committed` — Committed but not pushed
- `pushed` — All commits pushed; clean relative to remote
- `behind` — Local branch is behind origin/main; pull required

**Tools Exposed:**

#### `get_phase`

```
get_phase(
  workspace_root: String,
  base_ref: Optional<String>  // default: "origin/main"
) -> GitPhaseResult

GitPhaseResult {
  phase: "clean" | "dirty" | "staged" | "committed" | "pushed" | "behind",
  branch: String,
  ahead_count: u64,
  behind_count: u64,
  dirty_file_count: u64,
  staged_file_count: u64,
  untracked_file_count: u64,
  is_detached_head: bool,
  last_commit_sha: String,
  last_commit_message: String,
  last_commit_at: String
}
```

Returns a comprehensive git phase snapshot. This is the structured equivalent of
`cargo cicd git status` output.

**Example:**
```
get_phase(workspace_root="/home/user/cargo-cicd")
// Returns: { phase: "dirty", branch: "feat/workspace-validate", ahead_count: 2, dirty_file_count: 3, ... }

get_phase(workspace_root="/home/user/cargo-cicd", base_ref="origin/main")
// Returns: { phase: "committed", ahead_count: 1, behind_count: 0, dirty_file_count: 0, ... }
```

#### `list_dirty_files`

```
list_dirty_files(
  workspace_root: String,
  filter: Optional<{
    status: "M" | "A" | "D" | "R" | "?" | "all",  // porcelain status codes
    include_untracked: bool
  }>
) -> Vec<DirtyFile>

DirtyFile {
  path: String,
  relative_path: String,
  status: String,          // git porcelain status (M, A, D, R, ??)
  staged: bool,
  extension: String,
  is_rust_source: bool,
  is_test_file: bool,
  is_trybuild_fixture: bool
}
```

Returns structured dirty file metadata including cargo-cicd-specific classifications
(Rust source, test file, trybuild fixture). This is the structured equivalent of
`git status --porcelain` plus the classification logic from `ChangedFileDetector`.

**Example:**
```
list_dirty_files(workspace_root="/home/user/cargo-cicd")
// Returns: [
//   { path: "/home/user/cargo-cicd/src/nouns/workspace.rs", status: "M", is_rust_source: true, is_test_file: false },
//   { path: "/home/user/cargo-cicd/tests/invariants.rs", status: "M", is_rust_source: true, is_test_file: true }
// ]

list_dirty_files(workspace_root="/home/user/cargo-cicd", filter={ status: "?", include_untracked: true })
// Returns only untracked files
```

#### `ahead_behind_main`

```
ahead_behind_main(
  workspace_root: String,
  base_ref: Optional<String>  // default: "origin/main"
) -> AheadBehindResult

AheadBehindResult {
  ahead: u64,
  behind: u64,
  base_ref: String,
  local_branch: String,
  diverged: bool,    // true if both ahead > 0 and behind > 0
  needs_pull: bool,
  needs_push: bool,
  up_to_date: bool
}
```

Returns the ahead/behind commit counts relative to the base ref. Cleaner than parsing
`git rev-list --count` output from Bash.

**Example:**
```
ahead_behind_main(workspace_root="/home/user/cargo-cicd")
// Returns: { ahead: 2, behind: 0, needs_push: true, diverged: false }

ahead_behind_main(workspace_root="/home/user/cargo-cicd", base_ref="origin/release")
// Returns: { ahead: 0, behind: 5, needs_pull: true }
```

#### `list_changed_rs_files`

```
list_changed_rs_files(
  workspace_root: String,
  base_ref: Optional<String>,   // default: "origin/main"
  options: Optional<{
    include_deleted: bool,
    classify_tests: bool
  }>
) -> ChangedRustFiles

ChangedRustFiles {
  total_changed: u64,
  source_files: Vec<String>,       // Changed non-test .rs files
  test_files: Vec<String>,         // Changed test .rs files (tests/ directory or #[cfg(test)])
  trybuild_fixtures: Vec<String>,  // Changed files under tests/ui/
  deleted_files: Vec<String>,
  base_ref: String
}
```

Returns classified changed Rust files between the current branch and the base ref.
This is the MCP-native equivalent of `ChangedFileDetector` from `src/adapters/changed_file_detector.rs`.

**Example:**
```
list_changed_rs_files(workspace_root="/home/user/cargo-cicd")
// Returns: {
//   total_changed: 4,
//   source_files: ["src/nouns/workspace.rs", "src/engine/mod.rs"],
//   test_files: ["tests/cli/test_workspace.rs"],
//   trybuild_fixtures: ["tests/ui/compile_fail/missing_field.rs"],
//   base_ref: "origin/main"
// }
```

**When to Use `git-phase` Server:**
- Checking workspace git state before deciding whether to run `cargo cicd git close`
- Classifying changed files to determine which tests to run (`test changed` vs `test all`)
- Detecting trybuild fixture changes to trigger `cargo cicd trybuild changed`
- Verifying branch is clean before a release tag
- Diagnosing ahead/behind counts when the `branch_behind` policy fires

**Current Workaround (without this server):**
```bash
# get_phase equivalent
git status --porcelain && git rev-list --count HEAD ^origin/main

# list_dirty_files equivalent
git status --porcelain

# ahead_behind_main equivalent
git rev-list --count HEAD ^origin/main && git rev-list --count origin/main ^HEAD

# list_changed_rs_files equivalent
git diff origin/main --name-only | grep '\.rs$'
```

---

## YAML Plugin Bundle: `cargo-advanced-tools.yaml`

**File:** `.claude/plugins/cargo-advanced-tools.yaml`

**Type:** YAML plugin bundle definition (not an MCP server process)

**Purpose:**
`cargo-advanced-tools.yaml` is a declarative plugin bundle that defines structured tool
signatures for direct cargo command integration. Unlike the MCP filesystem and process
servers described above, this file is a **static YAML definition** — it declares tool
names, parameter schemas, return shapes, and feature compatibility metadata that the
cargo-cicd plugin toolkit (`plugins/cargo-cicd-kit/`) uses to generate or configure
typed tool wrappers.

The distinction from MCP servers is important:

| Aspect | MCP Servers (`.claude/mcp-servers.json`) | YAML Plugin Bundle (`cargo-advanced-tools.yaml`) |
|--------|------------------------------------------|--------------------------------------------------|
| Runtime model | Active server process or builtin | Static definition consumed by toolkit at build/load time |
| Protocol | MCP (Model Context Protocol) over stdio/HTTP | YAML parsed by plugin loader or code generator |
| Invocation | Claude calls tools via MCP protocol | Tools are registered as named wrappers around `cargo` CLI |
| Scope | Data access, external systems, oracle interfacing | Direct cargo command execution with typed parameters |
| Configuration | Declared in `mcp-servers.json` | Consumed by `plugins/cargo-cicd-kit/` bundle |

**Version:** 1.0.0 (matches cargo-cicd 26.6.2)

**MCP Namespace:** `cargo` (tool calls are prefixed `cargo.*` in the protocol namespace)

---

### Tools Exposed

The bundle declares eight tools under the `cargo` namespace:

#### `build_with_features`

Builds the workspace with specified feature flags. Supports individual features and
combinations (e.g., `"process-data,autonomic"`). Returns structured output including
success status, compiler warnings, errors, and duration.

**Key parameters:**
- `features` (required) — comma-separated feature list
- `release` — build in release mode (default: `false`)
- `all_targets` — build all targets (default: `false`)
- `workspace` — build all workspace members (default: `true`)

**Equivalent Bash:** `cargo build --features <features> [--release] [--all-targets]`

---

#### `test_with_filter`

Runs tests with optional scope and name filtering. Supports `all`, `lib`, `integration`,
`doc`, and `unit` scopes. Returns pass/fail counts, failed test details, and full output.

**Key parameters:**
- `scope` (required) — one of `all | lib | integration | doc | unit`
- `test_name` — optional substring filter for specific test names
- `features` — comma-separated features to enable
- `nocapture` — show test stdout/stderr (default: `false`)

**Equivalent Bash:** `cargo test [--test <name>] [--features <features>] [-- --nocapture]`

---

#### `check_all`

Runs `cargo check` across all workspace members and targets without producing binaries.
Faster than `build_with_features` for lint-only passes. Returns warning and error counts
with file/line locations.

**Key parameters:**
- `features` — optional feature selection
- `all_targets` — check all targets (default: `true`)
- `workspace` — check all members (default: `true`)
- `fix` — apply automatic fixes (default: `false`)

**Equivalent Bash:** `cargo check --workspace --all-targets [--features <features>]`

---

#### `analyze_workspace`

Extracts comprehensive workspace metadata including the full dependency graph, crate
structure, edition, build targets per crate, and the feature matrix across all members.
Takes no parameters; queries the workspace root automatically.

**Returns:** workspace root path, members list (name/path/version/edition/targets),
dependency graph, feature matrix per crate, resolver version.

**Equivalent Bash:** `cargo metadata --format-version 1`

---

#### `validate_features`

Validates that a specific feature combination compiles without conflicts. Resolves
transitive feature implications (e.g., `autonomic` implies `process-data`) and reports
compatibility notes. Performs a real compile check, not just a static graph walk.

**Key parameters:**
- `feature_combo` (required) — comma-separated features to validate
- `check_conflicts` — detect feature conflicts (default: `true`)

**Returns:** `valid` boolean, enabled features (including transitive), conflicts list,
implied features, compatibility notes, and compile test result.

**Feature implication map (from bundle):**
- `autonomic` → implies `process-data`
- `wasm4pm` → implies `process-data`
- `contrib` → implies `process-data`
- `advanced` → implies `process-data`

**Equivalent Bash:** `cargo check --features <feature_combo> 2>&1`

---

#### `clippy_suggestions`

Runs `cargo clippy` and returns lint suggestions organized by severity (`allow`, `warn`,
`deny`). Each suggestion includes lint name, file, line number, message, and fix hint.

**Key parameters:**
- `features` — features to enable during analysis
- `all_targets` — check all targets (default: `true`)
- `all_features` — enable all features (default: `false`)
- `fix` — apply fixes automatically (default: `false`)

**Equivalent Bash:** `cargo clippy --all-targets [--features <features>] [--fix]`

---

#### `doc_generation`

Generates rustdoc HTML documentation for the workspace. Supports feature-gated
documentation and optional inclusion of private items.

**Key parameters:**
- `features` — features to include in generated docs
- `all_features` — document with all features enabled (default: `false`)
- `document_private_items` — include private items (default: `false`)
- `open_browser` — open docs in browser after generation (default: `false`)

**Returns:** success status, output path to doc root, list of documented crates,
documentation warnings, duration.

**Equivalent Bash:** `cargo doc [--features <features>] [--document-private-items]`

---

#### `metadata_extraction`

Extracts detailed cargo metadata in structured form with an optional format selector
(`json`, `tree`, `compact`). Returns package information, dependency tree, and a
summary of total crates, dependencies, and workspace members.

**Key parameters:**
- `format` — output format: `json | tree | compact` (default: `json`)
- `include_dependencies` — include full dependency tree (default: `true`)

**Equivalent Bash:** `cargo metadata --format-version 1 [--no-deps]`

---

### How It Differs from MCP Servers

**MCP servers** (declared in `mcp-servers.json`) run as active processes or use Claude
Code builtins. They communicate over the MCP protocol and are invoked at runtime during
a Claude Code session.

**`cargo-advanced-tools.yaml`** is a static bundle definition. It does not run as a
server process. Instead, it serves as:

1. **A schema registry** — the YAML declares parameter types, required fields, and return
   shapes that the `plugins/cargo-cicd-kit/` toolkit uses to generate type-safe wrappers.
2. **A feature compatibility manifest** — the `features.available` and
   `features.recommended_combinations` sections document the cargo-cicd feature flag
   matrix in machine-readable form for validation tooling.
3. **An integration declaration** — the `integration.adapters` and `integration.nouns`
   sections declare which cargo-cicd internal adapters and noun modules are touched by
   each tool, enabling the plugin toolkit to generate correct dependency graphs.
4. **A performance configuration source** — rate limits, timeouts, caching TTLs, and
   parallelism limits are declared here and consumed by the plugin loader rather than
   hardcoded in individual tool implementations.

**When to reference this file:**
- Adding a new cargo tool wrapper to the plugin bundle: add a new entry under `tools:`
- Updating feature flag compatibility after adding a new feature: update
  `features.available` and `features.recommended_combinations`
- Diagnosing plugin rate limit or timeout issues: check the `performance:` and
  `error_handling:` sections
- Verifying which adapters a tool touches: see `integration.adapters`

**Plugin bundle location:** `.claude/plugins/cargo-advanced-tools.yaml`
**Distributable toolkit:** `plugins/cargo-cicd-kit/` — this YAML is the bundle seed for
the distributable toolkit intended for use in other Rust workspaces.

---

## Installation

### Adding a Configured MCP Server

To add a new MCP server to `.claude/mcp-servers.json`, follow this structure:

**Filesystem Server (read-only access to specific paths):**
```json
{
  "mcpServers": {
    "my-server-name": {
      "type": "filesystem",
      "description": "Short description of what this server provides",
      "roots": [
        "${workspaceRoot}/path/to/directory",
        "/absolute/path/if/needed"
      ],
      "include": ["*.toml", "*.json"],
      "capabilities": {
        "read": true,
        "write": false,
        "search": true
      }
    }
  }
}
```

**Process Server (wraps a local binary):**
```json
{
  "mcpServers": {
    "my-process-server": {
      "type": "process",
      "description": "Wraps the my-tool binary to provide structured tool calls",
      "command": "/usr/local/bin/my-tool-mcp",
      "args": ["--mode", "mcp"],
      "env": {
        "MY_TOOL_CONFIG": "${workspaceRoot}/.my-tool-config"
      }
    }
  }
}
```

**Builtin Server (Claude Code native integration):**
```json
{
  "mcpServers": {
    "github": {
      "type": "builtin",
      "description": "GitHub integration — no binary required"
    }
  }
}
```

### Variable Substitution

The following variables are available in `mcp-servers.json`:

| Variable | Expands to |
|----------|-----------|
| `${workspaceRoot}` | Absolute path to the workspace root (where `.claude/` lives) |
| `${userHome}` | Absolute path to the user's home directory |
| `${env:VAR_NAME}` | Value of environment variable `VAR_NAME` at session start |

### Validating a New Server

After adding a server to `mcp-servers.json`:

1. Restart the Claude Code session (the file is read at session start)
2. Verify the server is active by checking that its tool names appear in the available
   tool list
3. Run a simple tool call to confirm connectivity:
   ```
   # For filesystem servers: read a known file in the declared root
   # For process servers: call a lightweight tool like get_version or list
   ```
4. If the server fails to start, check:
   - Binary is on PATH (for process servers)
   - Roots exist and are accessible (for filesystem servers)
   - JSON syntax in `mcp-servers.json` is valid: `python3 -m json.tool .claude/mcp-servers.json`

---

## Security Notes

### MCP Servers Run with Session Permissions

MCP servers operate with the same filesystem and process permissions as the Claude Code
session. There is no privilege separation between MCP server processes and the parent
session. This means:

- A process MCP server can read any file the Claude Code user can read
- Filesystem servers are scoped by the `roots` and `include` declarations, but the
  underlying process is not sandboxed at the OS level
- Write-enabled filesystem servers (`"write": true`) can modify any file in the declared
  roots

**Guideline:** Declare all filesystem servers with `"write": false` unless write access
is explicitly required. For cargo-cicd, all configured servers are read-only.

### Validate All File Paths Before Passing to Tools

When constructing file paths to pass to MCP tools (especially the `xes-evidence` and
`wasm4pm-oracle` servers), always validate that the path:

1. Is within the expected directory (evidence directory, receipts directory, workspace root)
2. Matches the expected file extension (`.xes`, `.jsonl`, `.json`, `.toml`)
3. Does not contain path traversal sequences (`../`, `./`)
4. Is an absolute path (relative paths can resolve unexpectedly depending on working directory)

**Example validation pattern (Bash, pre-flight before tool call):**
```bash
EVIDENCE_FILE="target/cargo-cicd/evidence/evt-status-show-20260614134507123Z.xes"
EVIDENCE_DIR="/home/user/cargo-cicd/target/cargo-cicd/evidence"
ABSOLUTE_PATH="/home/user/cargo-cicd/${EVIDENCE_FILE}"

# Verify it's within the expected directory
if [[ "${ABSOLUTE_PATH}" != "${EVIDENCE_DIR}/"* ]]; then
  echo "ERROR: Path escapes evidence directory"
  exit 1
fi

# Verify it has the expected extension
if [[ "${ABSOLUTE_PATH}" != *.xes ]]; then
  echo "ERROR: Expected .xes extension"
  exit 1
fi
```

### Never Pass Forbidden Terms Through MCP Interfaces

The `FORBIDDEN TERMS` list in `CLAUDE.md` applies to all output channels, including MCP
tool call arguments, tool descriptions, and responses parsed from MCP servers.

Specifically:
- Do not pass forbidden terms as query strings to `search_code` on the GitHub MCP server
  unless searching for violations to eliminate them
- Do not include forbidden terms in PR titles, issue bodies, or commit messages created
  via GitHub MCP tools
- Do not construct XES event content that embeds forbidden terms in `command` or
  `verdict_claimed` fields, as this content may appear in evidence that is stored
  persistently

**Enforcement:** The `invariant_public_boundary_no_forbidden_terms_in_all_help()` test in
`tests/invariants.rs` scans `--help` output. Evidence files and receipts are currently
not scanned by this test, but future versions of the wasm4pm oracle may reject evidence
containing internal terminology.

### GitHub MCP Scope Restriction

The `github` MCP server is scoped to `seanchatmangpt/cargo-cicd`. Do not use GitHub MCP
tools to:
- Access private repositories that are not related to this project
- Fork the repository to external accounts without explicit authorization
- Push directly to `main` without a pull request review (use `create_pull_request`)
- Trigger Actions workflows on third-party repositories

### Evidence Immutability

Evidence files in `target/cargo-cicd/evidence/` are write-once artifacts. The
`cargo-cicd-evidence` MCP server enforces `"write": false` at the MCP configuration
level. Additionally:

- Never manually edit `.xes` or `.jsonl` evidence files — the wasm4pm oracle computes
  checksums and will issue a `Refuse` verdict for tampered files
- Never delete evidence files before running the evidence gate — the gate expects a
  complete set of events
- If you need to regenerate evidence (e.g., after a test run clears the directory), run
  the appropriate cargo-cicd verb to re-emit: `cargo cicd status show`,
  `cargo cicd evidence doctor`, etc.

---

## Integration with Subagents and Skills

The MCP servers described in this document are available to all subagents declared in
`.claude/subagents.json`. Each subagent has access to the full MCP tool set unless
further restricted.

### Evidence-Auditor Subagent

The `evidence-auditor` subagent (`.claude/subagents.json`) is the primary consumer of
the `cargo-cicd-evidence` MCP server and the `wasm4pm-oracle` hypothetical server.
When spawning this subagent, pass the specific XES file path or evidence directory
to scope its investigation:

```
Agent({
  subagent_type: "evidence-auditor",
  prompt: "Inspect all XES files in target/cargo-cicd/evidence/ for verdict discrepancies.
           Focus on files where verdict_claimed is PASS but wpm oracle returns Refuse.
           Use the cargo-cicd-evidence MCP server to read file contents."
})
```

### Release-Gate Skill

The `release-gate` skill (`.claude/skills.json`) invokes `cargo make test` and the
evidence gate tests. It depends on:
- `cargo-cicd-evidence` MCP server (to inspect evidence after tests run)
- `github` MCP server (to create the release tag after gate passes)
- Bash tool (to invoke `wpm audit` and `wpm receipt doctor`)

### Pipeline-Run Skill

The `pipeline-run` skill runs `cargo cicd test changed && cargo cicd workspace doctor &&
cargo cicd evidence doctor`. After the pipeline completes, the `cargo-cicd-evidence` MCP
server can be used to read the emitted evidence files without additional Bash invocations.

---

## Troubleshooting

### MCP Server Not Responding

If a configured MCP server is not responding to tool calls:

1. Verify `.claude/mcp-servers.json` is valid JSON:
   ```bash
   python3 -m json.tool /home/user/cargo-cicd/.claude/mcp-servers.json
   ```

2. For process servers, verify the binary is on PATH:
   ```bash
   which cargo-workspace-mcp 2>/dev/null || echo "not found"
   ```

3. For filesystem servers, verify the declared roots exist:
   ```bash
   ls -la /home/user/cargo-cicd/target/cargo-cicd/evidence/ 2>/dev/null || echo "directory missing"
   ls -la /home/user/cargo-cicd/receipts/ 2>/dev/null || echo "directory missing"
   ```
   Note: `target/cargo-cicd/evidence/` only exists after at least one cargo-cicd command
   has been run. Create it manually if needed:
   ```bash
   mkdir -p /home/user/cargo-cicd/target/cargo-cicd/evidence
   ```

4. Restart the Claude Code session — MCP servers are initialized at session start and
   are not hot-reloaded when `mcp-servers.json` changes.

### `cargo-cicd-evidence` Server Returns Empty Results

If the evidence MCP server finds no files:
- Verify at least one cargo-cicd command has been run to emit evidence
- Check the directory: `ls -la target/cargo-cicd/evidence/`
- If empty, run: `cargo cicd status show` to emit the first evidence file
- Verify the `include` filter in `mcp-servers.json` is not filtering out the files you expect

### GitHub MCP Returns 403 or 404

- Confirm the GitHub session is authenticated (`get_me` should return your username)
- Verify the repository `seanchatmangpt/cargo-cicd` is accessible to your account
- Check that the branch name passed to `create_branch` does not already exist
- For 404 on file reads: verify the `ref` parameter matches an existing commit SHA or branch

### `wasm4pm-oracle` Server Reports `oracle_available: false`

This is expected behavior when the `wpm` binary is not on PATH. Current workaround:

1. Locate or build the `wpm` binary
2. Add it to PATH: `export PATH="/path/to/wasm4pm/target/release:$PATH"`
3. Verify: `wpm --version`
4. Tests that require the oracle must declare `ExpectedWpmVerdict::Blocked` when oracle
   is unavailable — this is not an error state

---

## Reference

### MCP Server Configuration Schema (`mcp-servers.json`)

```json
{
  "mcpServers": {
    "<server-name>": {
      "type": "builtin" | "filesystem" | "process",
      "description": "<human-readable description>",

      // filesystem only:
      "roots": ["<path>", ...],
      "include": ["<glob>", ...],
      "capabilities": {
        "read": true | false,
        "write": true | false,
        "search": true | false
      },

      // process only:
      "command": "<binary-path>",
      "args": ["<arg>", ...],
      "env": { "<KEY>": "<value>" }
    }
  },
  "notes": {
    "<key>": "<free-text note>"
  }
}
```

### Evidence File Naming Convention

XES and JSONL evidence files follow this naming pattern:

```
evt-<command-slug>-<timestamp-compact>Z.<ext>

Examples:
  evt-status-show-20260614134507123Z.xes
  evt-status-show-20260614134507123Z.jsonl
  evt-publish-run-20260614140023456Z.xes
  evt-pipeline-run-20260614141200000Z.xes
```

- `<command-slug>`: noun and verb joined by `-` (spaces replaced with `-`)
- `<timestamp-compact>`: ISO 8601 timestamp with separators removed, milliseconds included
- `Z` suffix: UTC timezone indicator (always UTC)
- Extension: `.xes` for XML Event Stream, `.jsonl` for JSON Lines companion

### wpm Oracle Exit Code Contract

When the `wasm4pm-oracle` MCP server wraps `wpm`, it interprets exit codes as follows:

| Exit Code | Verdict | Meaning |
|-----------|---------|---------|
| `0` | `Accept` | Evidence is conformant; process is valid |
| `1` | `Refuse` | Evidence is non-conformant; process has violations |
| `2` | `Blocked` | Oracle cannot adjudicate (internal error, malformed XES, etc.) |
| Not found | `Blocked` | `wpm` binary not on PATH |

Tests in `tests/wasm4pm_evidence_gate.rs` assert on these verdicts using the
`ExpectedWpmVerdict` enum, never on raw exit codes.

---

*Last updated: 2026-06-16*
*Corresponds to cargo-cicd version: 26.6.2*
