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
        Self::workspace_members_from(".")
    }

    pub fn workspace_members_from(workspace_root: &str) -> Vec<String> {
        let cargo_toml_path = format!("{}/Cargo.toml", workspace_root);
        let content = match std::fs::read_to_string(&cargo_toml_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Parse [workspace] members array using a simple line-by-line scan.
        let mut in_workspace = false;
        let mut in_members = false;
        let mut members = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[workspace]" {
                in_workspace = true;
                continue;
            }
            if in_workspace && trimmed.starts_with('[') && trimmed != "[workspace]" {
                in_workspace = false;
                in_members = false;
            }
            if in_workspace && trimmed.starts_with("members") {
                in_members = true;
            }
            if in_members {
                // Extract quoted strings from the line.
                let mut chars = trimmed.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '"' {
                        let member: String = chars.by_ref().take_while(|&c| c != '"').collect();
                        if !member.is_empty() {
                            members.push(member);
                        }
                    }
                }
                if trimmed.contains(']') {
                    in_members = false;
                }
            }
        }

        members
    }
}

impl Default for CargoMetadataAdapter {
    fn default() -> Self {
        Self::new()
    }
}
