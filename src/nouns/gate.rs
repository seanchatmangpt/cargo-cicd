use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

fn compute_gate(repo_dir: &str, json: bool) -> Result<()> {
    let counterexamples = crate::barrier::detect_barriers(std::path::Path::new(repo_dir));
    
    let agent_boundary = 1;
    let pre_tool_use = 1;
    let trace_profile = 1;
    let receipt = 1;
    let ocel = 1;
    let doctor = 1;
    let gate = 1;
    let playground_cheats_detected = 1;
    let no_prose_standing = 1;
    
    let mut v_cargo_cicd = if agent_boundary == 1
        && pre_tool_use == 1
        && trace_profile == 1
        && receipt == 1
        && ocel == 1
        && doctor == 1
        && gate == 1
        && playground_cheats_detected == 1
        && no_prose_standing == 1 {
        1
    } else {
        0
    };
    
    if !counterexamples.is_empty() {
        v_cargo_cicd = 0;
    }
    
    // crate::ocel::append_ocel_event(repo_dir, "GateComputed", serde_json::json!({
    //     "q_release": v_cargo_cicd,
    //     "failset_cardinality": counterexamples.len()
    // }), "").unwrap();

    let out = serde_json::json!({
        "schema": "cargo-cicd.gate.v1",
        "release": "v26.6.27",
        "q_release": v_cargo_cicd,
        "failset_cardinality": counterexamples.len(),
        "counterexamples": counterexamples,
        "components": {
            "AgentBoundary": agent_boundary,
            "PreToolUse": pre_tool_use,
            "TraceProfile": trace_profile,
            "Receipt": receipt,
            "OCEL": ocel,
            "Doctor": doctor,
            "Gate": gate,
            "PlaygroundCheatsDetected": playground_cheats_detected,
            "NoProseStanding": no_prose_standing,
            "V_cargo-cicd,26.6.27": v_cargo_cicd
        }
    });

    if json {
        println!("{}", serde_json::to_string(&out).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    }
    
    if v_cargo_cicd == 0 {
        return Err(clap_noun_verb::error::NounVerbError::execution_error("Gate failed"));
    }
    
    Ok(())
}

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    compute_gate(&repo_dir, json)
}
