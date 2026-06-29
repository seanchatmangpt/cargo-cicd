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
