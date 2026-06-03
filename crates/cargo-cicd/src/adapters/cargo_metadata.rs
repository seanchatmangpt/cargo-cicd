use std::process::Command;

pub struct CargoMetadataAdapter;

impl CargoMetadataAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn workspace_name() -> String {
        std::fs::read_to_string("Cargo.toml")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.trim().starts_with("name = "))
                    .map(|l| l.split('"').nth(1).unwrap_or("workspace").to_string())
            })
            .unwrap_or_else(|| "workspace".into())
    }

    pub fn target_dir() -> String {
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into())
    }

    pub fn workspace_members() -> Vec<String> {
        let output = Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version=1"])
            .output()
            .ok();
        if let Some(out) = output {
            if let Ok(s) = String::from_utf8(out.stdout) {
                // Parse workspace_members array from JSON output.
                // Simplified: real impl would use serde_json.
                if s.find("\"workspace_members\"").is_some() {
                    return vec![];
                }
            }
        }
        vec![]
    }
}

impl Default for CargoMetadataAdapter {
    fn default() -> Self {
        Self::new()
    }
}
