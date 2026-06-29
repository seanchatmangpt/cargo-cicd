use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use crate::ocel::{ReplayStatus, ReplaySummary};

#[derive(serde::Serialize)]
pub struct OcelReplayOutput {
    pub schema: String,
    pub repo: String,
    pub status: String,
    pub events_verified: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken_at_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub q: i32,
}

impl OcelReplayOutput {
    fn from_summary(repo: String, summary: ReplaySummary) -> Self {
        match summary.status {
            ReplayStatus::Success => OcelReplayOutput {
                schema: "cargo-cicd.ocel.replay.v1".to_string(),
                repo,
                status: "success".to_string(),
                events_verified: summary.events_verified,
                broken_at_index: None,
                error: None,
                q: 1,
            },
            ReplayStatus::Empty => OcelReplayOutput {
                schema: "cargo-cicd.ocel.replay.v1".to_string(),
                repo,
                status: "empty".to_string(),
                events_verified: 0,
                broken_at_index: None,
                error: None,
                q: 1,
            },
            ReplayStatus::ChainBroken { index } => OcelReplayOutput {
                schema: "cargo-cicd.ocel.replay.v1".to_string(),
                repo,
                status: "chain_broken".to_string(),
                events_verified: summary.events_verified,
                broken_at_index: Some(index),
                error: Some(format!("Hash chain broken at event index {}", index)),
                q: 0,
            },
            ReplayStatus::HashInvalid { index } => OcelReplayOutput {
                schema: "cargo-cicd.ocel.replay.v1".to_string(),
                repo,
                status: "hash_invalid".to_string(),
                events_verified: summary.events_verified,
                broken_at_index: Some(index),
                error: Some(format!("Invalid event hash at event index {}", index)),
                q: 0,
            },
        }
    }
}

pub fn evaluate_ocel_replay(repo_dir: &str) -> OcelReplayOutput {
    match crate::ocel::replay_events_in_repo(repo_dir) {
        Ok(summary) => OcelReplayOutput::from_summary(repo_dir.to_string(), summary),
        Err(e) => OcelReplayOutput {
            schema: "cargo-cicd.ocel.replay.v1".to_string(),
            repo: repo_dir.to_string(),
            status: "failure".to_string(),
            events_verified: 0,
            broken_at_index: None,
            error: Some(e),
            q: 0,
        },
    }
}

#[verb("replay")]
pub fn cmd_replay(repo: Option<String>, json: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let output = evaluate_ocel_replay(&repo_dir);

    if json {
        println!("{}", serde_json::to_string(&output).unwrap());
    } else if output.status == "success" {
        println!("OCEL replay successful: {} events verified in {}", output.events_verified, repo_dir);
    } else if output.status == "empty" {
        println!("OCEL replay: no events log found in {}", repo_dir);
    } else {
        eprintln!("OCEL replay failed: {}", output.error.as_ref().unwrap());
    }

    if output.q == 0 {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(output.error.unwrap()));
    }

    Ok(())
}
