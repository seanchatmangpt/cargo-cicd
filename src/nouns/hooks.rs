//! Installs git hooks that integrate cargo-cicd with an external CI provider.
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use std::io::Read;

#[verb("install")]
pub fn cmd_install(repo: Option<String>, provider: Option<String>, json: bool) -> Result<()> {
    let provider_str = provider.as_deref().unwrap_or("antigravity");
    if provider_str != "antigravity" {
        let err = serde_json::json!({
            "schema": "cargo-cicd.hooks.install.error.v1",
            "error": "unsupported_provider",
            "provider": provider_str,
            "supported": ["antigravity"]
        });
        println!("{}", err);
        std::process::exit(1);
    }

    let binary_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "cargo-cicd".to_string());

    let repo_dir = repo.unwrap_or_else(|| ".".to_string());
    let agents_dir = format!("{}/.agents", repo_dir);
    let hook_path = format!("{}/.agents/hooks.json", repo_dir);

    let hooks_json = format!(
        r#"{{
  "cargo-cicd.execute": {{
    "enabled": true,
    "PreToolUse": [
      {{
        "matcher": "run_command",
        "hooks": [
          {{
            "type": "command",
            "command": "{binary} hooks pre-tool-use --repo $WORKSPACE_DIR --json",
            "timeout": 30
          }}
        ]
      }}
    ]
  }}
}}"#,
        binary = binary_path
    );

    std::fs::create_dir_all(&agents_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {}", agents_dir, e));
    std::fs::write(&hook_path, &hooks_json)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", hook_path, e));

    if json {
        let receipt = serde_json::json!({
            "schema": "cargo-cicd.hooks.install.v1",
            "q_install": 1,
            "hook_path": hook_path,
            "provider": "antigravity",
            "binary_path": binary_path
        });
        println!("{}", receipt);
    } else {
        println!("Installed Antigravity PreToolUse hook to {}", hook_path);
    }
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
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    let _ = json;
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        println!(r#"{{"allow_tool": true}}"#);
        std::process::exit(0);
    }

    check_tool_use(&repo_dir, &input);
    Ok(())
}

fn check_tool_use(repo_dir: &str, input: &str) {
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

    let _ = crate::ocel::append_ocel_event(
        repo_dir,
        "PreToolUseAttempt",
        serde_json::json!({"command": cmd_str}),
        "",
    );

    if is_allowed(cmd_str) {
        let _ = crate::ocel::append_ocel_event(
            repo_dir,
            "PolicyDecision",
            serde_json::json!({"allow_tool": true}),
            "",
        );
        println!(r#"{{"allow_tool": true}}"#);
        std::process::exit(0);
    } else if let Some(reason) = get_block_reason(cmd_str) {
        let _ = crate::ocel::append_ocel_event(
            repo_dir,
            "PolicyDecision",
            serde_json::json!({"allow_tool": false, "deny_reason": reason}),
            "",
        );
        println!(r#"{{"allow_tool": false, "deny_reason": "{}"}}"#, reason);
        std::process::exit(1);
    } else {
        let _ = crate::ocel::append_ocel_event(
            repo_dir,
            "PolicyDecision",
            serde_json::json!({"allow_tool": true}),
            "",
        );
        println!(r#"{{"allow_tool": true}}"#);
        std::process::exit(0);
    }
}

fn is_allowed(cmd_str: &str) -> bool {
    let allowed = [
        "cargo cicd ",
        "just ",
        "pwd",
        "ls",
        "find ",
        "grep ",
        "git status",
        "git diff --stat",
    ];
    allowed
        .iter()
        .any(|a| cmd_str.starts_with(a) || cmd_str == &a[..a.len().saturating_sub(1)])
}

fn get_block_reason(cmd_str: &str) -> Option<&'static str> {
    let blocked = ["cargo ", "bash ", "sh ", "python ", "make "];
    if blocked.iter().any(|b| cmd_str.starts_with(b)) {
        Some(
            "Use cargo cicd trace run --repo . --profile test or cargo cicd doctor repo --repo . --json",
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_install_writes_correct_binary_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().to_string_lossy().to_string();

        cmd_install(Some(repo.clone()), Some("antigravity".to_string()), false)
            .expect("cmd_install should succeed");

        let hook_path = format!("{}/.agents/hooks.json", repo);
        let content = std::fs::read_to_string(&hook_path).expect("hooks.json should exist");

        assert!(
            !content.contains("cargo run"),
            "hook must not use 'cargo run'"
        );

        let binary_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        assert!(
            content.contains(&binary_path),
            "hook command must contain the current binary path: {}",
            binary_path
        );
        assert!(
            content.contains("cargo-cicd.execute"),
            "hook must use cargo-cicd.execute key"
        );
        assert!(
            content.contains("pre-tool-use"),
            "hook must reference pre-tool-use verb"
        );
    }
}
