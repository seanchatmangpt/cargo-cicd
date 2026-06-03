# CICD Diagnostic Codes

All diagnostic codes emitted by cargo-cicd-lsp follow the pattern `CICD-{DOMAIN}-{NNN}`.

Domains:
- `CFG` — cicd.toml configuration problems
- `TGT` — target scan and artifact problems
- `TCH` — toolchain problems
- `CHG` — changed-file coverage problems
- `GIT` — git phase and state problems
- `WRK` — workspace-level structural problems

Severities follow LSP conventions: `error`, `warning`, `information`, `hint`.

---

## CFG — Configuration

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-CFG-001 | `cicd.toml` missing | error | Creating a valid `cicd.toml` at the workspace root |
| CICD-CFG-002 | `cicd.toml` parse failure | error | Fixing the TOML syntax error identified in the diagnostic message |
| CICD-CFG-003 | `[workspace]` section missing | warning | Adding a `[workspace]` section with at least `name` |
| CICD-CFG-004 | `name` field empty or whitespace | warning | Setting a non-empty `name` in `[workspace]` |
| CICD-CFG-005 | Unknown key in `[state]` section | hint | Removing or renaming the unrecognised key |
| CICD-CFG-006 | `[[events]]` entry missing required field | warning | Adding the missing field (`event`, `timestamp`, or `artifact`) |
| CICD-CFG-007 | `[autonomic]` mode is not a recognised value | warning | Setting `mode` to `suggest`, `observe`, or `off` |
| CICD-CFG-008 | `cicd.toml` written by a future version | information | Upgrading cargo-cicd to the version that wrote the file |

---

## TGT — Targets

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-TGT-001 | No targets found in workspace | warning | Adding at least one `[lib]` or `[[bin]]` to a member `Cargo.toml` |
| CICD-TGT-002 | Target scan produced zero artifacts | error | Ensuring the workspace builds without error (`cargo make build`) |
| CICD-TGT-003 | Named target not found | error | Fixing the target name in `cicd.toml [target]` section |
| CICD-TGT-004 | Target artifact path does not exist | warning | Running a build to produce the artifact, or correcting the path |
| CICD-TGT-005 | Duplicate target name in workspace | warning | Renaming one of the duplicate targets |

---

## TCH — Toolchain

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-TCH-001 | `rust-toolchain.toml` missing | hint | Adding a `rust-toolchain.toml` to pin the channel |
| CICD-TCH-002 | Active toolchain does not match pinned channel | error | Running `rustup override set <channel>` or updating `rust-toolchain.toml` |
| CICD-TCH-003 | Required component missing | error | Running `rustup component add <component>` |
| CICD-TCH-004 | Toolchain detection failed | warning | Ensuring `rustup` is installed and on `$PATH` |

---

## CHG — Changed Files

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-CHG-001 | Changed file has no test coverage mapping | information | Adding a test that exercises the changed file, or updating coverage config |
| CICD-CHG-002 | Changed files exceed coverage threshold | warning | Adding tests until the threshold is met |
| CICD-CHG-003 | Changed file detector could not determine base ref | warning | Setting a valid `base_ref` in `cicd.toml [state]` |

---

## GIT — Git Phase

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-GIT-001 | Not in a git repository | error | Initialising a git repository (`git init`) |
| CICD-GIT-002 | Working tree has uncommitted changes that block release | warning | Committing or stashing changes |
| CICD-GIT-003 | Current branch is not push-ready | warning | Resolving the condition reported in the diagnostic detail |
| CICD-GIT-004 | Git phase closure not reached | error | Ensuring all required phase events are present in `[[events]]` |
| CICD-GIT-005 | Merge conflict markers present | error | Resolving all merge conflicts in the identified file |

---

## WRK — Workspace

| Code | Title | Severity | Cleared By |
|------|-------|----------|------------|
| CICD-WRK-001 | Workspace root `Cargo.toml` is not a virtual manifest | hint | Adding `[workspace]` to the root manifest |
| CICD-WRK-002 | Member crate not listed in workspace members | warning | Adding the crate path to the `members` array in the root manifest |
| CICD-WRK-003 | Workspace `Cargo.lock` missing | information | Running `cargo build` to generate the lockfile |
| CICD-WRK-004 | Cargo metadata resolution failed | error | Fixing the manifest error reported by `cargo metadata` |

---

## Notes on Severity

- **error** — the condition blocks a push-ready state; CI would fail
- **warning** — the condition degrades readiness and should be resolved before release
- **information** — informational only; CI is unlikely to fail but the condition is worth knowing
- **hint** — best-practice suggestion; no direct CI impact expected
