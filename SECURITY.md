# Security Policy & Guidelines for cargo-cicd

This document provides security-first guidance for contributors and reviewers. All pull requests must satisfy the **Security Review Checklist** before merge.

---

## Overview

cargo-cicd is a Level 5 process-data engine exposed as a Rust CI/CD helper. It:

- Reads workspace metadata (`Cargo.toml`, `rust-toolchain.toml`)
- Invokes subprocess commands (`cargo`, `git`, `wpm`)
- Writes state to `cicd.toml` and XES evidence files
- Runs policies and autonomic suggestions (never destructive in default mode)
- Integrates with wasm4pm oracle for evidence-gate closure

Security concerns center on **subprocess injection**, **path traversal**, **log leakage**, and **unsafe code**. This guide enforces defense-in-depth practices for each domain.

---

## 1. Security Review Checklist

Use this checklist for all PRs modifying `src/`, `src/adapters/`, `src/nouns/`, or `src/integrations/`.

- [ ] **No SQL injection** — not applicable, but audit all `exec` strings and shell invocations
- [ ] **No command injection** — all `Command::new()` calls use `.arg()`, never format user input into strings
- [ ] **No path traversal** — all user-supplied paths validated with `Path::canonicalize()` or whitelist checks
- [ ] **No hardcoded secrets** — no API keys, crates.io tokens, or wpm credentials in code
- [ ] **No sensitive data in logs** — RUST_LOG output sanitized; no workspace paths, file names, or environment variables logged
- [ ] **Cryptographic operations correct** — blake3 usage: document assumptions, verify nonce-freshness if applicable
- [ ] **No unsafe code** — unless performance-critical; every `unsafe` block has `// SAFETY: <justification>` and passes audit
- [ ] **Dependencies vetted** — `cargo audit` passes; new dependencies reviewed for maintenance status
- [ ] **Forbidden terms absent** — no ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 in public-facing docs
- [ ] **Input validation present** — file paths, environment variables, subprocess args all validated before use

---

## 2. Command Injection Prevention

### Rule: Never format user input into shell commands

**Bad:**
```rust
// FORBIDDEN: user input directly in format string
let cmd = format!("cargo {}", user_provided_args);
std::process::Command::new("sh").arg("-c").arg(cmd).status()?;
```

**Good:**
```rust
// Safe: structured argument passing
let mut cmd = std::process::Command::new("cargo");
for arg in parsed_args {
    cmd.arg(arg);  // Each arg is a separate OS-level parameter
}
cmd.status()?;
```

### Rule: Use `Command::new()` with `.arg()`, never `.arg("-c string")`

Cargo-cicd invokes subprocesses in:
- `src/integrations/wasm4pm_shell.rs` — `Command::new(&self.binary).args(args).output()?`
- `src/main.rs` — `std::process::Command::new(bin).args(rest).status()?`

These are **safe** because arguments come from `std::env::args()` (not user input) or parsed clap arguments (validated).

When integrating new subprocess calls:
```rust
// Example: invoke cargo in target directory
let output = std::process::Command::new("cargo")
    .arg("test")
    .arg("--test")
    .arg(test_name)  // Passed through clap, already validated
    .current_dir(&workspace_root)
    .output()?;
```

---

## 3. Path Traversal Prevention

### Rule: Validate all paths with `canonicalize()` or whitelist

Cargo-cicd reads:
- `Cargo.toml`, `rust-toolchain.toml` from workspace root
- `cicd.toml` from workspace root
- Target directory (`target/`)
- Fixture directories in tests

**Safe patterns:**

```rust
// Pattern 1: Canonicalize user-provided paths
fn read_user_config(path: &str) -> Result<String> {
    let canonical = std::fs::canonicalize(path)?;
    // Ensure path is within workspace
    let workspace_root = std::env::current_dir()?;
    if !canonical.starts_with(&workspace_root) {
        anyhow::bail!("path traversal detected: {:?}", path);
    }
    std::fs::read_to_string(&canonical)
}

// Pattern 2: Build paths from safe components
let target_dir = workspace_root.join("target");  // Safe: .join() normalizes
let evidence_dir = target_dir.join("cargo-cicd").join("evidence");

// Pattern 3: Never concatenate paths with strings
// Bad:  Path::new(&format!("{}/{}", root, user_input))
// Good: Path::new(root).join(user_input)
```

### Symlink Handling

If reading symlinks, resolve them:
```rust
let resolved = std::fs::canonicalize(symlink_path)?;
// Now safe to read resolved
```

---

## 4. Secrets & Credentials

### Rule: Never hardcode, never log

**Forbidden in code:**
- crates.io publish tokens
- wasm4pm oracle credentials
- GitHub PAT, SSH keys
- API keys of any kind

**Safe patterns:**

```rust
// Load credentials from environment, not code
let wpm_bin = std::env::var("WPM_BIN").unwrap_or_else(|_| "wpm".into());
let wpm_path = std::env::var("WPM_PATH").ok();

// Redact before logging
let safe_log = if token.contains("sk_") {
    format!("sk_{}...{}", &token[..3], &token[token.len()-3..])
} else {
    token.clone()
};
eprintln!("Token: {}", safe_log);
```

### Environment Variables

- `WPM_BIN` — path to wasm4pm binary (safe, validated with `which`)
- `WPM_PATH` — wasm4pm oracle socket (safe, used as-is)
- `CARGO_TARGET_DIR` — inherited from cargo; validated in adapters
- `RUST_LOG` — never log file contents or workspace paths

---

## 5. Logging Security

### Rule: No secrets, no PII, no workspace structure

**Bad:**
```rust
eprintln!("Reading {}", file_path);  // Leaks workspace structure
eprintln!("Token: {}", token);       // Leaks credential
```

**Good:**
```rust
eprintln!("Reading config file");           // Generic
eprintln!("Token: ***");                    // Redacted
eprintln!("Workspace: {}", workspace.name); // Name only, not path
```

### Structured Logging

When `tracing` feature is enabled (`advanced` feature):
```rust
use tracing::debug;

debug!(count = changed_files.len(), "Scanning changed files");  // Structured
debug!(file_count = 42, "All safe");
```

Output: `file_count=42 msg="All safe"` — JSON-parseable, no shell injection.

Never construct JSON manually:
```rust
// Bad: shell-injectable
let json = format!(r#"{{"path": "{}"}}"#, user_path);

// Good: serde handles escaping
let event = serde_json::json!({"path": user_path});
```

---

## 6. Cryptographic Operations

### blake3 (optional, `advanced` feature)

Blake3 is used for **fast hashing**, not encryption. It's safe as-is:

```rust
use blake3::Hasher;

fn hash_file(path: &Path) -> Result<[u8; 32]> {
    let content = std::fs::read(path)?;
    Ok(blake3::hash(&content).as_bytes().clone())
}
```

**Rules:**
- Never use blake3 for passwords (use `argon2` or `scrypt` if adding auth)
- Document why blake3 was chosen (speed, tree hashing, etc.)
- Don't rely on blake3 alone for integrity; pair with signatures for crates.io artifacts

### Nonce Freshness

Cargo-cicd does not use encryption or nonces. If future versions add encryption:
- Generate nonces with `getrandom` or `rand`
- Never reuse nonces under the same key
- Document nonce scope (per-message, per-session, etc.)

---

## 7. Unsafe Code Policy

### Rule: Unsafe is forbidden except in performance-critical, audited paths

Cargo-cicd currently has **no unsafe code** (by design). If unsafe is required:

1. **Justify in a comment** — `// SAFETY: <why this is safe>`
2. **Document invariants** — what must be true for this code to be sound
3. **Encapsulate in safe API** — wrap in a safe function, don't expose `unsafe` to callers
4. **Minimal scope** — smallest possible block

**Example (hypothetical):**
```rust
// SAFETY: We control the buffer lifetime and ensure it's only accessed
// while mutable references are held. This unsafe is needed to avoid
// boxing overhead in hot path (benchmarks show 15% improvement).
unsafe {
    let ptr = buf.as_mut_ptr();
    libc::memset(ptr as *mut _, 0, buf.len());
}
```

### Audit Trail

Every unsafe block must:
- Have a GitHub issue tracking the audit
- Be reviewed by at least two maintainers
- Include a benchmark showing the performance gain

---

## 8. Dependency Security

### Pre-merge: `cargo audit`

Every PR must pass:
```sh
cargo audit
cargo audit --deny warnings
```

This runs in CI; local developers should check before pushing:
```sh
cargo audit
# If advisory found, update Cargo.lock or patch the dependency
```

### New Dependencies

For any new `Cargo.toml` addition:
1. Check maintenance status: GitHub commits in last 6 months?
2. Review dependency graph: `cargo tree --depth 3`
3. Check size: is this 500KB+ of code for one function?
4. Verify it's used (no accidental pulls)

**Discouraged (bloat):**
- `regex` (unless core to feature, use `aho-corasick` for multi-pattern scanning)
- Full HTTP clients if we only need one endpoint
- Serialization frameworks beyond `serde`

**Encouraged:**
- `anyhow` / `thiserror` (error handling)
- `serde` + `toml` (config parsing)
- `walkdir` (directory traversal, safe-by-design)
- `tempfile` (secure temporary files, not `/tmp`)

### Lock File

`Cargo.lock` is checked in. Never run `cargo update` on a release branch; changes must be reviewed PR-by-PR.

---

## 9. Secure File Handling

### Rule: Use Rust stdlib, never shell out to `rm`, `cp`, `mkdir`

**Bad:**
```rust
// FORBIDDEN: shell invocation + potential injection
std::process::Command::new("sh")
    .arg("-c")
    .arg(format!("rm -rf {}", dir))
    .status()?;
```

**Good:**
```rust
// Safe: stdlib handles all edge cases
std::fs::remove_dir_all(&dir)?;
std::fs::create_dir_all(&parent)?;
std::fs::copy(&src, &dst)?;
```

### Temporary Files

Always use `tempfile` crate, never write to `/tmp` or `std::env::temp_dir()` unguarded:

```rust
use tempfile::NamedTempFile;

// Safe: auto-cleaned, unique name, restricted permissions
let mut tmp = NamedTempFile::new()?;
tmp.write_all(b"data")?;
tmp.flush()?;
// Cleaned up when `tmp` drops
```

### Permissions

Respect filesystem umask; don't make files world-readable:

```rust
use std::os::unix::fs::PermissionsExt;

let file = std::fs::File::create(path)?;
// Default umask applied automatically
// If you need tighter perms:
let perms = std::fs::Permissions::from_mode(0o600);  // Owner only
std::fs::set_permissions(path, perms)?;
```

### Evidence Files

XES evidence written to `target/cargo-cicd/evidence/`:
```rust
// src/evidence.rs
fn emit_xes_impl(events: &[ProcessEvent], path: &Path, filter: bool) -> Result<()> {
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("invalid path"))?;
    std::fs::create_dir_all(parent)?;  // Safe
    std::fs::write(path, xes_xml)?;    // Safe
    Ok(())
}
```

---

## 10. Process Security

### Subprocess Timeouts

If invoking cargo, git, or wpm, enforce a timeout:

```rust
use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;

fn run_cargo_with_timeout(args: &[&str], timeout: Duration) -> Result<std::process::Output> {
    let mut child = Command::new("cargo")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;
    
    thread::sleep(timeout);
    if child.try_wait()?.is_none() {
        child.kill()?;  // SIGKILL if still running
        anyhow::bail!("cargo command timed out");
    }
    
    Ok(child.wait_with_output()?)
}
```

### Subprocess Arguments Validation

- Arguments from clap are validated
- Arguments from env are sanitized (`WPM_BIN` checked with `which`)
- Never pass raw file paths as cargo package names

---

## 11. CI/CD Security

### GitHub Actions (if applicable)

- **Never commit secrets**: use GitHub Secrets for crates.io token, wasm4pm credentials
- **Pin action versions**: `actions/checkout@v4` not `@latest`
- **Review PRs before merge**: no auto-merge for untrusted contributors
- **Artifact signing**: sign release artifacts with `cosign` or GitHub release signatures

### Release Flow

1. Bump version in `Cargo.toml`
2. Run `cargo audit` (must pass)
3. Run full test suite: `cargo make test` + wasm4pm evidence-gate tests
4. Sign commit: `git commit -S -m "..."`
5. Tag: `git tag -s v26.6.2 -m "Release v26.6.2"`
6. Push tags
7. CI builds and publishes crate with signature

### Build Artifact Verification

Before installation, verify the published crate:
```sh
cargo install cargo-cicd --locked
# Verify checksum in crates.io registry
```

---

## 12. Evidence & wasm4pm Integration

### XES Evidence Format

Evidence is emitted as XML Event Stream (XES), not JSON. This is intentional:
- XES is standardized (IEEE 1849)
- Avoids JSON injection risks
- Integrates with process mining tools

**Safe patterns:**
```rust
// src/evidence.rs
fn emit_xes_impl(events: &[ProcessEvent], path: &Path, _filter: bool) -> Result<()> {
    // XML escaping handled by library
    let xml = construct_xes(events)?;
    std::fs::write(path, xml)?;  // Safe: bytes, no interpretation
    Ok(())
}
```

### wasm4pm Oracle Integration

Never trust wpm output alone; validate:

```rust
// Run wpm audit and receipt doctor
let audit_output = Command::new("wpm")
    .args(&["audit", evidence_path.to_str().unwrap()])
    .output()?;

if !audit_output.status.success() {
    anyhow::bail!("wpm audit failed: {}", String::from_utf8_lossy(&audit_output.stderr));
}

// Parse JSON verdict (if present)
let verdict: WpmVerdict = serde_json::from_slice(&audit_output.stdout)?;
assert_eq!(verdict.status, "Accept");
```

---

## 13. Configuration Security

### cicd.toml

`cicd.toml` is the carrier for workspace state. Protect it:

- **Committed to repo**: yes (it's state, not secrets)
- **Contains credentials**: no (environment variables only)
- **Readable by team**: yes (state is non-sensitive)
- **Writable by untrusted users**: no (validate in `CicdTomlWriter`)

### Safe Parsing

TOML parsing is safe (no code execution):
```rust
let toml: CicdToml = toml::from_str(&content)?;
// Events and state are just data, no risk
```

### Configuration Validation

Before using a config value, validate:
```rust
impl CicdToml {
    pub fn validate(&self) -> Result<()> {
        // Ensure workspace name is not a path traversal attempt
        if self.workspace.name.contains("..") || self.workspace.name.contains("/") {
            anyhow::bail!("invalid workspace name");
        }
        Ok(())
    }
}
```

---

## 14. Governance & Forbidden Terms

### Public-Facing Restrictions

The following terms are **forbidden** in:
- CLI help text
- README.md
- Public documentation
- Error messages
- Log output (when sensitive)

**Forbidden terms:** ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8

These are governance/manufacturing terms not for public use.

**Rationale**: Cargo-cicd is a "boring" CI/CD tool; these terms reveal the Level 5 engine nature, which is private.

### Internal vs. Public Boundaries

- Public: `src/nouns/`, `src/main.rs`, `README.md`
- Private (Level 5): `src/engine/`, `src/policies/`, `src/autonomic/`, `src/evidence/`

Features guard the boundary:
- `process-data` feature: unlocks internal state
- `autonomic` feature: implies `process-data`, unlocks policies
- `advanced` feature: unlocks tracing, blake3, performance optimizations

---

## 15. Security Resources & References

### OWASP Top 10

- **A03: Injection** — mitigated by Command struct, TOML parsing
- **A01: Broken Access Control** — N/A (no auth), but validate paths
- **A02: Cryptographic Failures** — no encryption; blake3 for hashing only

### Rust Security

- [The Rustonomicon: unsafe code](https://doc.rust-lang.org/nomicon/)
- [CWE-78: Improper Neutralization of Special Elements used in an OS Command](https://cwe.mitre.org/data/definitions/78.html)
- [OWASP Path Traversal](https://owasp.org/www-community/attacks/Path_Traversal)

### Crate Audit

```sh
# Check for known vulnerabilities
cargo audit

# See transitive dependencies
cargo tree --duplicate

# Check license compliance (if needed)
cargo license
```

### Tools

- `cargo clippy` — lint pass (run before PR)
- `cargo fmt` — code style (enforced in CI)
- `cargo audit` — dependency vulnerabilities (enforced in CI)
- `cargo test` — unit + integration tests (enforced in CI)

---

## 16. Reporting Security Issues

If you discover a security vulnerability in cargo-cicd:

1. **Do not open a public issue**
2. **Email security report to**: maintainer (check `Cargo.toml` for contact)
3. **Include**: description, reproduction steps, affected versions, suggested fix
4. **Wait** for acknowledgment (48 hours)
5. **Embargo**: vulnerability stays private until patch released

We will:
- Acknowledge receipt within 24 hours
- Provide a timeline for patch
- Credit you in release notes (unless you prefer anonymity)

---

## 17. Pre-Merge Checklist

Every PR author must self-review:

- [ ] `cargo fmt` passes (`cargo fmt -- --check`)
- [ ] `cargo clippy` passes (`cargo clippy`)
- [ ] `cargo audit` passes
- [ ] `cargo make test` passes (all integration tests)
- [ ] Security checklist (section 1) passed
- [ ] No hardcoded secrets or credentials
- [ ] No unsafe code (or audited with SAFETY comment)
- [ ] No forbidden terms in public docs
- [ ] commit message follows `feat(core|cli|...)` format

Reviewers will:
- Run the above checks
- Verify command injection paths (section 2)
- Verify path handling (section 3)
- Check for new unsafe code (section 7)
- Audit new dependencies (section 8)

---

## 18. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-06-14 | Initial security policy; covers v26.6.2 |

---

## Questions?

Contact the maintainers via GitHub issues or email. Keep security discussions private; use the process in section 16.
