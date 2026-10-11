//! src/nouns/pipeline.rs — cargo cicd pipeline verbs (inlined from the retired legacy layer; see CCICD-106)

pub struct PipelineRunVerb;

impl PipelineRunVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();

        // Fresh session: remove existing JSONL + OCEL so this pipeline run
        // starts a clean trace.
        let jsonl = evidence_dir.join("events.jsonl");
        let ocel = evidence_dir.join("events.ocel.json");
        let _ = std::fs::remove_file(&jsonl);
        let _ = std::fs::remove_file(&ocel);
        // Create a new session id after clearing state.
        let case_id = {
            let session_file = evidence_dir.join(".session");
            let _ = std::fs::remove_file(&session_file);
            crate::session::read_or_create_session_id(&evidence_dir)
        };

        let pipeline_start = std::time::Instant::now();
        println!("cargo-cicd manufacturing pipeline");
        println!("==================================");

        // Discover our own binary path so sub-commands share the same build.
        let binary = std::env::current_exe()?;

        // The declared activities executed as sub-processes in partial-order order.
        // status:audit (+ evidence:audit + receipt:write) is handled inline at the end.
        #[cfg_attr(not(feature = "affidavit"), allow(unused_mut))]
        let mut steps: Vec<(&str, Vec<&str>)> = vec![
            ("status:show", vec!["status", "show"]),
            ("target:show", vec!["target", "show"]),
            ("test:changed", vec!["test", "changed"]),
            ("trybuild:changed", vec!["trybuild", "changed"]),
            ("workspace:doctor", vec!["workspace", "doctor"]),
            ("publish:run", vec!["publish", "run"]),
        ];

        #[cfg(feature = "affidavit")]
        {
            steps.push(("affidavit:seal", vec!["affidavit", "seal"]));
            steps.push(("affidavit:verify", vec!["affidavit", "verify"]));
        }

        let mut pipeline_ok = true;

        // Truthful record of each step's actual outcome; this becomes the
        // OCEL 2.0 event log the wpm oracle adjudicates. Failures appear as
        // failures — never laundered into WARN.
        let mut step_events: Vec<crate::evidence::ProcessEvent> = Vec::new();

        for (activity, args) in &steps {
            let step_start = std::time::Instant::now();
            print!("  {} ... ", activity);
            // Flush stdout so progress is visible even if sub-command is slow.
            use std::io::Write as _;
            let _ = std::io::stdout().flush();

            let output = std::process::Command::new(&binary).args(args).output();

            let elapsed_ms = step_start.elapsed().as_millis() as u64;

            // Fail-closed: any non-success exit or spawn error is a FAIL step.
            let (step_verdict, detail) = match output {
                Ok(o) if o.status.success() => ("PASS", None),
                Ok(o) => ("FAIL", o.status.code().map(|c| format!("exit code {}", c))),
                Err(e) => ("FAIL", Some(e.to_string())),
            };
            println!("{} ({}ms)", step_verdict, elapsed_ms);
            if step_verdict == "FAIL" {
                pipeline_ok = false;
                if let Some(d) = &detail {
                    println!("        {}", d);
                }
            }

            let mut ev =
                crate::evidence::ProcessEvent::completed(activity, step_start, step_verdict);
            ev.case_id = Some(case_id.clone());
            ev.trace_class = "pipeline_run".to_string();
            step_events.push(ev);

            // Journal the step outcome in the hash-chained OCEL log.
            if let Err(e) = crate::ocel::append_ocel_event(
                ".",
                "PipelineStepRan",
                serde_json::json!({
                    "activity": activity,
                    "verdict": step_verdict,
                    "duration_ms": elapsed_ms,
                }),
                "",
            ) {
                eprintln!("warning: OCEL journal write failed for {}: {}", activity, e);
            }
        }

        // ── status:audit (inline): oracle adjudicates the TRUTHFUL OCEL log ────
        //
        // No XES anywhere: the audit log is the OCEL 2.0 JSON built from the
        // actual step outcomes recorded above, and the oracle is
        // `wpm receipt verify-ocel2` (the OCEL 2.0 receipt verifier).
        print!("  status:audit ... ");
        use std::io::Write as _;
        let _ = std::io::stdout().flush();

        let audit_start = std::time::Instant::now();

        let wpm = crate::integrations::Wasm4pmShell::detect();
        let audit_ocel_path = evidence_dir.join("audit-events.ocel.json");
        let audit_result = if let Some(wpm_shell) = &wpm {
            match crate::evidence::build_ocel_log_filtered(&step_events) {
                serde_json::Value::Object(doc) => {
                    let write_res = std::fs::write(
                        &audit_ocel_path,
                        serde_json::to_string_pretty(&serde_json::Value::Object(doc))
                            .unwrap_or_default(),
                    );
                    match write_res {
                        Ok(()) => Some(wpm_shell
                            .receipt_verify_ocel2(audit_ocel_path.to_str().unwrap_or(""))
                            .unwrap_or_else(|e| crate::integrations::WpmResult {
                                command: "wpm receipt verify-ocel2".to_string(),
                                success: false,
                                stdout: String::new(),
                                stderr: e.to_string(),
                                verdict: crate::integrations::WpmVerdict::Fail,
                            })),
                        Err(e) => {
                            eprintln!("warning: audit OCEL write failed: {}", e);
                            None
                        }
                    }
                }
                _ => {
                    eprintln!("warning: OCEL log builder returned non-object");
                    None
                }
            }
        } else {
            None
        };

        let audit_elapsed_ms = audit_start.elapsed().as_millis() as u64;
        // Map wpm's TRUTHFUL/VARIANCE/DECEPTIVE fitness bands (surfaced via
        // WpmVerdict::Pass/Warn/Fail by `infer_audit_verdict`) to the
        // pipeline's ACCEPT/WARN/REFUSE self-check vocabulary. E1: this is
        // purely a label remap of the oracle's own verdict — cargo-cicd never
        // substitutes its own judgment for wpm's.
        let oracle_verdict = audit_result
            .as_ref()
            .map(|r| match r.verdict {
                crate::integrations::WpmVerdict::Pass => "ACCEPT",
                crate::integrations::WpmVerdict::Warn
                | crate::integrations::WpmVerdict::Partial => "WARN",
                crate::integrations::WpmVerdict::Fail
                | crate::integrations::WpmVerdict::NotAvailable => "REFUSE",
            })
            .unwrap_or("SKIP");
        println!("{} ({}ms)", oracle_verdict, audit_elapsed_ms);

        // Emit status:audit + evidence:audit + receipt:write events.
        let (mut sa_start_evt, sa_t0) = crate::evidence::ProcessEvent::started("status:audit");
        sa_start_evt.case_id = Some(case_id.clone());
        sa_start_evt.trace_class = "pipeline_run".to_string();
        let mut sa_complete =
            crate::evidence::ProcessEvent::completed("status:audit", sa_t0, oracle_verdict);
        sa_complete.case_id = Some(case_id.clone());
        sa_complete.trace_class = "pipeline_run".to_string();

        let wpm_path = wpm
            .as_ref()
            .map(|w| w.binary_path().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut ea_evt = crate::evidence::ProcessEvent::new_adjudicated(
            "evidence:audit",
            oracle_verdict,
            &wpm_path,
        );
        ea_evt.case_id = Some(case_id.clone());
        ea_evt.trace_class = "pipeline_run".to_string();

        let mut events_to_append = vec![sa_start_evt, sa_complete, ea_evt];

        if oracle_verdict == "ACCEPT" {
            let mut rw_evt =
                crate::evidence::ProcessEvent::for_pipeline("receipt:write", "COMPLETE");
            rw_evt.case_id = Some(case_id.clone());
            events_to_append.push(rw_evt);
        }

        if let Err(e) = crate::evidence::append_events(&events_to_append, &evidence_dir) {
            eprintln!("warning: pipeline evidence emission failed: {}", e);
        }

        // ── Final verdict ───────────────────────────────────────────────────────
        println!();
        if let Some(r) = &audit_result {
            println!("oracle stdout: {}", r.stdout.trim());
            if !r.stderr.trim().is_empty() {
                println!("oracle stderr: {}", r.stderr.trim());
            }
        }

        let total_ms = pipeline_start.elapsed().as_millis();
        println!("\nPipeline completed in {}ms", total_ms);

        if !pipeline_ok || oracle_verdict == "REFUSE" {
            anyhow::bail!("pipeline failed: oracle_verdict={}", oracle_verdict);
        }

        Ok(())
    }
}


// ── pipeline status verb ──────────────────────────────────────────────────────

pub struct PipelineStatusVerb;

impl PipelineStatusVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = crate::evidence::ProcessEvent::started("pipeline:status");
        start_evt.case_id = Some(case_id.clone());

        println!("pipeline status");
        println!("===============");

        // Show cicd.toml state
        let toml_path = std::path::Path::new("cicd.toml");
        if toml_path.exists() {
            println!("cicd.toml:      present");
            if let Ok(content) = std::fs::read_to_string(toml_path) {
                for line in content.lines() {
                    if line.starts_with("target_size_gb")
                        || line.starts_with("changed_files")
                        || line.starts_with("dirty")
                        || line.starts_with("changed_tests")
                    {
                        println!("  {}", line.trim());
                    }
                }
            }
        } else {
            println!("cicd.toml:      MISSING — run 'cargo cicd pipeline run' first");
        }

        // Show evidence state
        let events_jsonl = evidence_dir.join("events.jsonl");
        if events_jsonl.exists() {
            let line_count = std::fs::read_to_string(&events_jsonl)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            println!("evidence:       {} events in events.jsonl", line_count);
        } else {
            println!("evidence:       no events.jsonl");
        }

        let ocel_path = evidence_dir.join("events.ocel.json");
        println!(
            "events.ocel.json: {}",
            if ocel_path.exists() {
                "present"
            } else {
                "missing"
            }
        );

        // Count receipts
        let receipt_dir = evidence_dir.join("receipts");
        let receipt_count = std::fs::read_dir(&receipt_dir)
            .map(|r| r.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        println!("receipts:       {}", receipt_count);

        println!();
        println!("next: run 'cargo cicd pipeline run' to execute the full pipeline");

        let mut complete_evt =
            crate::evidence::ProcessEvent::completed("pipeline:status", t0, "PASS");
        complete_evt.case_id = Some(case_id);
        let _ = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir);

        Ok(())
    }
}

// ── pipeline validate verb ────────────────────────────────────────────────────

pub struct PipelineValidateVerb;

impl PipelineValidateVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
        println!("pipeline validate");
        println!("=================");

        let mut all_pass = true;

        let checks: &[(&str, bool)] = &[
            ("Cargo.toml", std::path::Path::new("Cargo.toml").exists()),
            ("git initialized", std::path::Path::new(".git").exists()),
            ("cicd.toml", std::path::Path::new("cicd.toml").exists()),
        ];

        for (name, ok) in checks {
            let tag = if *ok {
                "PASS"
            } else {
                all_pass = false;
                "WARN"
            };
            println!("[{}] {}", tag, name);
        }

        // wpm is optional
        let wpm_ok = std::process::Command::new("wpm")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        println!(
            "[{}] wpm oracle{}",
            if wpm_ok { "PASS" } else { "WARN" },
            if wpm_ok {
                ""
            } else {
                " (optional — set WPM_PATH to enable)"
            }
        );

        println!();
        if all_pass {
            println!("all preconditions met — ready to run 'cargo cicd pipeline run'");
        } else {
            println!("some preconditions missing — review WARNs above");
        }
        Ok(())
    }
}

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("run")]
pub fn cmd_run() -> Result<()> {
    PipelineRunVerb
        .execute()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

#[verb("status")]
pub fn cmd_status() -> Result<()> {
    PipelineStatusVerb.execute()
}

#[verb("validate")]
pub fn cmd_validate() -> Result<()> {
    PipelineValidateVerb.execute()
}
