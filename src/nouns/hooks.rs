use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use std::io::Read;

#[verb("install")]
pub fn cmd_install(repo: Option<String>, provider: Option<String>, json: bool) -> Result<()> {
    let _ = repo;
    let _ = provider;
    let _ = json;
    // Generate .agents/hooks.json
    let hooks_json = r#"{
  "cargo-cicd-guard": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          {
            "type": "command",
            "command": "cargo run --bin cargo-cicd -- cicd hooks pre-tool-use --repo $WORKSPACE_DIR --json",
            "timeout": 30
          }
        ]
      }
    ]
  }
}"#;
    std::fs::create_dir_all(".agents").unwrap();
    std::fs::write(".agents/hooks.json", hooks_json).unwrap();
    println!("Installed Antigravity PreToolUse hook to .agents/hooks.json");
    Ok(())
}

#[verb("uninstall")]
pub fn cmd_uninstall(repo: Option<String>, provider: Option<String>, json: bool) -> Result<()> {
    let _ = repo;
    let _ = provider;
    let _ = json;
    if std::path::Path::new(".agents/hooks.json").exists() {
        std::fs::remove_file(".agents/hooks.json").unwrap();
        println!("Uninstalled Antigravity PreToolUse hook");
    }
    Ok(())
}

#[verb("pre-tool-use")]
pub fn cmd_pre_tool_use(repo: Option<String>, json: bool) -> Result<()> {
    let _ = repo;
    let _ = json;
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        println!(r#"{{"allow_tool": true}}"#);
        return Ok(());
    }
    
    check_tool_use(&input);
    Ok(())
}

fn check_tool_use(input: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => {
            println!(r#"{{"allow_tool": true}}"#);
            std::process::exit(0);
        }
    };

    let cmd_str = match parsed.pointer("/toolCall/args/CommandLine") {
        Some(serde_json::Value::String(s)) => s,
        _ => {
            println!(r#"{{"allow_tool": true}}"#);
            std::process::exit(0);
        }
    };

    if is_allowed(cmd_str) {
        println!(r#"{{"allow_tool": true}}"#);
    } else if let Some(reason) = get_block_reason(cmd_str) {
        println!(r#"{{"allow_tool": false, "deny_reason": "{}"}}"#, reason);
    } else {
        println!(r#"{{"allow_tool": true}}"#);
    }
    
    std::process::exit(0);
}


fn is_allowed(cmd_str: &str) -> bool {
    let allowed = ["cargo cicd ", "just ", "pwd", "ls", "find ", "grep ", "git status", "git diff --stat"];
    allowed.iter().any(|a| cmd_str.starts_with(a) || cmd_str == &a[..a.len().saturating_sub(1)])
}

fn get_block_reason(cmd_str: &str) -> Option<&'static str> {
    let blocked = ["cargo ", "bash ", "sh ", "python ", "make "];
    if blocked.iter().any(|b| cmd_str.starts_with(b)) {
        Some("Use cargo cicd trace run --repo . --profile test or cargo cicd doctor repo --repo . --json")
    } else {
        None
    }
}

