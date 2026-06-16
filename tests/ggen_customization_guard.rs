//! Guard: ggen protected blocks must remain canonical across rerenders.
//! Custom blocks must survive rerenders.
//! Public docs must contain no forbidden terms.
//! cargo-cicd v26.6.2

use std::fs;
use std::path::Path;

const FORBIDDEN: &[&str] = &[
    "ALIVE",
    "Inspection Gate",
    "Nehemiah",
    "Field8",
    "Instinct8",
    "Cargo Court",
    "Truex",
    "CONSTRUCT8",
];

fn scan_forbidden(content: &str, path: &str) -> Vec<String> {
    FORBIDDEN
        .iter()
        .filter(|&&term| content.contains(term))
        .map(|&term| format!("{}:{}", path, term))
        .collect()
}

#[test]
fn no_forbidden_terms_in_public_docs() {
    let mut violations: Vec<String> = Vec::new();

    // Scan README.md
    if let Ok(content) = fs::read_to_string("README.md") {
        violations.extend(scan_forbidden(&content, "README.md"));
    }

    // Scan public Diataxis doc directories only (internal analysis dirs excluded)
    let public_doc_dirs = [
        "docs/tutorials",
        "docs/how-to",
        "docs/reference",
        "docs/explanation",
    ];
    for dir in &public_doc_dirs {
        let docs_path = Path::new(dir);
        if docs_path.exists() {
            for entry in walkdir::WalkDir::new(docs_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let path_str = entry.path().to_string_lossy();
                    violations.extend(scan_forbidden(&content, &path_str));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Forbidden terms in public docs: {:?}",
        violations
    );
}

#[test]
fn ggen_protected_blocks_balanced() {
    // Every BEGIN ggen: must have a matching END ggen:
    let check_files = ["README.md"];
    for file in &check_files {
        if let Ok(content) = fs::read_to_string(file) {
            let begins: Vec<&str> = content
                .lines()
                .filter(|l| l.contains("BEGIN ggen:"))
                .collect();
            let ends: Vec<&str> = content
                .lines()
                .filter(|l| l.contains("END ggen:"))
                .collect();
            assert_eq!(
                begins.len(),
                ends.len(),
                "{}: unbalanced ggen blocks — {} BEGIN vs {} END",
                file,
                begins.len(),
                ends.len()
            );
        }
    }
}

#[test]
fn custom_blocks_balanced() {
    let check_files = ["README.md"];
    for file in &check_files {
        if let Ok(content) = fs::read_to_string(file) {
            let begins = content.matches("BEGIN custom:").count();
            let ends = content.matches("END custom:").count();
            assert_eq!(
                begins, ends,
                "{}: unbalanced custom blocks — {} BEGIN vs {} END",
                file, begins, ends
            );
        }
    }
}

#[test]
fn readme_has_command_table() {
    let content = fs::read_to_string("README.md").expect("README.md must exist");
    // Every public command must appear in the README
    let commands = [
        "cargo cicd status",
        "cargo cicd target",
        "cargo cicd test",
        "cargo cicd publish",
        "cargo cicd workspace",
    ];
    for cmd in &commands {
        assert!(content.contains(cmd), "README.md missing command: {}", cmd);
    }
}

// ---------------------------------------------------------------------------
// Named tests required by guard spec
// ---------------------------------------------------------------------------

#[test]
fn readme_has_ggen_commands_block() {
    let content = fs::read_to_string("README.md").expect("README.md must exist");
    assert!(
        content.contains("BEGIN ggen:commands"),
        "README.md missing BEGIN ggen:commands"
    );
    assert!(
        content.contains("END ggen:commands"),
        "README.md missing END ggen:commands"
    );
}

#[test]
fn readme_has_custom_introduction() {
    let content = fs::read_to_string("README.md").expect("README.md must exist");
    assert!(
        content.contains("BEGIN custom:introduction"),
        "README.md missing BEGIN custom:introduction"
    );
}

#[test]
fn readme_no_forbidden_terms() {
    let content = fs::read_to_string("README.md").expect("README.md must exist");
    let forbidden = [
        "Nehemiah",
        "Field8",
        "Instinct8",
        "Cargo Court",
        "Truex",
        "CONSTRUCT8",
        "Inspection Gate",
    ];
    for term in &forbidden {
        assert!(
            !content.contains(term),
            "README.md must not contain: {:?}",
            term
        );
    }
}

#[test]
fn docs_no_forbidden_terms() {
    let docs_dir = Path::new("docs");
    if !docs_dir.exists() {
        return;
    }
    let forbidden = [
        "Nehemiah",
        "Field8",
        "Instinct8",
        "Cargo Court",
        "Truex",
        "CONSTRUCT8",
        "Inspection Gate",
    ];
    for entry in walkdir::WalkDir::new(docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();
        // Skip internal docs dirs (wasm4pm, release, testing, contributing, deferred).
        let internal = ["wasm4pm", "release", "testing", "contributing", "deferred"];
        if path
            .components()
            .any(|c| internal.contains(&c.as_os_str().to_str().unwrap_or("")))
        {
            continue;
        }
        let content =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("could not read {:?}", path));
        for term in &forbidden {
            assert!(
                !content.contains(term),
                "{:?} must not contain: {:?}",
                path,
                term
            );
        }
    }
}

#[test]
fn reference_docs_exist() {
    let ref_dir = Path::new("docs/reference/commands");
    let expected = [
        "status.md",
        "git-status.md",
        "git-close.md",
        "publish-run.md",
        "target-prune.md",
        "target-show.md",
        "test-changed.md",
        "trybuild-changed.md",
        "workspace-doctor.md",
    ];
    for name in &expected {
        assert!(
            ref_dir.join(name).exists(),
            "reference doc missing: docs/reference/commands/{}",
            name
        );
    }
}

#[test]
fn reference_docs_have_ggen_blocks() {
    let ref_dir = Path::new("docs/reference/commands");
    for name in &["status.md", "git-close.md"] {
        let path = ref_dir.join(name);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("could not read {:?}", path));
        assert!(
            content.contains("BEGIN ggen:command-reference"),
            "{:?} missing BEGIN ggen:command-reference",
            path
        );
    }
}

#[test]
fn playground_scripts_exist() {
    assert!(
        Path::new("playground/scripts/run-matrix.sh").exists(),
        "playground/scripts/run-matrix.sh must exist"
    );
}

#[test]
fn evidence_emission_not_removed() {
    let content = fs::read_to_string("src/evidence.rs").expect("src/evidence.rs must exist");
    assert!(
        content.contains("ProcessEvent"),
        "src/evidence.rs must define ProcessEvent"
    );
    assert!(
        content.contains("emit_xes"),
        "src/evidence.rs must contain emit_xes"
    );
}

#[test]
fn command_table_from_ontology() {
    let content = fs::read_to_string("README.md").expect("README.md must exist");
    let count = content.matches("| `cargo cicd").count();
    assert!(
        count >= 5,
        "README.md must have >=5 '| `cargo cicd' rows, found {}",
        count
    );
}
