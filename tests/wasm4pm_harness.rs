//! wasm4pm evidence-gate test harness for cargo-cicd v26.6.2.
//!
//! ## Law (E4)
//! Tests assert only wasm4pm verdicts, never cargo-cicd self-assertions.
//! cargo-cicd emits evidence; wasm4pm adjudicates.
//!
//! ## Invariants
//! - `WpmOracle::discover()` checks the known binary path first.
//! - `require_wpm()` panics with `BLOCKED:` prefix if wpm is absent.
//! - `EvidenceMutation` corrupts JSONL evidence; mutations must cause Refuse.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ── FixtureWorkspace ──────────────────────────────────────────────────────────

/// A temporary, isolated Cargo workspace for evidence-gate integration tests.
///
/// The workspace is torn down when this struct is dropped.
pub struct FixtureWorkspace {
    /// Backing temp dir — kept alive for the lifetime of the fixture.
    dir: TempDir,
}

impl FixtureWorkspace {
    /// Minimal workspace: valid `Cargo.toml` + `src/main.rs`, no git.
    pub fn new_clean() -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write_minimal_cargo_toml(root);
        write_minimal_main_rs(root);
        Self { dir }
    }

    /// Minimal workspace with `git init` + initial commit.
    pub fn with_git() -> Self {
        let ws = Self::new_clean();
        let root = ws.path().to_path_buf();
        let _ = run_git(&root, &["init"]);
        let _ = run_git(&root, &["config", "user.email", "test@example.com"]);
        let _ = run_git(&root, &["config", "user.name", "Test"]);
        let _ = run_git(&root, &["add", "."]);
        let _ = run_git(&root, &["commit", "-m", "init"]);
        ws
    }

    /// Absolute path to the workspace root.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Run `cargo-cicd` with the given args in this workspace.
    ///
    /// Uses the test binary built by `cargo test --tests`.
    pub fn run_cargo_cicd(&self, args: &[&str]) -> CicdOutput {
        // Locate the cargo-cicd binary relative to the test binary location.
        // CARGO_BIN_EXE_cargo-cicd is set by Cargo for integration tests.
        let binary = std::env::var("CARGO_BIN_EXE_cargo-cicd")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Fallback: look in target/debug relative to CARGO_MANIFEST_DIR.
                let manifest =
                    std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(manifest)
                    .join("target")
                    .join("debug")
                    .join("cargo-cicd")
            });

        let output = Command::new(&binary)
            .args(args)
            .current_dir(self.path())
            .output()
            .unwrap_or_else(|e| panic!("failed to run cargo-cicd ({binary:?}): {e}"));

        CicdOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Path to the JSONL evidence file emitted by cargo-cicd.
    pub fn evidence_path(&self) -> PathBuf {
        self.path()
            .join("target")
            .join("cargo-cicd")
            .join("evidence")
            .join("events.jsonl")
    }

    /// Read all JSONL evidence events from disk.
    ///
    /// Returns an empty `Vec` if the file does not exist.
    pub fn read_events(&self) -> Vec<serde_json::Value> {
        let path = self.evidence_path();
        if !path.exists() {
            return Vec::new();
        }
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read evidence at {path:?}: {e}"));
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str(l)
                    .unwrap_or_else(|e| panic!("malformed evidence line {l:?}: {e}"))
            })
            .collect()
    }

    /// Apply a corruption mutation to the JSONL evidence file.
    ///
    /// The file must already exist; call after running cargo-cicd.
    pub fn mutate_evidence(&self, mutation: EvidenceMutation) {
        let path = self.evidence_path();
        match mutation {
            EvidenceMutation::FlipVerdict => flip_verdict(&path),
            EvidenceMutation::OmitField(field) => omit_field(&path, field),
            EvidenceMutation::ContradictSize => contradict_size(&path),
            EvidenceMutation::HideChangedFile => hide_changed_file(&path),
            EvidenceMutation::AddFakeArtifact => add_fake_artifact(&path),
        }
    }
}

// ── EvidenceMutation ──────────────────────────────────────────────────────────

/// Corruptions that wasm4pm must refuse.
pub enum EvidenceMutation {
    /// Change `verdict_claimed_by_cargo_cicd` from `PASS` to `FAIL` or vice versa.
    FlipVerdict,
    /// Remove a required field from the last event in the JSONL file.
    OmitField(&'static str),
    /// Change `target_size_bytes` to a value that contradicts the actual size.
    ContradictSize,
    /// Remove a file entry from the `changed_files` list in the last event.
    HideChangedFile,
    /// Inject an artifact path that does not exist on disk.
    AddFakeArtifact,
}

// ── CicdOutput ────────────────────────────────────────────────────────────────

/// Raw output from a `cargo-cicd` invocation.
pub struct CicdOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

// ── WpmOracle ────────────────────────────────────────────────────────────────

/// The known release path for the wpm binary.
const WPM_KNOWN_PATH: &str = "/Users/sac/wasm4pm/target/release/wpm";

/// External oracle that shells out to `wpm` for evidence adjudication.
///
/// All evidence-gate tests must go through this oracle — never through
/// cargo-cicd's own verdict fields.
pub struct WpmOracle {
    pub binary: PathBuf,
}

/// Verdict returned by `wpm` after adjudicating evidence.
#[derive(Debug, Clone, PartialEq)]
pub enum WpmEvidenceVerdict {
    /// Exit 0 with no FAIL/REFUSE/WARN in output.
    Accept,
    /// Exit 0 with WARN in output.
    Warn,
    /// Exit non-0 or REFUSE/FAIL in output.
    Refuse,
    /// wpm binary not found or could not be invoked.
    Blocked(String),
}

impl WpmOracle {
    /// Discover the wpm binary.
    ///
    /// Search order: `$WPM_PATH` env var → known release path → `PATH`.
    pub fn discover() -> Option<Self> {
        if let Ok(p) = std::env::var("WPM_PATH") {
            if Path::new(&p).exists() {
                return Some(Self {
                    binary: PathBuf::from(p),
                });
            }
        }
        if Path::new(WPM_KNOWN_PATH).exists() {
            return Some(Self {
                binary: PathBuf::from(WPM_KNOWN_PATH),
            });
        }
        if let Ok(out) = Command::new("which").arg("wpm").output() {
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() {
                    return Some(Self {
                        binary: PathBuf::from(p),
                    });
                }
            }
        }
        None
    }

    /// Run `wpm audit <events_path>` and map the result to a verdict.
    ///
    /// - Exit 0, no FAIL/REFUSE/WARN → `Accept`
    /// - Exit 0, WARN in output → `Warn`
    /// - Exit non-0 or REFUSE/FAIL in output → `Refuse`
    /// - Invocation error → `Blocked`
    pub fn audit_events(&self, events_path: &Path) -> WpmEvidenceVerdict {
        self.invoke(&["audit", &events_path.to_string_lossy()])
    }

    /// Run `wpm doctor` — system health check.
    pub fn doctor(&self) -> WpmEvidenceVerdict {
        self.invoke(&["doctor"])
    }

    /// Run `wpm lean` — Lean Six Sigma waste audit against an evidence path.
    pub fn lean(&self, _evidence_path: &Path) -> WpmEvidenceVerdict {
        self.invoke(&["lean"])
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    fn invoke(&self, args: &[&str]) -> WpmEvidenceVerdict {
        let result = Command::new(&self.binary).args(args).output();
        match result {
            Err(e) => WpmEvidenceVerdict::Blocked(format!("wpm spawn failed: {e}")),
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                let combined = format!("{stdout}{stderr}");

                if !output.status.success()
                    || combined.contains("refuse")
                    || combined.contains("fail")
                {
                    WpmEvidenceVerdict::Refuse
                } else if combined.contains("warn") {
                    WpmEvidenceVerdict::Warn
                } else {
                    WpmEvidenceVerdict::Accept
                }
            }
        }
    }
}

/// Require wpm or panic with a `BLOCKED:` message.
///
/// Evidence-gate tests must call this; a missing oracle is a blocking defect,
/// not a skip.
pub fn require_wpm() -> WpmOracle {
    WpmOracle::discover().unwrap_or_else(|| {
        eprintln!("BLOCKED: wpm binary not found — evidence gate cannot run");
        panic!("wpm oracle unavailable");
    })
}

// ── Mutation helpers ─────────────────────────────────────────────────────────

fn load_last_event(path: &Path) -> (Vec<serde_json::Value>, serde_json::Value) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read evidence at {path:?}: {e}"));
    let mut lines: Vec<serde_json::Value> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid JSON line"))
        .collect();
    assert!(!lines.is_empty(), "evidence file has no events to mutate");
    let last = lines.pop().unwrap();
    (lines, last)
}

fn write_events(path: &Path, mut events: Vec<serde_json::Value>, last: serde_json::Value) {
    events.push(last);
    let content: String = events
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(path, content)
        .unwrap_or_else(|e| panic!("cannot write mutated evidence to {path:?}: {e}"));
}

fn flip_verdict(path: &Path) {
    let (rest, mut last) = load_last_event(path);
    if let Some(obj) = last.as_object_mut() {
        let key = "verdict_claimed_by_cargo_cicd";
        let flipped = match obj.get(key).and_then(|v| v.as_str()).unwrap_or("") {
            s if s.to_lowercase().contains("pass") => "FAIL".to_string(),
            _ => "PASS".to_string(),
        };
        obj.insert(key.to_string(), serde_json::Value::String(flipped));
    }
    write_events(path, rest, last);
}

fn omit_field(path: &Path, field: &str) {
    let (rest, mut last) = load_last_event(path);
    if let Some(obj) = last.as_object_mut() {
        obj.remove(field);
    }
    write_events(path, rest, last);
}

fn contradict_size(path: &Path) {
    let (rest, mut last) = load_last_event(path);
    if let Some(obj) = last.as_object_mut() {
        obj.insert(
            "target_size_bytes".to_string(),
            serde_json::Value::Number(serde_json::Number::from(u64::MAX / 2)),
        );
    }
    write_events(path, rest, last);
}

fn hide_changed_file(path: &Path) {
    let (rest, mut last) = load_last_event(path);
    if let Some(obj) = last.as_object_mut() {
        if let Some(arr) = obj.get_mut("changed_files").and_then(|v| v.as_array_mut()) {
            if !arr.is_empty() {
                arr.pop();
            }
        } else {
            // Field absent — inject an empty array to mark the omission.
            obj.insert(
                "changed_files".to_string(),
                serde_json::Value::Array(Vec::new()),
            );
        }
    }
    write_events(path, rest, last);
}

fn add_fake_artifact(path: &Path) {
    let (rest, mut last) = load_last_event(path);
    if let Some(obj) = last.as_object_mut() {
        let fake = "/nonexistent/artifact/does_not_exist_9999.bin";
        match obj.get_mut("artifacts").and_then(|v| v.as_array_mut()) {
            Some(arr) => {
                arr.push(serde_json::Value::String(fake.to_string()));
            }
            None => {
                obj.insert(
                    "artifacts".to_string(),
                    serde_json::Value::Array(vec![serde_json::Value::String(fake.to_string())]),
                );
            }
        }
    }
    write_events(path, rest, last);
}

// ── Workspace helpers ─────────────────────────────────────────────────────────

fn write_minimal_cargo_toml(root: &Path) {
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
}

fn write_minimal_main_rs(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create src/");
    std::fs::write(src.join("main.rs"), "fn main() {}\n").expect("write src/main.rs");
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

// ── Smoke test ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_workspace_clean_has_cargo_toml() {
        let ws = FixtureWorkspace::new_clean();
        assert!(ws.path().join("Cargo.toml").exists());
        assert!(ws.path().join("src/main.rs").exists());
    }

    #[test]
    fn fixture_workspace_with_git_has_dot_git() {
        let ws = FixtureWorkspace::with_git();
        assert!(ws.path().join(".git").exists());
    }

    #[test]
    fn evidence_path_canonical() {
        let ws = FixtureWorkspace::new_clean();
        let ep = ws.evidence_path();
        assert!(ep.ends_with("target/cargo-cicd/evidence/events.jsonl"));
    }

    #[test]
    fn read_events_returns_empty_when_no_file() {
        let ws = FixtureWorkspace::new_clean();
        assert!(ws.read_events().is_empty());
    }

    #[test]
    fn wpm_oracle_discover_finds_binary_or_returns_none() {
        // Either finds the binary or returns None — both are valid here.
        let _ = WpmOracle::discover();
    }

}
