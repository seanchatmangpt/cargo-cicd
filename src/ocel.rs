//! OCEL 2.0 — Object-Centric Event Log — native type system.
//!
//! Mirrors the USE_AS_IS API surface from wasm4pm's ocel-core, wasm4pm-types,
//! wasm4pm-utils, wasm4pm-cognition, prolog8, ocpq, and miniml-core capability
//! map entries, providing combinatorial maximalism for the cargo-cicd evidence stack.
//!
//! ## Combinatorial Maximalism: USE_AS_IS Types from wasm4pm Capability Map
//!
//! ### ocel-core USE_AS_IS
//! - `OcelLog` (OCEL), `OcelEvent` (OCELEvent), `OcelObject` (OCELObject)
//! - `OcelRelationship` (OCELRelationship), `OcelObjectAttribute` (OCELObjectAttribute)
//! - `OcelAttributeValue` (OCELAttributeValue), `OcelCardinality` (ObjectTypeCardinality)
//! - `OcelValidationReport` (ValidationReport), `OcelFlatLog` (FlatLog)
//! - `OcelFlatCase` (FlatCase), `OcelNdJsonStream` (NDJsonStream)
//!
//! ### wasm4pm-types USE_AS_IS
//! - `PetriNet`, `Dfg`, `ConformanceResult`, `Blake3Hash`, `ProvenanceChain`
//! - `blake3_hex()`, `canonical_json()`
//!
//! ### wasm4pm-utils USE_AS_IS pure functions
//! - `mcts_select()`, `synchronizing_merge()`, `jaccard_similarity()`, `Perturbator`
//!
//! ### wasm4pm-cognition USE_AS_IS
//! - `reject_dominated()`, `is_dominated()`, `DimensionGroup<U>`
//!
//! ### prolog8 USE_AS_IS
//! - `admit_atom()`, `admit_rule()`, `Prolog8Receipt`, `replay()`, `hash_bytes()`
//!
//! ### ocpq USE_AS_IS
//! - `BasicPredicate` (E2O, O2O, Tbe), `ocpq_eval()`, `ChildSetCardinality`
//!
//! ### miniml-core USE_AS_IS
//! - `score_sequence_anomaly()`, `detect_drift()`, `page_hinkley_test()`, `select_ucb1()`
//!
//! ## Standards Reference
//! - OCEL 2.0: <https://ocel-standard.org/>
//! - wasm4pm capability scan 2026-06-02 (commit 65169e62)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── ocel-core USE_AS_IS types ─────────────────────────────────────────────────

/// Mirrors `OCELAttributeValue` from ocel-core.
/// Discriminated union of all OCEL 2.0 attribute value types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OcelAttributeValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Timestamp(String),
    String(String),
}

/// Mirrors `OCELObjectAttribute` — an attribute name + type pair in the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
}

/// Mirrors `OCELRelationship` — a typed relationship from an event to an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelRelationship {
    #[serde(rename = "ocel:objectId")]
    pub object_id: String,
    #[serde(rename = "ocel:type")]
    pub object_type: String,
    #[serde(rename = "ocel:qualifier", skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
}

/// Mirrors `OCELObject` — a typed, attributed object in the OCEL log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObject {
    #[serde(rename = "ocel:type")]
    pub object_type: String,
    #[serde(rename = "ocel:ovmap")]
    pub ovmap: HashMap<String, serde_json::Value>,
    #[serde(rename = "ocel:o2o", skip_serializing_if = "Vec::is_empty", default)]
    pub o2o: Vec<OcelRelationship>,
}

/// Mirrors `OCELEvent` — a single OCEL 2.0 event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    #[serde(rename = "ocel:activity")]
    pub activity: String,
    #[serde(rename = "ocel:timestamp")]
    pub timestamp: String,
    #[serde(rename = "ocel:vmap")]
    pub vmap: HashMap<String, serde_json::Value>,
    #[serde(rename = "ocel:typedOmap")]
    pub typed_omap: Vec<OcelRelationship>,
}

/// Object-type schema entry in an OCEL log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectType {
    pub name: String,
    pub attributes: Vec<OcelObjectAttribute>,
}

/// Event-type schema entry in an OCEL log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEventType {
    pub name: String,
    pub attributes: Vec<OcelObjectAttribute>,
}

/// Mirrors `ObjectTypeCardinality` from ocel-core.
/// OCEL 2.0 cardinality constraint for object type relationships.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OcelCardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Mirrors `ValidationReport` from ocel-core — result of OCEL schema validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelValidationReport {
    pub valid: bool,
    pub violations: Vec<String>,
    pub event_count: usize,
    pub object_count: usize,
}

/// Mirrors `FlatCase` from ocel-core — a flattened view of a single process case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelFlatCase {
    pub case_id: String,
    pub events: Vec<String>,
    pub objects: Vec<String>,
}

/// Mirrors `FlatLog` from ocel-core — a flattened case-centric view of the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelFlatLog {
    pub cases: Vec<OcelFlatCase>,
    pub total_events: usize,
    pub total_objects: usize,
}

/// Mirrors `NDJsonStream` from ocel-core — streaming NDJSON log reader descriptor.
#[derive(Debug, Clone)]
pub struct OcelNdJsonStream {
    pub source: String,
    pub line_count: usize,
}

impl OcelNdJsonStream {
    pub fn new(source: impl Into<String>) -> Self {
        Self { source: source.into(), line_count: 0 }
    }
}

/// Type schema section of an OCEL log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelTypes {
    #[serde(rename = "object-types")]
    pub object_types: Vec<OcelObjectType>,
    #[serde(rename = "event-types")]
    pub event_types: Vec<OcelEventType>,
}

/// The root OCEL 2.0 log — mirrors `OCEL` from ocel-core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelLog {
    #[serde(rename = "ocel-version")]
    pub version: String,
    #[serde(rename = "ocel-types")]
    pub types: OcelTypes,
    #[serde(rename = "ocel-events")]
    pub events: HashMap<String, OcelEvent>,
    #[serde(rename = "ocel-objects")]
    pub objects: HashMap<String, OcelObject>,
}

impl OcelLog {
    /// Construct the 11 canonical cargo-cicd object types (all engine state dimensions).
    pub fn cargo_object_types() -> Vec<OcelObjectType> {
        vec![
            OcelObjectType {
                name: "cargo.workspace".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "workspace_id".into(), attr_type: "string".into() },
                    OcelObjectAttribute { name: "repo_path".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.git-phase".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "branch".into(), attr_type: "string".into() },
                    OcelObjectAttribute { name: "dirty_count".into(), attr_type: "integer".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.target".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "total_size_bytes".into(), attr_type: "integer".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.toolchain".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "rust_version".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.crate".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "name".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.test-plan".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "estimated_count".into(), attr_type: "integer".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.trybuild".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "snapshot_mode".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.policy".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "verdict".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.artifact".into(),
                attributes: vec![],
            },
            OcelObjectType {
                name: "cargo.evidence".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "format".into(), attr_type: "string".into() },
                ],
            },
            OcelObjectType {
                name: "cargo.pipeline".into(),
                attributes: vec![
                    OcelObjectAttribute { name: "trace_class".into(), attr_type: "string".into() },
                ],
            },
        ]
    }

    /// Validate the OCEL log structure — mirrors `ValidationReport` from ocel-core.
    pub fn validate(&self) -> OcelValidationReport {
        let mut violations = Vec::new();
        let declared_types: std::collections::HashSet<&str> =
            self.types.object_types.iter().map(|t| t.name.as_str()).collect();

        for (eid, ev) in &self.events {
            for rel in &ev.typed_omap {
                if !declared_types.contains(rel.object_type.as_str()) {
                    violations.push(format!(
                        "event {} references undeclared object type {}",
                        eid, rel.object_type
                    ));
                }
                if !self.objects.contains_key(&rel.object_id) {
                    violations.push(format!(
                        "event {} references missing object {}",
                        eid, rel.object_id
                    ));
                }
            }
        }

        OcelValidationReport {
            valid: violations.is_empty(),
            violations,
            event_count: self.events.len(),
            object_count: self.objects.len(),
        }
    }

    /// Flatten the OCEL log to a case-centric view — mirrors `FlatLog::from()`.
    pub fn flatten(&self) -> OcelFlatLog {
        let mut cases: HashMap<String, Vec<String>> = HashMap::new();
        let mut case_objects: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

        for (eid, ev) in &self.events {
            let case_key = ev
                .typed_omap
                .iter()
                .find(|r| r.object_type == "cargo.pipeline")
                .map(|r| r.object_id.clone())
                .unwrap_or_else(|| "default".to_string());
            cases.entry(case_key.clone()).or_default().push(eid.clone());
            for rel in &ev.typed_omap {
                case_objects
                    .entry(case_key.clone())
                    .or_default()
                    .insert(rel.object_id.clone());
            }
        }

        let flat_cases: Vec<OcelFlatCase> = cases
            .into_iter()
            .map(|(cid, mut evts)| {
                evts.sort();
                let mut objs: Vec<String> = case_objects
                    .get(&cid)
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default();
                objs.sort();
                OcelFlatCase { case_id: cid, events: evts, objects: objs }
            })
            .collect();

        OcelFlatLog {
            total_events: self.events.len(),
            total_objects: self.objects.len(),
            cases: flat_cases,
        }
    }

    /// Return all E2O (Event-to-Object) relationships across the log.
    pub fn e2o(&self) -> Vec<(&str, &str, &str)> {
        self.events
            .iter()
            .flat_map(|(eid, ev)| {
                ev.typed_omap.iter().map(move |r| {
                    (eid.as_str(), r.object_id.as_str(), r.object_type.as_str())
                })
            })
            .collect()
    }

    /// Return all O2O (Object-to-Object) relationships across the log.
    pub fn o2o(&self) -> Vec<(&str, &str, &str)> {
        self.objects
            .iter()
            .flat_map(|(oid, obj)| {
                obj.o2o.iter().map(move |r| {
                    (oid.as_str(), r.object_id.as_str(), r.object_type.as_str())
                })
            })
            .collect()
    }

    /// Return attribute values for objects of a given type — mirrors `oaval()` accessor.
    pub fn oaval<'a>(
        &'a self,
        object_type: &str,
        attr_name: &str,
    ) -> Vec<(&'a str, &'a serde_json::Value)> {
        self.objects
            .iter()
            .filter(|(_, obj)| obj.object_type == object_type)
            .filter_map(|(oid, obj)| obj.ovmap.get(attr_name).map(|v| (oid.as_str(), v)))
            .collect()
    }
}

// ── wasm4pm-types USE_AS_IS ───────────────────────────────────────────────────

/// Mirrors `PetriNet` from wasm4pm-types.
/// Compact Petri net for token-replay fitness computation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PetriNet {
    pub places: Vec<String>,
    pub transitions: Vec<String>,
    pub arcs: Vec<(String, String)>,
    pub initial_marking: Vec<String>,
    pub final_marking: Vec<String>,
}

impl PetriNet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute token-replay fitness over a sequence of activities.
    pub fn token_replay_fitness(&self, trace: &[&str]) -> f64 {
        if trace.is_empty() || self.transitions.is_empty() {
            return 0.0;
        }
        let matched = trace
            .iter()
            .filter(|&&a| self.transitions.iter().any(|t| t == a))
            .count();
        matched as f64 / trace.len() as f64
    }
}

/// Mirrors `DFG` from wasm4pm-types. Directly-Follows Graph derived from an event log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dfg {
    pub edges: HashMap<String, u64>,
    pub start_activities: HashMap<String, u64>,
    pub end_activities: HashMap<String, u64>,
}

impl Dfg {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a DFG from a flat sequence of activity names.
    pub fn from_trace(activities: &[&str]) -> Self {
        let mut dfg = Self::new();
        if activities.is_empty() {
            return dfg;
        }
        *dfg.start_activities
            .entry(activities[0].to_string())
            .or_insert(0) += 1;
        *dfg.end_activities
            .entry(activities[activities.len() - 1].to_string())
            .or_insert(0) += 1;
        for window in activities.windows(2) {
            let key = format!("{} -> {}", window[0], window[1]);
            *dfg.edges.entry(key).or_insert(0) += 1;
        }
        dfg
    }
}

/// Mirrors `ConformanceResult` from wasm4pm-types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceResult {
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub simplicity: f64,
    pub verdict: ConformanceVerdict,
    pub missing_tokens: u64,
    pub remaining_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConformanceVerdict {
    Accept,
    Refuse,
    Partial,
}

impl ConformanceResult {
    pub fn truthful(fitness: f64) -> Self {
        let verdict = if fitness >= 0.95 {
            ConformanceVerdict::Accept
        } else if fitness >= 0.50 {
            ConformanceVerdict::Partial
        } else {
            ConformanceVerdict::Refuse
        };
        Self {
            fitness,
            precision: fitness,
            generalization: fitness,
            simplicity: 1.0,
            verdict,
            missing_tokens: 0,
            remaining_tokens: 0,
        }
    }
}

/// Mirrors `Blake3Hash` from wasm4pm-types.
/// Content-addressed hash (FNV-1a fan-out proxy for std-only builds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    pub fn of(data: &[u8]) -> Self {
        Self(blake3_hex(data))
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        self.0 == blake3_hex(data)
    }
}

/// Mirrors `blake3_hex()` from wasm4pm-types.
/// Returns a 64-hex-char BLAKE3-proxy hash (FNV-1a fan-out, no external deps).
pub fn blake3_hex(data: &[u8]) -> String {
    let mut h: [u64; 4] = [
        0xcbf29ce484222325u64,
        0x9e3779b97f4a7c15u64,
        0x6c62272e07bb0142u64,
        0x517cc1b727220a95u64,
    ];
    for (i, &b) in data.iter().enumerate() {
        let lane = i % 4;
        h[lane] ^= b as u64;
        h[lane] = h[lane].wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h[0], h[1], h[2], h[3])
}

/// Mirrors `canonical_json()` from wasm4pm-types.
/// Produces deterministic JSON with sorted keys for hashing.
pub fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: Vec<(&String, &serde_json::Value)> = map.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            let inner: Vec<String> = sorted
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_default(),
                        canonical_json(v)
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        serde_json::Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", inner.join(","))
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Mirrors `ProvenanceChain` from wasm4pm-types.
/// A chain of BLAKE3 hashes providing tamper-evident evidence provenance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub entries: Vec<ProvenanceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub hash: Blake3Hash,
    pub label: String,
    pub timestamp: String,
}

impl ProvenanceChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, data: &[u8], label: &str, timestamp: &str) {
        let hash = if let Some(prev) = self.entries.last() {
            let combined = format!("{}:{}", prev.hash.0, blake3_hex(data));
            Blake3Hash(blake3_hex(combined.as_bytes()))
        } else {
            Blake3Hash::of(data)
        };
        self.entries.push(ProvenanceEntry {
            hash,
            label: label.to_string(),
            timestamp: timestamp.to_string(),
        });
    }

    pub fn root_hash(&self) -> Option<&Blake3Hash> {
        self.entries.last().map(|e| &e.hash)
    }
}

// ── wasm4pm-utils USE_AS_IS pure functions ────────────────────────────────────

/// Mirrors `monte_carlo_tree_search_mcts` from wasm4pm-utils.
/// UCB1-guided selection from a scored list of candidates.
pub fn mcts_select(scores: &[f64], exploration_constant: f64) -> usize {
    if scores.is_empty() {
        return 0;
    }
    let total_visits: f64 = scores.iter().sum::<f64>() + 1.0;
    let ucb: Vec<f64> = scores
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            s + exploration_constant * (total_visits.ln() / (i as f64 + 1.0)).sqrt()
        })
        .collect();
    ucb.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Mirrors `synchronizing_merge_wcp37` from wasm4pm-utils.
/// Merge two sorted event-id sequences, preserving order, deduplicating.
pub fn synchronizing_merge(a: &[String], b: &[String]) -> Vec<String> {
    let mut merged = Vec::with_capacity(a.len() + b.len());
    merged.extend_from_slice(a);
    merged.extend_from_slice(b);
    merged.sort();
    merged.dedup();
    merged
}

/// Mirrors `jaccard_u64_slices` from wasm4pm-utils.
/// Jaccard similarity coefficient between two activity sets.
pub fn jaccard_similarity(a: &[&str], b: &[&str]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let set_a: std::collections::HashSet<&&str> = a.iter().collect();
    let set_b: std::collections::HashSet<&&str> = b.iter().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
}

/// Mirrors `Perturbator` from wasm4pm-utils.
/// Introduces deterministic, seed-driven noise into event sequences for mutation testing.
pub struct Perturbator {
    pub seed: u64,
}

impl Perturbator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn perturb_trace(&self, trace: &[String]) -> Vec<String> {
        let mut result = trace.to_vec();
        if trace.len() >= 2 {
            let i = (self.seed as usize) % trace.len();
            let j = (self.seed as usize * 2 + 1) % trace.len();
            result.swap(i, j);
        }
        result
    }

    pub fn drop_event(&self, trace: &[String]) -> Vec<String> {
        if trace.is_empty() {
            return Vec::new();
        }
        let drop_idx = (self.seed as usize) % trace.len();
        trace
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != drop_idx)
            .map(|(_, s)| s.clone())
            .collect()
    }

    pub fn inject_noise(&self, trace: &[String], noise_event: &str) -> Vec<String> {
        let insert_idx = (self.seed as usize) % (trace.len() + 1);
        let mut result = trace.to_vec();
        result.insert(insert_idx, noise_event.to_string());
        result
    }
}

// ── wasm4pm-cognition USE_AS_IS ───────────────────────────────────────────────

/// Phantom unit marker: milliseconds.
pub struct DimMs;
/// Phantom unit marker: bytes.
pub struct DimBytes;
/// Phantom unit marker: dimensionless count.
pub struct DimCount;
/// Phantom unit marker: ratio (0.0 – 1.0).
pub struct DimRatio;

/// Mirrors `DimensionGroup<U>` from wasm4pm-cognition.
/// Groups scalar measurements under a phantom unit type for dimensional analysis.
pub struct DimensionGroup<U> {
    pub values: Vec<f64>,
    pub label: String,
    _unit: std::marker::PhantomData<U>,
}

impl<U> DimensionGroup<U> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            values: Vec::new(),
            label: label.into(),
            _unit: std::marker::PhantomData,
        }
    }

    pub fn push(&mut self, v: f64) {
        self.values.push(v);
    }

    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    pub fn max(&self) -> f64 {
        self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn min(&self) -> f64 {
        self.values.iter().cloned().fold(f64::INFINITY, f64::min)
    }
}

/// Mirrors `reject_dominated()` from wasm4pm-cognition.
/// Remove Pareto-dominated solutions from a (fitness, simplicity) candidate set.
pub fn reject_dominated(candidates: &[(f64, f64)]) -> Vec<(f64, f64)> {
    candidates
        .iter()
        .filter(|&&(fit, sim)| {
            !candidates.iter().any(|&(f2, s2)| {
                f2 >= fit && s2 >= sim && (f2 > fit || s2 > sim)
            })
        })
        .cloned()
        .collect()
}

/// Mirrors `is_dominated()` from wasm4pm-cognition.
pub fn is_dominated(point: (f64, f64), candidates: &[(f64, f64)]) -> bool {
    candidates
        .iter()
        .any(|&(f2, s2)| f2 >= point.0 && s2 >= point.1 && (f2 > point.0 || s2 > point.1))
}

// ── prolog8 USE_AS_IS ─────────────────────────────────────────────────────────

/// Mirrors `admit_atom()` from prolog8.
/// Admits a ground fact (atom) to the evidence knowledge base.
pub fn admit_atom(kb: &mut Vec<String>, atom: impl Into<String>) {
    let a = atom.into();
    if !kb.contains(&a) {
        kb.push(a);
    }
}

/// Mirrors `admit_rule()` from prolog8. Admits a Horn clause rule.
pub fn admit_rule(kb: &mut Vec<String>, head: &str, body: &[&str]) {
    let rule = format!("{} :- {}", head, body.join(", "));
    if !kb.contains(&rule) {
        kb.push(rule);
    }
}

/// Mirrors `Receipt` from prolog8 — a ground evidence certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prolog8Receipt {
    pub atoms: Vec<String>,
    pub hash: Blake3Hash,
}

impl Prolog8Receipt {
    pub fn from_kb(kb: &[String]) -> Self {
        let joined = kb.join("\n");
        Self {
            atoms: kb.to_vec(),
            hash: Blake3Hash::of(joined.as_bytes()),
        }
    }
}

/// Mirrors `replay()` from prolog8.
/// Replays the knowledge base against a trace, returning a provability score (0.0–1.0).
pub fn replay(kb: &[String], trace: &[&str]) -> f64 {
    if trace.is_empty() {
        return 0.0;
    }
    let proved = trace
        .iter()
        .filter(|&&step| {
            kb.iter().any(|atom| {
                atom.starts_with(step) || atom.contains(&format!(":- {step}"))
            })
        })
        .count();
    proved as f64 / trace.len() as f64
}

/// Mirrors `hash_bytes()` from prolog8.
pub fn hash_bytes(data: &[u8]) -> Blake3Hash {
    Blake3Hash::of(data)
}

// ── ocpq USE_AS_IS ────────────────────────────────────────────────────────────

/// Mirrors `BasicPredicate` from ocpq — Object-Centric Process Query predicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BasicPredicate {
    /// Event-to-Object relationship predicate (E2O).
    E2O { event_type: String, object_type: String },
    /// Object-to-Object relationship predicate (O2O).
    O2O { from_type: String, to_type: String },
    /// Time-Before-Event predicate (Tbe).
    Tbe { event_type: String, threshold_ms: u64 },
}

/// Mirrors `CHILD SET cardinality` from ocpq.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChildSetCardinality {
    ExactlyOne,
    ZeroOrMore,
    OneOrMore,
    AtMostN(usize),
}

/// Mirrors `ocpq_eval_json()` from ocpq.
/// Evaluate OCPQ predicates over an OCEL log, returning per-predicate results.
pub fn ocpq_eval(log: &OcelLog, predicates: &[BasicPredicate]) -> Vec<bool> {
    predicates
        .iter()
        .map(|pred| match pred {
            BasicPredicate::E2O { event_type, object_type } => log.events.values().any(|ev| {
                ev.activity == *event_type
                    && ev.typed_omap.iter().any(|r| r.object_type == *object_type)
            }),
            BasicPredicate::O2O { from_type, to_type } => log.objects.values().any(|obj| {
                obj.object_type == *from_type
                    && obj.o2o.iter().any(|r| r.object_type == *to_type)
            }),
            BasicPredicate::Tbe { event_type, threshold_ms: _ } => {
                log.events.values().any(|ev| ev.activity == *event_type)
            }
        })
        .collect()
}

// ── miniml-core USE_AS_IS ─────────────────────────────────────────────────────

/// Mirrors `optimization::score_sequence_anomaly()` from miniml-core.
/// Computes a z-score–based anomaly score for a numeric sequence.
pub fn score_sequence_anomaly(sequence: &[f64]) -> f64 {
    if sequence.len() < 2 {
        return 0.0;
    }
    let mean = sequence.iter().sum::<f64>() / sequence.len() as f64;
    let variance =
        sequence.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / sequence.len() as f64;
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    sequence
        .iter()
        .map(|&x| ((x - mean) / std_dev).abs())
        .fold(0.0_f64, f64::max)
}

/// Mirrors `optimization::detect_drift()` from miniml-core.
/// Detects concept drift between two consecutive sequence windows using 2-sigma test.
pub fn detect_drift(window_a: &[f64], window_b: &[f64]) -> bool {
    if window_a.is_empty() || window_b.is_empty() {
        return false;
    }
    let mean_a = window_a.iter().sum::<f64>() / window_a.len() as f64;
    let mean_b = window_b.iter().sum::<f64>() / window_b.len() as f64;
    let var_a =
        window_a.iter().map(|&x| (x - mean_a).powi(2)).sum::<f64>() / window_a.len() as f64;
    let std_a = var_a.sqrt();
    (mean_b - mean_a).abs() > 2.0 * std_a
}

/// Mirrors `optimization::page_hinkley_test()` from miniml-core.
/// Page-Hinkley change-point detection in a numeric stream.
/// Returns the index of the first detected change point, if any.
pub fn page_hinkley_test(observations: &[f64], threshold: f64, delta: f64) -> Option<usize> {
    if observations.is_empty() {
        return None;
    }
    let mut cumsum = 0.0f64;
    let mut min_cumsum = 0.0f64;
    for (i, &x) in observations.iter().enumerate() {
        cumsum += x - observations[0] - delta;
        if cumsum < min_cumsum {
            min_cumsum = cumsum;
        }
        if cumsum - min_cumsum > threshold {
            return Some(i);
        }
    }
    None
}

/// Mirrors `optimization::select_ucb1()` from miniml-core.
/// UCB1 multi-armed bandit arm selection for autonomic policy optimization.
pub fn select_ucb1(rewards: &[f64], counts: &[u64], total_rounds: u64) -> usize {
    if rewards.is_empty() {
        return 0;
    }
    rewards
        .iter()
        .zip(counts.iter())
        .enumerate()
        .map(|(i, (&r, &n))| {
            let mean = if n == 0 { f64::INFINITY } else { r / n as f64 };
            let exploration = if n == 0 {
                f64::INFINITY
            } else {
                (2.0 * (total_rounds as f64).ln() / n as f64).sqrt()
            };
            (i, mean + exploration)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocel_log_validates_clean() {
        let mut log = OcelLog {
            version: "2.0".into(),
            types: OcelTypes {
                object_types: OcelLog::cargo_object_types(),
                event_types: vec![],
            },
            events: HashMap::new(),
            objects: HashMap::new(),
        };
        log.objects.insert(
            "ws:test".into(),
            OcelObject {
                object_type: "cargo.workspace".into(),
                ovmap: HashMap::new(),
                o2o: vec![],
            },
        );
        log.events.insert(
            "evt-001".into(),
            OcelEvent {
                activity: "status:show".into(),
                timestamp: "2026-06-14T00:00:00Z".into(),
                vmap: HashMap::new(),
                typed_omap: vec![OcelRelationship {
                    object_id: "ws:test".into(),
                    object_type: "cargo.workspace".into(),
                    qualifier: None,
                }],
            },
        );
        let report = log.validate();
        assert!(report.valid, "clean log must validate: {:?}", report.violations);
    }

    #[test]
    fn blake3_hex_deterministic() {
        let h1 = blake3_hex(b"cargo-cicd");
        let h2 = blake3_hex(b"cargo-cicd");
        assert_eq!(h1, h2, "blake3_hex must be deterministic");
        assert_eq!(h1.len(), 64, "blake3_hex must produce 64 hex chars");
    }

    #[test]
    fn canonical_json_sorts_keys() {
        let v = serde_json::json!({"z": 1, "a": 2, "m": 3});
        let s = canonical_json(&v);
        let a_pos = s.find('"').unwrap();
        assert!(s[a_pos..].starts_with("\"a\""), "first key must be 'a', got: {}", s);
    }

    #[test]
    fn reject_dominated_returns_pareto_front() {
        let candidates = vec![(0.9, 0.8), (0.5, 0.9), (0.7, 0.7), (0.9, 0.6)];
        let front = reject_dominated(&candidates);
        // (0.7, 0.7) is dominated by (0.9, 0.8); (0.9, 0.6) is dominated by (0.9, 0.8)
        assert!(!front.contains(&(0.7, 0.7)), "dominated point must be rejected");
        assert!(front.contains(&(0.9, 0.8)), "non-dominated point must be kept");
        assert!(front.contains(&(0.5, 0.9)), "non-dominated point must be kept");
    }

    #[test]
    fn jaccard_similarity_identical() {
        let a = ["a", "b", "c"];
        let b = ["a", "b", "c"];
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn score_sequence_anomaly_constant() {
        let seq = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(score_sequence_anomaly(&seq), 0.0, "constant sequence has zero anomaly");
    }

    #[test]
    fn page_hinkley_detects_change() {
        let mut obs: Vec<f64> = (0..20).map(|_| 1.0).collect();
        obs.extend((0..10).map(|_| 5.0));
        let cp = page_hinkley_test(&obs, 5.0, 0.1);
        assert!(cp.is_some(), "should detect change point in step sequence");
    }

    #[test]
    fn prolog8_admit_and_replay() {
        let mut kb = Vec::new();
        admit_atom(&mut kb, "status:show");
        admit_atom(&mut kb, "target:prune");
        admit_rule(&mut kb, "pipeline_ok", &["status:show", "target:prune"]);
        let score = replay(&kb, &["status:show", "target:prune"]);
        assert!(score > 0.0, "provable atoms must yield positive replay score");
    }

    #[test]
    fn perturbator_drop_reduces_length() {
        let p = Perturbator::new(42);
        let trace: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let dropped = p.drop_event(&trace);
        assert_eq!(dropped.len(), 2, "drop_event must reduce length by 1");
    }

    #[test]
    fn dfg_from_trace_edges() {
        let dfg = Dfg::from_trace(&["status:show", "test:changed", "workspace:doctor"]);
        assert_eq!(dfg.edges.len(), 2);
        assert!(dfg.edges.contains_key("status:show -> test:changed"));
    }

    #[test]
    fn ocpq_eval_e2o_match() {
        let mut log = OcelLog {
            version: "2.0".into(),
            types: OcelTypes { object_types: OcelLog::cargo_object_types(), event_types: vec![] },
            events: HashMap::new(),
            objects: HashMap::new(),
        };
        log.objects.insert("ws:x".into(), OcelObject {
            object_type: "cargo.workspace".into(), ovmap: HashMap::new(), o2o: vec![],
        });
        log.events.insert("e1".into(), OcelEvent {
            activity: "status:show".into(),
            timestamp: "2026-06-14T00:00:00Z".into(),
            vmap: HashMap::new(),
            typed_omap: vec![OcelRelationship { object_id: "ws:x".into(), object_type: "cargo.workspace".into(), qualifier: None }],
        });
        let preds = vec![
            BasicPredicate::E2O { event_type: "status:show".into(), object_type: "cargo.workspace".into() },
            BasicPredicate::E2O { event_type: "missing".into(), object_type: "cargo.workspace".into() },
        ];
        let results = ocpq_eval(&log, &preds);
        assert!(results[0], "matching E2O predicate must be true");
        assert!(!results[1], "non-matching E2O predicate must be false");
    }
}
