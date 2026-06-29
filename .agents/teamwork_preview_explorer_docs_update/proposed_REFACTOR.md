# Refactoring Guide: Migrating cargo-cicd to star-toml

This guide maps out the step-by-step refactoring needed to migrate the configuration loading and validation of `cargo-cicd` from standard TOML deserialization to the `star-toml` config admission framework.

---

## 1. Step-by-Step Refactoring Steps

### Step 1: Update Configuration Schema & Implement Validation
**Target File**: `cargo-cicd-core/src/config.rs`  
**Goal**: Implement `star_toml::Validate` and the `star_toml::loader::ConfigLifecycle` traits on the workspace configuration structures.

1. Add `star-toml` and `star-toml-derive` to `cargo-cicd-core/Cargo.toml`.
2. Replace raw serde attributes with `star_toml` custom validators:

```rust
use serde::{Deserialize, Serialize};
use star_toml::{Validate, Validator, loader::ConfigLifecycle};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CicdConfig {
    pub target: TargetConfig,
    pub test: TestConfig,
    pub autonomic: AutonomicConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetConfig {
    pub max_size_gb: f64,
    pub prune_after_days: u32,
    pub build_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    pub base: String,
    pub check_trybuild: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutonomicConfig {
    pub enabled: bool,
    pub mode: String,
}

// Implement validation rules replacing ad-hoc logic
impl Validate for CicdConfig {
    fn validate(&self, v: &mut Validator) {
        // Target assertions
        v.check_range("target.max_size_gb", self.target.max_size_gb, 0.1..=100.0);
        v.check_range("target.prune_after_days", self.target.prune_after_days, 1..=365);
        
        // Path Sandboxing check
        v.check_path_policy("target.build_dir", &self.target.build_dir, star_toml::PathPolicy::Sandbox);

        // Test configuration base check
        v.check_non_empty("test.base", &self.test.base);

        // Autonomic mode constraints
        v.check_enum("autonomic.mode", &self.autonomic.mode, &["suggest", "enforce"]);
    }
}

impl ConfigLifecycle for CicdConfig {}
```

---

### Step 2: Inject AdmittedConfig into EngineState
**Target File**: `cargo-cicd-core/src/engine.rs`  
**Goal**: Wrap the config in `star_toml::AdmittedConfig<T>` within the `EngineState` aggregate.

1. Locate the definition of `EngineState`.
2. Replace the optional config field with an admitted configuration envelope:

```rust
use star_toml::AdmittedConfig;
use crate::config::CicdConfig;

pub struct EngineState {
    // Before: pub config: Option<CicdConfig>,
    // After: Enforce that all state has admitted operational law
    pub config: AdmittedConfig<CicdConfig>,
    pub workspace: WorkspaceState,
    pub toolchain: ToolchainState,
    // ... other dimensions
}
```

---

### Step 3: Implement Loader Pipeline & Strict Admission
**Target File**: `cargo-cicd-cli/src/main.rs`  
**Goal**: Initialize configuration loading via `TrustedLoader` with strict validation rules.

Modify `main` or the config-init logic to load using `star-toml`:

```rust
use star_toml::loader::TrustedLoader;
use cargo_cicd_core::config::CicdConfig;

fn load_configuration() -> Result<star_toml::AdmittedConfig<CicdConfig>, anyhow::Error> {
    let config_path = std::env::var("CICD_TOML_PATH").unwrap_or_else(|_| "cicd.toml".to_string());

    // Build the admission loader
    let loader = TrustedLoader::new()
        .layer_file(&config_path)
        .env_prefix("CICD_")
        .strict_unknown_fields(true); // Fail-fast on unrecognized fields

    match loader.load_admitted::<CicdConfig>() {
        Ok(admitted) => Ok(admitted),
        Err(err) => {
            // Emits structured diagnostics on stderr before failing
            eprintln!("Configuration Admission Refused ($q_config = 0$)!");
            eprintln!("{}", err.render_diagnostics());
            std::process::exit(2); // Exit Code 2: Workspace Invalid
        }
    }
}
```

---

### Step 4: IDE Diagnostics via star-toml-lsp
**Goal**: Wire the configuration JSON schema schema export to enable live inline diagnostics.

1. Add a schema generator bin command or target:
   `cargo run --bin cargo_cicd_schema_export > docs/schema.json`
2. Configure the local workspace LSP settings (`.vscode/settings.json` or `.helix/config.toml`):

```json
{
  "json.schemas": [
    {
      "fileMatch": ["cicd.toml"],
      "url": "./docs/schema.json"
    }
  ]
}
```

When developers edit `cicd.toml`, the JSON-schema bridge checks for unsupported keywords, missing descriptions, or invalid ranges, warning the developer inside their editor.

---

### Step 5: Receipt Verification Loop
**Target File**: `cargo-cicd-core/src/publish/run.rs`  
**Goal**: Check the final execution receipt against the `AdmittedConfig` witness digest.

Before committing the publish operation to Crates.io:

```rust
pub fn verify_publish_gate(
    config: &star_toml::AdmittedConfig<CicdConfig>,
    receipt_path: &str
) -> Result<(), anyhow::Error> {
    // 1. Read the latest execution receipt
    let receipt = read_receipt(receipt_path)?;

    // 2. Fetch the cryptographic witness digest from the admitted configuration
    let expected_config_digest = config.witness().hash();

    // 3. Verify they are bound
    if receipt.admitted_config_digest != expected_config_digest {
        return Err(anyhow::anyhow!(
            "Publish Refused: Admitted configuration hash ({}) does not match the execution receipt digest ({})!",
            expected_config_digest,
            receipt.admitted_config_digest
        ));
    }

    Ok(())
}
```

---

### Step 6: Conformance Testing
**Target File**: `tests/invariants.rs`  
**Goal**: Utilize `star-toml-config-test` helper assertions to verify configuration laws.

```rust
#[test]
fn test_config_invariants() {
    use star_toml_config_test::{assert_schema_admitted, assert_schema_refused};

    // Assert that a standard config is admitted
    let valid_toml = r#"
        [target]
        max_size_gb = 15.0
        prune_after_days = 7
        build_dir = "./target"

        [test]
        base = "origin/main"
        check_trybuild = true

        [autonomic]
        enabled = false
        mode = "suggest"
    "#;
    assert_schema_admitted::<CicdConfig>(valid_toml);

    // Assert that directory traversal triggers refusal
    let traversal_toml = r#"
        [target]
        max_size_gb = 15.0
        prune_after_days = 7
        build_dir = "../../../etc/passwd"
        # ... rest of valid fields
    "#;
    assert_schema_refused::<CicdConfig>(traversal_toml);
}
```
