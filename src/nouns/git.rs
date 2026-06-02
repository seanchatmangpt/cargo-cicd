use crate::adapters::GitStatusAdapter;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct GitNoun;
impl GitNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for GitNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for GitNoun {
    fn name(&self) -> &'static str {
        "git"
    }
    fn about(&self) -> &'static str {
        "Git phase management"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(GitStatusVerb), Box::new(GitCloseVerb)]
    }
}

pub struct GitStatusVerb;
impl VerbCommand for GitStatusVerb {
    fn name(&self) -> &'static str {
        "status"
    }
    fn about(&self) -> &'static str {
        "Show git repository state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let status = GitStatusAdapter::query()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("git status");
        println!("==========");
        println!("branch:       {}", status.branch);
        println!("staged:       {}", status.staged_files.len());
        println!("dirty:        {}", status.dirty_files.len());
        println!("untracked:    {}", status.untracked_files.len());
        println!("ahead:        {}", status.ahead);
        println!("behind:       {}", status.behind);
        println!();
        if !status.dirty_files.is_empty() {
            println!("dirty files:");
            for f in &status.dirty_files {
                println!("  M {}", f);
            }
        }
        if !status.untracked_files.is_empty() {
            println!("untracked:");
            for f in &status.untracked_files {
                println!("  ? {}", f);
            }
        }
        let next = if status.dirty_files.is_empty() && status.untracked_files.is_empty() {
            "tree is clean — ready to push"
        } else {
            "recommendation: run 'cargo cicd git close' to stage and commit"
        };
        println!("next: {}", next);
        Ok(())
    }
}

pub struct GitCloseVerb;
impl VerbCommand for GitCloseVerb {
    fn name(&self) -> &'static str {
        "close"
    }
    fn about(&self) -> &'static str {
        "Enforce phase closure: stage outputs, commit, verify clean"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let status = GitStatusAdapter::query()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("git phase closure");
        println!("=================");
        if status.dirty_files.is_empty() && status.untracked_files.is_empty() {
            println!("tree is clean — phase already closed");
            return Ok(());
        }
        println!("dirty files:   {}", status.dirty_files.len());
        println!("untracked:     {}", status.untracked_files.len());
        println!();
        println!("phase closure requires a clean tree.");
        println!("stage and commit your changes before closing the phase.");
        println!();
        println!("refusing to hide unrelated dirty files — no silent batch commit.");
        Err(clap_noun_verb::error::NounVerbError::execution_error(
            "phase closure refused: tree is dirty. Stage and commit manually, then re-run.",
        ))
    }
}
