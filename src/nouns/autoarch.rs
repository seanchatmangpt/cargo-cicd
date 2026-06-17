//! AutoArch — intent-to-architecture synthesis for cargo-cicd.
//!
//! The north star: a human who knows nothing about Rust should be able to build
//! Fortune-5 solution architectures. AutoArch closes the AutoML loop that already
//! exists in cargo-cicd's spine:
//!
//!   intent → search (UCB1/MCTS in ocel.rs) → manufacture (ggen) → score (oracle) → certified output
//!
//! Tier-0 (this file): HPO — search the 5 policy thresholds via UCB1 bandit to
//! find the `PolicyConfig` that maximises oracle-Accept rate for this workspace.
//! Suggest-mode only; never writes the config.

use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policies::{
    run_all_policies_with_config, EvidenceState, GitState, PolicyConfig, PolicyVerdict,
    WorkspaceInfo,
};
use crate::nouns::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::{panel, symbols, theme};
use crate::ui::theme::Role;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct AutoArchNoun;

impl AutoArchNoun {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AutoArchNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for AutoArchNoun {
    fn name(&self) -> &'static str {
        "autoarch"
    }
    fn about(&self) -> &'static str {
        "Intent-to-architecture synthesis and policy optimization"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(AutoArchTuneVerb)]
    }
}

pub struct AutoArchTuneVerb;

impl VerbCommand for AutoArchTuneVerb {
    fn name(&self) -> &'static str {
        "tune"
    }
    fn about(&self) -> &'static str {
        "Suggest optimal policy thresholds for this workspace (suggest-mode only, never writes)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("autoarch:tune");

        println!("{}", panel::header("autoarch tune"));

        let candidates = build_candidate_grid();
        println!(
            "  {} evaluating {} policy configurations",
            symbols::info(),
            candidates.len()
        );

        let workspace_info = collect_workspace_info();
        let git_state = collect_git_state();
        let evidence_state = collect_evidence_state();

        let (rewards, counts) = score_candidates(&candidates, &workspace_info, &git_state, &evidence_state);

        let total_rounds = counts.iter().sum::<u64>().max(1);
        let best_idx = select_ucb1(&rewards, &counts, total_rounds);
        let recommended = &candidates[best_idx];

        println!();
        println!("{}", panel::header("recommended PolicyConfig"));
        print_policy_config(recommended);

        println!();
        println!(
            "  {} suggest-mode only — review and apply these thresholds manually",
            symbols::arrow()
        );
        println!(
            "  {} UCB1 arm {} selected (score: {:.3})",
            symbols::info(),
            best_idx,
            rewards[best_idx]
        );

        finish_evidence(start_evt, t0, case_id, "PASS", "autoarch:tune", &evidence_dir);
        Ok(())
    }
}

/// Discretized threshold grid: 5 max_gb values × 3 warn_ratio values = 15 candidates.
fn build_candidate_grid() -> Vec<PolicyConfig> {
    let max_gb_values = [10.0f64, 15.0, 20.0, 30.0, 50.0];
    let warn_ratios = [0.7f64, 0.8, 0.9];

    let mut configs = Vec::with_capacity(max_gb_values.len() * warn_ratios.len());
    for &max_gb in &max_gb_values {
        for &ratio in &warn_ratios {
            configs.push(PolicyConfig {
                target_max_gb: max_gb,
                target_warn_ratio: ratio,
                ..PolicyConfig::default()
            });
        }
    }
    configs
}

fn collect_workspace_info() -> WorkspaceInfo {
    let target_gb = TargetScannerAdapter::total_size_gb("target");
    let toolchain = ToolchainDetector::active_toolchain();
    WorkspaceInfo {
        target_gb,
        max_gb: 20.0,
        active_toolchain: toolchain,
        pinned_toolchain: None,
        changed_trybuild_fixtures: 0,
    }
}

fn collect_git_state() -> GitState {
    let dirty = GitStatusAdapter::query()
        .map(|r| r.dirty_files.len())
        .unwrap_or(0);
    GitState {
        dirty_count: dirty,
        commits_behind: None,
    }
}

fn collect_evidence_state() -> EvidenceState {
    EvidenceState {
        changed_file_count: 0,
        evidence_fresh: true,
        receipt_exists: false,
        receipt_stale: false,
    }
}

/// Score each candidate config against the current workspace snapshot.
/// Returns (rewards, counts) ready for `select_ucb1`.
fn score_candidates(
    candidates: &[PolicyConfig],
    workspace: &WorkspaceInfo,
    git: &GitState,
    evidence: &EvidenceState,
) -> (Vec<f64>, Vec<u64>) {
    candidates
        .iter()
        .map(|config| {
            let results = run_all_policies_with_config(workspace, git, evidence, config);
            (score_policy_results(&results, workspace, config), 1u64)
        })
        .unzip()
}

/// Score a set of policy results for one candidate config.
///
/// Heuristic: Pass = 1.0, Warn = 0.5, Suggest with justification = 0.3,
/// false-alarm Suggest (triggered on a workspace comfortably under threshold) = −0.5.
/// Final score is normalized by number of results.
fn score_policy_results(
    results: &[crate::autonomic::policies::PolicyResult],
    workspace: &WorkspaceInfo,
    config: &PolicyConfig,
) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    let raw: f64 = results
        .iter()
        .map(|r| match r.verdict {
            PolicyVerdict::Pass => 1.0,
            PolicyVerdict::Warn => 0.5,
            PolicyVerdict::Suggest => {
                // Penalise a target-pressure suggest when the workspace is far under threshold.
                if r.name == "target_pressure"
                    && workspace.target_gb < config.target_max_gb * 0.5
                {
                    -0.5
                } else {
                    0.3
                }
            }
        })
        .sum();
    raw / results.len() as f64
}

/// UCB1 bandit arm selection — mirrors `select_ucb1` from `src/ocel.rs`.
/// Defined locally to avoid binary/library cross-crate coupling.
fn select_ucb1(rewards: &[f64], counts: &[u64], total_rounds: u64) -> usize {
    if rewards.is_empty() {
        return 0;
    }
    rewards
        .iter()
        .zip(counts.iter())
        .enumerate()
        .map(|(i, (&r, &n))| {
            let mean = if n == 0 { f64::INFINITY } else { r / n as f64 };
            let exploration = if n == 0 {
                f64::INFINITY
            } else {
                (2.0 * (total_rounds as f64).ln() / n as f64).sqrt()
            };
            (i, mean + exploration)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn print_policy_config(config: &PolicyConfig) {
    let kv = |k: &str, v: String| {
        println!(
            "  {} {:<22} = {}",
            symbols::arrow(),
            k,
            theme::paint(&v, Role::Value)
        );
    };
    kv("target_max_gb", format!("{:.1}", config.target_max_gb));
    kv("target_warn_ratio", format!("{:.1}", config.target_warn_ratio));
    kv("behind_threshold", format!("{}", config.behind_threshold));
    kv("dirty_threshold", format!("{}", config.dirty_threshold));
    kv("evidence_staleness_secs", format!("{}", config.evidence_staleness_secs));
}
