pub struct CargoMetadataAdapter;

impl CargoMetadataAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn workspace_name() -> String {
        std::fs::read_to_string("Cargo.toml")
            .ok()
            .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            .and_then(|v| {
                v.get("package")
                    .or_else(|| v.get("workspace").and_then(|w| w.get("package")))
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
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
        std::fs::read_to_string(&cargo_toml_path)
            .ok()
            .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            .and_then(|v| {
                v.get("workspace")
                    .and_then(|w| w.get("members"))
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
            })
            .unwrap_or_default()
    }
}

impl Default for CargoMetadataAdapter {
    fn default() -> Self {
        Self::new()
    }
}
