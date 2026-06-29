// src/certification/registry.rs
//
// Safety-critical crate registry — TOML-backed, load/query/format.

use std::path::{Path, PathBuf};

/// Entry in the safety-critical crate registry.
#[derive(Debug, Clone)]
pub struct SafetyCriticalEntry {
    /// Crate name on crates.io.
    pub crate_name: String,
    /// Version that was certified.
    pub version: String,
    /// Certification body that issued the receipt.
    pub cert_body_id: String,
    /// Standards satisfied, e.g. `["IEC 61508 SIL 2", "ISO 26262 ASIL B"]`.
    pub standards: Vec<String>,
    /// Date of certification (YYYY-MM-DD).
    pub certified_at: String,
    /// Receipt hash (SHA-256 hex, prefixed with "sha256:").
    pub receipt_hash: String,
    /// Link to a public evidence archive, if available.
    pub evidence_url: Option<String>,
}

impl SafetyCriticalEntry {
    /// Construct a new entry with all required fields.
    pub fn new(
        crate_name: impl Into<String>,
        version: impl Into<String>,
        cert_body_id: impl Into<String>,
        standards: Vec<String>,
        certified_at: impl Into<String>,
        receipt_hash: impl Into<String>,
    ) -> Self {
        SafetyCriticalEntry {
            crate_name: crate_name.into(),
            version: version.into(),
            cert_body_id: cert_body_id.into(),
            standards,
            certified_at: certified_at.into(),
            receipt_hash: receipt_hash.into(),
            evidence_url: None,
        }
    }

    /// Set the optional evidence URL.
    pub fn with_evidence_url(mut self, url: impl Into<String>) -> Self {
        self.evidence_url = Some(url.into());
        self
    }
}

/// Load the safety-critical crate registry from a TOML file at `path`.
///
/// Uses simple line-by-line parsing; the `toml` crate is available in the
/// workspace but this parser avoids pulling in the full deserialization stack
/// for a flat record format.
///
/// Unknown keys are silently ignored; missing required keys produce a default
/// empty-string value rather than panicking.
pub fn load_registry(path: &Path) -> Vec<SafetyCriticalEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    parse_toml_entries(&content)
}

/// Parse `[[entries]]` blocks from TOML content.
fn parse_toml_entries(content: &str) -> Vec<SafetyCriticalEntry> {
    let mut entries: Vec<SafetyCriticalEntry> = Vec::new();
    let mut current: Option<std::collections::HashMap<String, String>> = None;
    let mut current_standards: Vec<String> = Vec::new();
    let mut in_standards_array = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and blank lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Start of a new entry block
        if trimmed == "[[entries]]" {
            if let Some(map) = current.take() {
                entries.push(map_to_entry(map, current_standards.clone()));
            }
            current = Some(std::collections::HashMap::new());
            current_standards = Vec::new();
            in_standards_array = false;
            continue;
        }

        let Some(ref mut map) = current else {
            continue;
        };

        // Detect start of inline standards array
        if trimmed.starts_with("standards") && trimmed.contains('[') {
            in_standards_array = true;
            // Collect items on this line
            if let Some(bracket_content) = trimmed.find('[').map(|i| &trimmed[i + 1..]) {
                parse_array_items(bracket_content, &mut current_standards);
                if bracket_content.contains(']') {
                    in_standards_array = false;
                }
            }
            continue;
        }

        // Continuation of multi-line standards array
        if in_standards_array {
            parse_array_items(trimmed, &mut current_standards);
            if trimmed.contains(']') {
                in_standards_array = false;
            }
            continue;
        }

        // Key = "value" pairs
        if let Some((key, val)) = parse_kv(trimmed) {
            map.insert(key, val);
        }
    }

    // Flush the last entry
    if let Some(map) = current.take() {
        entries.push(map_to_entry(map, current_standards));
    }

    entries
}

/// Extract string items from a partial TOML array line (handles quotes, commas, brackets).
fn parse_array_items(fragment: &str, out: &mut Vec<String>) {
    let frag = fragment.trim_end_matches(']').trim();
    for part in frag.split(',') {
        let item = part.trim().trim_matches('"').trim_matches('\'').to_string();
        if !item.is_empty() {
            out.push(item);
        }
    }
}

/// Parse a simple `key = "value"` TOML line.
fn parse_kv(line: &str) -> Option<(String, String)> {
    let eq = line.find('=')?;
    let key = line[..eq].trim().to_string();
    let raw_val = line[eq + 1..].trim();
    // Remove surrounding quotes if present
    let val = raw_val.trim_matches('"').trim_matches('\'').to_string();
    Some((key, val))
}

/// Convert a parsed key/value map into a `SafetyCriticalEntry`.
fn map_to_entry(
    map: std::collections::HashMap<String, String>,
    standards: Vec<String>,
) -> SafetyCriticalEntry {
    SafetyCriticalEntry {
        crate_name: map.get("crate_name").cloned().unwrap_or_default(),
        version: map.get("version").cloned().unwrap_or_default(),
        cert_body_id: map.get("cert_body_id").cloned().unwrap_or_default(),
        standards,
        certified_at: map.get("certified_at").cloned().unwrap_or_default(),
        receipt_hash: map.get("receipt_hash").cloned().unwrap_or_default(),
        evidence_url: map.get("evidence_url").cloned(),
    }
}

/// Check if a specific crate + version appears in the registry.
pub fn is_certified(registry: &[SafetyCriticalEntry], crate_name: &str, version: &str) -> bool {
    registry
        .iter()
        .any(|e| e.crate_name == crate_name && e.version == version)
}

/// Format a human-readable listing of all registry entries.
pub fn format_registry_listing(entries: &[SafetyCriticalEntry]) -> String {
    let mut out = String::new();
    out.push_str("Safety-Critical Crate Registry\n");
    out.push_str("==============================\n");

    if entries.is_empty() {
        out.push_str("(no certified crates registered)\n");
        return out;
    }

    for entry in entries {
        out.push_str(&format!("\n  {} v{}\n", entry.crate_name, entry.version));
        out.push_str(&format!("    Cert body: {}\n", entry.cert_body_id));
        out.push_str(&format!("    Standards: {}\n", entry.standards.join(", ")));
        out.push_str(&format!("    Certified: {}\n", entry.certified_at));
        out.push_str(&format!("    Receipt:   {}\n", entry.receipt_hash));
        if let Some(ref url) = entry.evidence_url {
            out.push_str(&format!("    Evidence:  {}\n", url));
        }
    }

    out
}

/// Default registry file path: `{workspace_root}/safety-critical-registry.toml`.
///
/// Uses the current working directory as the workspace root when no root is
/// available from `EngineState` (avoids a circular dependency).
pub fn default_registry_path() -> PathBuf {
    PathBuf::from("safety-critical-registry.toml")
}
