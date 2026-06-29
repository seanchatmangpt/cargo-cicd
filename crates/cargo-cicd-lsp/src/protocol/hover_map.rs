//! Per-field hover cards for cicd.toml keys.

use std::collections::HashMap;

/// A hover card describing a single cicd.toml field.
pub struct HoverCard {
    pub code: &'static str,
    pub section: &'static str,
    pub field: &'static str,
    pub controls: &'static str,
    pub repair_hint: &'static str,
}

/// Return the static map from TOML field name to HoverCard.
pub fn hover_card_map() -> &'static HashMap<&'static str, HoverCard> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, HoverCard>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();

        m.insert(
            "require_clean_tree",
            HoverCard {
                code: "CICD-GIT-001",
                section: "[git]",
                field: "require_clean_tree",
                controls:
                    "Blocks phase closure when uncommitted changes exist in the working tree.",
                repair_hint: "Commit or stash all changes before attempting phase closure.",
            },
        );

        m.insert(
            "max_size_gb",
            HoverCard {
                code: "CICD-TARGET-001",
                section: "[target]",
                field: "max_size_gb",
                controls: "Maximum allowed size of the target directory in gigabytes.",
                repair_hint:
                    "Run `cargo cicd target show` then `cargo cicd target prune` to reclaim space.",
            },
        );

        m.insert(
            "base",
            HoverCard {
                code: "CICD-TESTS-001",
                section: "[test.changed]",
                field: "base",
                controls: "Baseline git ref for changed-file detection.",
                repair_hint:
                    "Set to a stable branch ref (e.g. `main`) for accurate change detection.",
            },
        );

        m.insert(
            "test_command",
            HoverCard {
                code: "CICD-TEST-001",
                section: "[test]",
                field: "test_command",
                controls: "Shell command used to execute the test suite.",
                repair_hint: "Ensure the command exits non-zero on test failure.",
            },
        );

        m.insert(
            "require_oracle",
            HoverCard {
                code: "CICD-WPM-001",
                section: "[wpm]",
                field: "require_oracle",
                controls: "Requires the wpm runtime court to adjudicate evidence before close.",
                repair_hint:
                    "Install wpm or set WPM_BIN env var, then run `cargo cicd evidence doctor`.",
            },
        );

        // EVIDENCE category
        m.insert(
            "evidence_dir",
            HoverCard {
                code: "CICD-EVIDENCE-001",
                section: "[evidence]",
                field: "evidence_dir",
                controls: "Path to the directory where process evidence is written.",
                repair_hint:
                    "Run any `cargo cicd` command to emit process evidence to this directory.",
            },
        );

        // PUBLISH category
        m.insert(
            "dry_run",
            HoverCard {
                code: "CICD-PUBLISH-002",
                section: "[publish]",
                field: "dry_run",
                controls: "When true, publish is simulated without uploading to a registry.",
                repair_hint:
                    "Run `cargo cicd evidence doctor` then `cargo cicd publish` to complete.",
            },
        );

        // GGEN category
        m.insert(
            "sync_on_change",
            HoverCard {
                code: "CICD-GGEN-002",
                section: "[ggen]",
                field: "sync_on_change",
                controls: "Automatically re-run ggen sync when source law changes are detected.",
                repair_hint: "Run `ggen sync` to regenerate rendered surfaces.",
            },
        );

        // WORKSPACE category
        m.insert(
            "validate_members",
            HoverCard {
                code: "CICD-WORKSPACE-001",
                section: "[workspace]",
                field: "validate_members",
                controls: "Validates that all workspace member crates are structurally consistent.",
                repair_hint: "Run `cargo cicd workspace validate` to diagnose structural issues.",
            },
        );

        m
    })
}
