//! Standing ingestors: turn evidence lying around the workspace (doctor
//! output, OCEL process-validation logs, receipt ledgers, plan-run
//! artifacts, benchmark raw files, claim tables, client build results)
//! into [`StandingArtifact`] records.
//!
//! ## Tolerance contract
//!
//! Every `ingest_*` function here is infallible in the panic sense: a
//! missing file, an unparseable command, or a timed-out build never panics
//! and never aborts the caller's refresh. When a configured source is
//! absent or empty, the artifact still gets a standing entry — `DISCOVERED`
//! if the configured path exists on disk, `UNSEEN` otherwise — so a refresh
//! always documents what it looked for, not just what it found.

use crate::standing::glob;
use crate::standing::model::{ArtifactKind, EvidenceRef, StandingArtifact, StandingStatus};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn now_iso() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", dur.as_secs())
}

/// FNV-1a fan-out proxy hash — mirrors the same std-only algorithm used by
/// `blake3_hex` in the main crate's `src/ocel.rs`. Duplicated intentionally
/// (a handful of lines) rather than depending on the main crate from here,
/// since `cargo-cicd-core` sits below `cargo-cicd` in the dependency graph.
fn proxy_hash_hex(data: &[u8]) -> String {
    let mut h: [u64; 4] = [
        0xcbf29ce484222325u64,
        0x9e3779b97f4a7c15u64,
        0x6c62272e07bb0142u64,
        0x517cc1b727220a95u64,
    ];
    for (i, &b) in data.iter().enumerate() {
        let lane = i % 4;
        h[lane] ^= b as u64;
        h[lane] = h[lane].wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h[0], h[1], h[2], h[3])
}

/// Build the tolerant fallback artifact for a configured-but-absent source:
/// `DISCOVERED` if `path` exists on disk, `UNSEEN` otherwise.
fn fallback_artifact(id: &str, kind: ArtifactKind, path: &str) -> StandingArtifact {
    let standing = if glob::path_exists(path) {
        vec![StandingStatus::Discovered]
    } else {
        vec![StandingStatus::Unseen]
    };
    StandingArtifact {
        id: id.to_string(),
        kind,
        path: path.to_string(),
        standing,
        scope: None,
        ladder_level: 0,
        evidence: vec![],
        external_operator_side_effects: vec![],
    }
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

// ── 1. Doctor JSON ────────────────────────────────────────────────────────

/// Shell out to `doctor_command` (e.g. `"cargo cicd doctor check --format
/// json"`) and ingest the `{build, config, frontier, receipts, tools,
/// features}` shape. `frontier.pass_rate` and `receipts.record_count` are
/// recorded as evidence; a truthy `build` field additionally contributes
/// `BUILDS` to the standing set.
pub fn ingest_doctor_json(command: Option<&str>) -> Vec<StandingArtifact> {
    let Some(command) = command else {
        return vec![fallback_artifact("doctor-report", ArtifactKind::Doc, "")];
    };

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => {
            let mut a = fallback_artifact("doctor-report", ArtifactKind::Doc, "");
            a.evidence.push(EvidenceRef::Command {
                command: command.to_string(),
                exit_code: -1,
                utc: now_iso(),
                artifact: None,
            });
            return vec![a];
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Option<serde_json::Value> = serde_json::from_str(&stdout).ok();

    let mut standing = vec![StandingStatus::Discovered];
    let mut evidence = vec![EvidenceRef::Command {
        command: command.to_string(),
        exit_code,
        utc: now_iso(),
        artifact: None,
    }];

    if let Some(v) = &parsed {
        let build_truthy = match v.get("build") {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::Null) | None => false,
            Some(_) => true,
        };
        if build_truthy {
            standing.push(StandingStatus::Builds);
        }
        if let Some(pass_rate) = v.pointer("/frontier/pass_rate") {
            evidence.push(EvidenceRef::Artifact {
                path: "doctor:frontier.pass_rate".to_string(),
                hash: pass_rate.to_string(),
            });
        }
        if let Some(record_count) = v.pointer("/receipts/record_count") {
            evidence.push(EvidenceRef::Artifact {
                path: "doctor:receipts.record_count".to_string(),
                hash: record_count.to_string(),
            });
        }
    }

    vec![StandingArtifact {
        id: "doctor-report".to_string(),
        kind: ArtifactKind::Doc,
        path: command.to_string(),
        standing,
        scope: None,
        ladder_level: 0,
        evidence,
        external_operator_side_effects: vec![],
    }]
}

// ── 2. OCEL process-validation ───────────────────────────────────────────

/// Glob for OCEL process-validation JSON files matching `{is_conforming,
/// fitness, violations, ...}` and ingest one artifact per file.
pub fn ingest_ocel_process_validation(globs: &[String]) -> Vec<StandingArtifact> {
    if globs.is_empty() {
        return vec![fallback_artifact(
            "ocel-process-validation",
            ArtifactKind::Workflow,
            "",
        )];
    }

    let mut out = vec![];
    for pattern in globs {
        let matches = glob::expand(pattern);
        if matches.is_empty() {
            out.push(fallback_artifact(
                &format!("ocel-process-validation:{pattern}"),
                ArtifactKind::Workflow,
                pattern,
            ));
            continue;
        }
        for path in matches {
            out.push(ingest_one_process_validation(&path));
        }
    }
    out
}

fn ingest_one_process_validation(path: &Path) -> StandingArtifact {
    let id = format!("ocel:{}", file_stem(path));
    let path_str = path.to_string_lossy().to_string();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let parsed: Option<serde_json::Value> = serde_json::from_str(&content).ok();

    let Some(v) = parsed else {
        return StandingArtifact {
            id,
            kind: ArtifactKind::Workflow,
            path: path_str.clone(),
            standing: vec![StandingStatus::Discovered],
            scope: None,
            ladder_level: 0,
            evidence: vec![EvidenceRef::Artifact {
                path: path_str,
                hash: "unparseable".to_string(),
            }],
            external_operator_side_effects: vec![],
        };
    };

    let is_conforming = v
        .get("is_conforming")
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    let mut standing = vec![StandingStatus::Discovered];
    if is_conforming {
        standing.push(StandingStatus::OcelProven);
    }

    let event_id = v
        .get("event_id")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| file_stem(path));

    let mut evidence = vec![EvidenceRef::OcelEvent {
        event_id,
        path: path_str.clone(),
    }];
    if let Some(fitness) = v.get("fitness") {
        evidence.push(EvidenceRef::Artifact {
            path: format!("{path_str}:fitness"),
            hash: fitness.to_string(),
        });
    }

    StandingArtifact {
        id,
        kind: ArtifactKind::Workflow,
        path: path_str,
        standing,
        scope: None,
        ladder_level: 0,
        evidence,
        external_operator_side_effects: vec![],
    }
}

// ── 3. Receipt ledgers (JSONL) ───────────────────────────────────────────

/// Ingest JSONL receipt ledgers, one record per line, each carrying a
/// `chain_hash_hex` field.
pub fn ingest_receipt_ledgers(paths: &[String]) -> Vec<StandingArtifact> {
    if paths.is_empty() {
        return vec![fallback_artifact("receipt-ledgers", ArtifactKind::Doc, "")];
    }

    paths.iter().map(|p| ingest_one_receipt_ledger(p)).collect()
}

fn ingest_one_receipt_ledger(path: &str) -> StandingArtifact {
    if !glob::path_exists(path) {
        return fallback_artifact(
            &format!("receipt-ledger:{}", file_stem(Path::new(path))),
            ArtifactKind::Doc,
            path,
        );
    }

    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut evidence = vec![];
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(hash) = v.get("chain_hash_hex").and_then(|h| h.as_str()) {
                evidence.push(EvidenceRef::Receipt {
                    chain_hash: hash.to_string(),
                    path: path.to_string(),
                });
            }
        }
    }

    let standing = if evidence.is_empty() {
        vec![StandingStatus::Discovered]
    } else {
        vec![StandingStatus::Receipted]
    };

    StandingArtifact {
        id: format!("receipt-ledger:{}", file_stem(Path::new(path))),
        kind: ArtifactKind::Doc,
        path: path.to_string(),
        standing,
        scope: None,
        ladder_level: 0,
        evidence,
        external_operator_side_effects: vec![],
    }
}

// ── 4. Plan-run artifacts ────────────────────────────────────────────────

/// Glob for `plan.json` plan-run artifacts carrying a `powl_chain_hash`.
pub fn ingest_plan_runs(pattern: Option<&str>) -> Vec<StandingArtifact> {
    let Some(pattern) = pattern else {
        return vec![fallback_artifact("plan-runs", ArtifactKind::Workflow, "")];
    };

    let matches = glob::expand(pattern);
    if matches.is_empty() {
        return vec![fallback_artifact(
            &format!("plan-runs:{pattern}"),
            ArtifactKind::Workflow,
            pattern,
        )];
    }

    matches.iter().map(|p| ingest_one_plan_run(p)).collect()
}

fn ingest_one_plan_run(path: &Path) -> StandingArtifact {
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_stem(path));
    let id = format!("plan:{parent_name}");
    let path_str = path.to_string_lossy().to_string();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let parsed: Option<serde_json::Value> = serde_json::from_str(&content).ok();

    let chain_hash = parsed
        .as_ref()
        .and_then(|v| v.get("powl_chain_hash"))
        .and_then(|h| h.as_str())
        .map(|s| s.to_string());

    match chain_hash {
        Some(hash) => StandingArtifact {
            id,
            kind: ArtifactKind::Workflow,
            path: path_str.clone(),
            standing: vec![StandingStatus::Discovered, StandingStatus::Receipted],
            scope: None,
            ladder_level: 0,
            evidence: vec![EvidenceRef::Receipt {
                chain_hash: hash,
                path: path_str,
            }],
            external_operator_side_effects: vec![],
        },
        None => StandingArtifact {
            id,
            kind: ArtifactKind::Workflow,
            path: path_str,
            standing: vec![StandingStatus::Discovered],
            scope: None,
            ladder_level: 0,
            evidence: vec![],
            external_operator_side_effects: vec![],
        },
    }
}

// ── 5. Benchmark raw files ───────────────────────────────────────────────

/// Glob for benchmark raw-output files. Deliberately does not attempt to
/// parse divan's ns/us table syntax — file existence + size/hash is
/// sufficient `BENCHMARKED` evidence.
pub fn ingest_bench_raw(pattern: Option<&str>) -> Vec<StandingArtifact> {
    let Some(pattern) = pattern else {
        return vec![fallback_artifact("bench-raw", ArtifactKind::Bench, "")];
    };

    let matches = glob::expand(pattern);
    if matches.is_empty() {
        return vec![fallback_artifact(
            &format!("bench-raw:{pattern}"),
            ArtifactKind::Bench,
            pattern,
        )];
    }

    matches
        .iter()
        .map(|path| {
            let path_str = path.to_string_lossy().to_string();
            let bytes = std::fs::read(path).unwrap_or_default();
            let hash = proxy_hash_hex(&bytes);
            StandingArtifact {
                id: format!("bench:{}", file_stem(path)),
                kind: ArtifactKind::Bench,
                path: path_str.clone(),
                standing: vec![StandingStatus::Discovered, StandingStatus::Benchmarked],
                scope: None,
                ladder_level: 0,
                evidence: vec![EvidenceRef::Artifact {
                    path: path_str,
                    hash,
                }],
                external_operator_side_effects: vec![],
            }
        })
        .collect()
}

// ── 6. Claim tables (markdown pipe-tables) ───────────────────────────────

/// Parse markdown pipe-table rows from claim-promotion-style docs.
///
/// Rows are informational only — this ingestor never elevates standing
/// beyond `DISCOVERED` from a claim table alone, since claims are not
/// authoritative evidence.
pub fn ingest_claim_tables(paths: &[String]) -> Vec<StandingArtifact> {
    if paths.is_empty() {
        return vec![fallback_artifact("claim-tables", ArtifactKind::Doc, "")];
    }
    paths.iter().map(|p| ingest_one_claim_table(p)).collect()
}

/// Parse the pipe-table rows out of markdown content. Returns `Vec<row
/// cells>`, skipping header separator rows (`---`) and blank lines.
pub fn parse_pipe_table_rows(content: &str) -> Vec<Vec<String>> {
    content
        .lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .map(|l| {
            l.trim()
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|cells: &Vec<String>| {
            !cells
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':') && !c.is_empty())
        })
        .collect()
}

fn ingest_one_claim_table(path: &str) -> StandingArtifact {
    if !glob::path_exists(path) {
        return fallback_artifact(
            &format!("claim:{}", file_stem(Path::new(path))),
            ArtifactKind::Doc,
            path,
        );
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let rows = parse_pipe_table_rows(&content);
    StandingArtifact {
        id: format!("claim:{}", file_stem(Path::new(path))),
        kind: ArtifactKind::Doc,
        path: path.to_string(),
        standing: vec![StandingStatus::Discovered],
        scope: None,
        ladder_level: 0,
        evidence: vec![EvidenceRef::Artifact {
            path: path.to_string(),
            hash: format!("rows={}", rows.len()),
        }],
        external_operator_side_effects: vec![
            "claim table rows are informational; not authoritative standing evidence".to_string(),
        ],
    }
}

// ── 7. Client build results ──────────────────────────────────────────────

/// One client build target: a path plus the shell command that builds it.
/// Reused as-is by the main crate's `cicd.toml` `[standing]` deserializer —
/// defined here (rather than duplicated in the main crate) since it is pure
/// data with no dependency on `cargo-cicd-core` internals.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientTarget {
    pub id: String,
    pub path: String,
    pub build_command: String,
}

/// Run each client's configured `build_command` with a timeout, recording
/// `BUILDS`/exit-code evidence. Never panics: a missing path, a spawn
/// failure, or a timeout all resolve to a fallback/failed artifact rather
/// than aborting the batch.
pub fn ingest_client_builds(clients: &[ClientTarget], timeout: Duration) -> Vec<StandingArtifact> {
    if clients.is_empty() {
        return vec![fallback_artifact("clients", ArtifactKind::Client, "")];
    }
    clients
        .iter()
        .map(|c| ingest_one_client_build(c, timeout))
        .collect()
}

fn ingest_one_client_build(client: &ClientTarget, timeout: Duration) -> StandingArtifact {
    if !glob::path_exists(&client.path) {
        return fallback_artifact(&client.id, ArtifactKind::Client, &client.path);
    }

    let (exit_code, timed_out) = run_with_timeout(&client.build_command, &client.path, timeout);

    let mut standing = vec![StandingStatus::Discovered];
    if exit_code == Some(0) {
        standing.push(StandingStatus::Builds);
    }

    let mut side_effects = vec![];
    if timed_out {
        side_effects.push(format!(
            "build_command for client '{}' exceeded {:?} and was killed",
            client.id, timeout
        ));
    }

    StandingArtifact {
        id: client.id.clone(),
        kind: ArtifactKind::Client,
        path: client.path.clone(),
        standing,
        scope: None,
        ladder_level: 0,
        evidence: vec![EvidenceRef::Command {
            command: client.build_command.clone(),
            exit_code: exit_code.unwrap_or(-1),
            utc: now_iso(),
            artifact: Some(client.path.clone()),
        }],
        external_operator_side_effects: side_effects,
    }
}

/// Spawn `command` via `sh -c` in `cwd`, polling for completion up to
/// `timeout`. Returns `(exit_code, timed_out)`; `exit_code` is `None` if the
/// process could not even be spawned.
fn run_with_timeout(command: &str, cwd: &str, timeout: Duration) -> (Option<i32>, bool) {
    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return (None, false),
    };

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status.code(), false),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return (None, true);
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => return (None, false),
        }
    }
}

// ── 8. Workspace member crates ────────────────────────────────────────────

/// Discover Rust workspace member crates from the root `Cargo.toml`'s
/// `[workspace] members` list and emit one `RustCrate` artifact per member.
///
/// Conservative by design: this ingestor never shells out to `cargo
/// build`/`test`/`clippy`, so it never fabricates a `BUILDS`/`TESTED`/
/// `LINT_CLEAN` status it cannot attribute to an actual command run.
/// Every discovered member gets `DISCOVERED` standing only — its
/// `Cargo.toml` parses and names a crate, nothing more is claimed. A caller
/// wanting `BUILDS`/`TESTED` evidence per crate should pair this with a
/// targeted `doctor_command` or a future per-crate build/test ingestor.
pub fn ingest_workspace_crates(repo_root: &str) -> Vec<StandingArtifact> {
    let root_cargo_toml = format!("{repo_root}/Cargo.toml");
    let Ok(root_toml_str) = std::fs::read_to_string(&root_cargo_toml) else {
        return vec![fallback_artifact(
            "workspace-crates",
            ArtifactKind::RustCrate,
            "",
        )];
    };
    let Ok(root_toml) = root_toml_str.parse::<toml::Value>() else {
        return vec![fallback_artifact(
            "workspace-crates",
            ArtifactKind::RustCrate,
            &root_cargo_toml,
        )];
    };

    let member_patterns: Vec<String> = root_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    if member_patterns.is_empty() {
        return vec![fallback_artifact(
            "workspace-crates",
            ArtifactKind::RustCrate,
            &root_cargo_toml,
        )];
    }

    let member_dirs = resolve_member_dirs(repo_root, &member_patterns);
    if member_dirs.is_empty() {
        return vec![fallback_artifact(
            "workspace-crates",
            ArtifactKind::RustCrate,
            &root_cargo_toml,
        )];
    }

    member_dirs
        .iter()
        .filter_map(|dir| ingest_one_workspace_crate(repo_root, dir))
        .collect()
}

/// Expand each member pattern to concrete member directories relative to
/// `repo_root`. Supports literal paths (`"."`, `"crates/foo"`) and a single
/// trailing `/*` glob segment (`"crates/*"`) — the two shapes actually used
/// by cargo workspace manifests in this fleet. Deliberately narrower than
/// `crate::standing::glob`, which matches files, not directories.
fn resolve_member_dirs(repo_root: &str, patterns: &[String]) -> Vec<PathBuf> {
    let mut out = vec![];
    for pattern in patterns {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let base = Path::new(repo_root).join(prefix);
            let Ok(entries) = std::fs::read_dir(&base) else {
                continue;
            };
            let mut subdirs: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.join("Cargo.toml").is_file())
                .collect();
            subdirs.sort();
            out.extend(subdirs);
        } else {
            let dir = Path::new(repo_root).join(pattern);
            if dir.join("Cargo.toml").is_file() {
                out.push(dir);
            }
        }
    }
    out
}

fn ingest_one_workspace_crate(repo_root: &str, dir: &Path) -> Option<StandingArtifact> {
    let cargo_toml_path = dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path).ok()?;
    let parsed: toml::Value = content.parse().ok()?;
    let name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())?;

    let rel_path = dir
        .strip_prefix(repo_root)
        .unwrap_or(dir)
        .to_string_lossy()
        .to_string();
    let rel_path = if rel_path.is_empty() {
        ".".to_string()
    } else {
        rel_path
    };

    let hash = proxy_hash_hex(content.as_bytes());

    Some(StandingArtifact {
        id: format!("crate:{name}"),
        kind: ArtifactKind::RustCrate,
        path: rel_path.clone(),
        standing: vec![StandingStatus::Discovered],
        scope: None,
        ladder_level: 0,
        evidence: vec![EvidenceRef::Artifact {
            path: format!("{rel_path}/Cargo.toml"),
            hash,
        }],
        external_operator_side_effects: vec![],
    })
}

/// Path to a fixture under `crates/cargo-cicd-core/fixtures/standing/`.
#[cfg(test)]
fn fixture(name: &str) -> String {
    format!("{}/fixtures/standing/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_json_fixture_yields_builds_and_evidence() {
        let cmd = format!("cat {}", fixture("doctor.json"));
        let out = ingest_doctor_json(Some(&cmd));
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::Builds));
        assert!(out[0].evidence.iter().any(
            |e| matches!(e, EvidenceRef::Artifact { path, .. } if path == "doctor:frontier.pass_rate")
        ));
    }

    #[test]
    fn process_validation_fixture_yields_ocel_proven() {
        let out = ingest_ocel_process_validation(&[fixture("process-validation.json")]);
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::OcelProven));
        assert!(out[0].evidence.iter().any(
            |e| matches!(e, EvidenceRef::OcelEvent { event_id, .. } if event_id == "case-fixture-1")
        ));
    }

    #[test]
    fn receipt_log_fixture_yields_two_receipts() {
        let out = ingest_receipt_ledgers(&[fixture("receipt-log.jsonl")]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Receipted]);
        assert_eq!(out[0].evidence.len(), 2);
    }

    #[test]
    fn doctor_json_none_command_is_unseen() {
        let out = ingest_doctor_json(None);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn doctor_json_parses_frontier_and_receipts_as_evidence() {
        let cmd = r#"echo '{"build": true, "frontier": {"pass_rate": 0.92}, "receipts": {"record_count": 7}}'"#;
        let out = ingest_doctor_json(Some(cmd));
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::Builds));
        assert!(out[0]
            .evidence
            .iter()
            .any(|e| matches!(e, EvidenceRef::Artifact { path, .. } if path == "doctor:frontier.pass_rate")));
        assert!(out[0]
            .evidence
            .iter()
            .any(|e| matches!(e, EvidenceRef::Artifact { path, .. } if path == "doctor:receipts.record_count")));
    }

    #[test]
    fn doctor_json_tolerates_non_json_output() {
        let out = ingest_doctor_json(Some("echo not-json"));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Discovered]);
    }

    #[test]
    fn ocel_validation_empty_globs_is_unseen() {
        let out = ingest_ocel_process_validation(&[]);
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn ocel_validation_conforming_file_yields_ocel_proven() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("case-1.json");
        std::fs::write(&file, r#"{"is_conforming": true, "fitness": 0.99}"#).unwrap();
        let out = ingest_ocel_process_validation(&[file.to_str().unwrap().to_string()]);
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::OcelProven));
    }

    #[test]
    fn ocel_validation_non_conforming_stays_discovered() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("case-2.json");
        std::fs::write(&file, r#"{"is_conforming": false, "fitness": 0.2}"#).unwrap();
        let out = ingest_ocel_process_validation(&[file.to_str().unwrap().to_string()]);
        assert!(!out[0].standing.contains(&StandingStatus::OcelProven));
        assert_eq!(out[0].standing, vec![StandingStatus::Discovered]);
    }

    #[test]
    fn receipt_ledger_missing_path_is_unseen() {
        let out = ingest_receipt_ledgers(&["/no/such/ledger.jsonl".to_string()]);
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn receipt_ledger_extracts_chain_hash_hex() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ledger.jsonl");
        std::fs::write(
            &file,
            "{\"chain_hash_hex\": \"abc123\"}\n{\"chain_hash_hex\": \"def456\"}\n",
        )
        .unwrap();
        let out = ingest_receipt_ledgers(&[file.to_str().unwrap().to_string()]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Receipted]);
        assert_eq!(out[0].evidence.len(), 2);
    }

    #[test]
    fn plan_run_none_pattern_is_unseen() {
        let out = ingest_plan_runs(None);
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn plan_run_with_powl_chain_hash_is_receipted() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run-42");
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::write(
            run_dir.join("plan.json"),
            r#"{"powl_chain_hash": "deadbeef"}"#,
        )
        .unwrap();
        let pattern = format!("{}/**/plan.json", dir.path().to_str().unwrap());
        let out = ingest_plan_runs(Some(&pattern));
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::Receipted));
        assert_eq!(out[0].id, "plan:run-42");
    }

    #[test]
    fn bench_raw_records_size_and_hash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("run.txt"), b"bench output").unwrap();
        let pattern = format!("{}/*.txt", dir.path().to_str().unwrap());
        let out = ingest_bench_raw(Some(&pattern));
        assert_eq!(out.len(), 1);
        assert!(out[0].standing.contains(&StandingStatus::Benchmarked));
        assert!(
            matches!(&out[0].evidence[0], EvidenceRef::Artifact { hash, .. } if hash.len() == 64)
        );
    }

    #[test]
    fn claim_table_parses_rows_without_elevating_standing() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("CLAIM_PROMOTION_TABLE.md");
        std::fs::write(
            &file,
            "| Claim | Status |\n|---|---|\n| foo | PRODUCTION_READY |\n",
        )
        .unwrap();
        let out = ingest_claim_tables(&[file.to_str().unwrap().to_string()]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Discovered]);
        assert!(!out[0].standing.contains(&StandingStatus::ProductionReady));
    }

    #[test]
    fn parse_pipe_table_rows_skips_separator_row() {
        let rows = parse_pipe_table_rows("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], vec!["1".to_string(), "2".to_string()]);
    }

    #[test]
    fn client_build_missing_path_is_unseen() {
        let out = ingest_client_builds(
            &[ClientTarget {
                id: "web".to_string(),
                path: "/no/such/client".to_string(),
                build_command: "true".to_string(),
            }],
            Duration::from_secs(5),
        );
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn client_build_success_yields_builds() {
        let dir = tempfile::tempdir().unwrap();
        let out = ingest_client_builds(
            &[ClientTarget {
                id: "web".to_string(),
                path: dir.path().to_str().unwrap().to_string(),
                build_command: "true".to_string(),
            }],
            Duration::from_secs(5),
        );
        assert!(out[0].standing.contains(&StandingStatus::Builds));
    }

    #[test]
    fn client_build_failure_omits_builds_but_records_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let out = ingest_client_builds(
            &[ClientTarget {
                id: "web".to_string(),
                path: dir.path().to_str().unwrap().to_string(),
                build_command: "false".to_string(),
            }],
            Duration::from_secs(5),
        );
        assert!(!out[0].standing.contains(&StandingStatus::Builds));
        assert_eq!(out[0].evidence.len(), 1);
    }

    fn write_crate(root: &Path, member: &str, crate_name: &str) {
        let dir = root.join(member);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\n"),
        )
        .unwrap();
    }

    #[test]
    fn workspace_crates_missing_root_manifest_is_unseen() {
        let dir = tempfile::tempdir().unwrap();
        let out = ingest_workspace_crates(dir.path().to_str().unwrap());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Unseen]);
    }

    #[test]
    fn workspace_crates_literal_members_are_discovered_rust_crates() {
        let dir = tempfile::tempdir().unwrap();
        // Root manifest combines `[package]` (the "." member) and
        // `[workspace]` in one file — the real shape a root-crate-plus-
        // members workspace takes (root.join(".") is the same file as
        // root's own Cargo.toml, so it cannot be written separately via
        // `write_crate`).
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"root-crate\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\".\", \"crates/foo\"]\n",
        )
        .unwrap();
        write_crate(dir.path(), "crates/foo", "foo-crate");

        let out = ingest_workspace_crates(dir.path().to_str().unwrap());
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|a| a.kind == ArtifactKind::RustCrate));
        assert!(out
            .iter()
            .all(|a| a.standing == vec![StandingStatus::Discovered]));
        assert!(out.iter().any(|a| a.id == "crate:root-crate"));
        assert!(out.iter().any(|a| a.id == "crate:foo-crate"));
        // Conservative: never fabricates BUILDS/TESTED/LINT_CLEAN.
        assert!(out
            .iter()
            .all(|a| !a.standing.contains(&StandingStatus::Builds)
                && !a.standing.contains(&StandingStatus::Tested)));
    }

    #[test]
    fn workspace_crates_glob_member_expands_subdirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();
        write_crate(dir.path(), "crates/a", "crate-a");
        write_crate(dir.path(), "crates/b", "crate-b");

        let out = ingest_workspace_crates(dir.path().to_str().unwrap());
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|a| a.id == "crate:crate-a"));
        assert!(out.iter().any(|a| a.id == "crate:crate-b"));
    }

    #[test]
    fn workspace_crates_empty_members_is_discovered_not_fabricated() {
        // The root Cargo.toml exists and parses (hence `DISCOVERED`, matching
        // every other ingestor's fallback contract) but declares zero
        // members, so no per-crate artifact is fabricated.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let out = ingest_workspace_crates(dir.path().to_str().unwrap());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].standing, vec![StandingStatus::Discovered]);
    }
}
