# ADR-015: Emit JSONL Companion Alongside XES for Streaming Parsers

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** evidence, jsonl, xes, streaming, analytics, dual-format

---

## Context

ADR-011 established XES 2.0 as the canonical evidence format for cargo-cicd. XES is authoritative for process mining conformance checking and oracle adjudication. However, the Rust ecosystem and DevOps tooling landscape includes a large class of consumers that work natively with JSONL (newline-delimited JSON, also called NDJSON) rather than XML:

1. **Stream processors**: Apache Kafka, Logstash, Fluent Bit, and Vector all have native JSONL support. XML requires a separate processing plugin.

2. **Analytics pipelines**: Grafana Loki, Elasticsearch, and similar log analytics tools ingest JSONL natively. XES ingest requires custom adapters.

3. **CLI tooling**: `jq`, `miller` (`mlr`), `fx`, and similar tools work on JSONL. Processing XES in shell scripts requires `xmllint` or `xq`.

4. **Real-time monitoring**: The process mining dashboard (see `docs/process-mining-architecture.md`) uses a streaming event ingestion pipeline. JSONL is more efficient for real-time fan-out than XML.

5. **Custom scripts**: Teams writing automation around cargo-cicd evidence frequently prefer JSON over XML for programmatic consumption.

6. **AI/ML pipelines**: Training or running ML models on process event sequences (see Phase 3 anti-pattern detection) requires JSON-serializable feature vectors, not XML.

### Problem with XES-Only Emission

If cargo-cicd emits only XES:

- Teams using Grafana for CI/CD observability must write XES-to-JSONL converters.
- Real-time dashboard ingestion requires XML streaming parsers (slower, more complex).
- Shell-script automation requires xmllint (not universally available).
- jq-based filtering of evidence events is not possible.

### Problem with JSONL-Only Emission

If cargo-cicd emits only JSONL:

- ProM/Disco import requires an XES converter.
- Oracle adjudication requires an XES converter (wpm consumes XES).
- ISO/IEC 20880:2013 conformance is not satisfied.
- Regulatory certification that requires XES is not supported without conversion.

### Dual Emission as the Solution

Emit both formats simultaneously at command execution time. The formats are complementary:

| Concern | Authority |
|---------|-----------|
| Oracle adjudication | XES (authoritative) |
| Conformance checking | XES (authoritative) |
| Long-term archival | XES (authoritative) |
| Regulatory compliance | XES (authoritative) |
| Real-time streaming | JSONL (preferred) |
| Analytics pipelines | JSONL (preferred) |
| CLI tooling | JSONL (preferred) |
| AI/ML pipelines | JSONL (preferred) |

---

## Decision

**Emit a JSONL companion file alongside every XES evidence file. XES is authoritative; JSONL is the streaming and analytics-friendly representation of the same events.**

### File Naming Convention

For each evidence emission, two files are created in `target/cargo-cicd/evidence/`:

```
target/cargo-cicd/evidence/
├── evt-status-show-20260617T140000Z.xes     ← Canonical (authoritative)
└── evt-status-show-20260617T140000Z.jsonl   ← Companion (streaming)
```

The base filename is identical; only the extension differs. This enables atomic pairing: if the XES file exists, the JSONL companion is expected to exist alongside it.

### JSONL Schema

Each line in the JSONL companion is one JSON object representing one event. The schema is a flattened projection of the XES event attributes:

```jsonl
{"event_id":"evt-status-show-20260617T140000Z-start","timestamp":"2026-06-17T14:00:00.000Z","command":"status show","lifecycle_transition":"start","verdict_claimed":"PASS","trace_class":"live_workspace","case_id":"status_show_phase","workspace_id":"cargo-cicd@/home/user/cargo-cicd","oracle_key_fingerprint":"pending","schema_version":"2.0"}
{"event_id":"evt-status-show-20260617T140001234Z-complete","timestamp":"2026-06-17T14:00:01.234Z","command":"status show","lifecycle_transition":"complete","verdict_claimed":"PASS","duration_ms":1234,"trace_class":"live_workspace","case_id":"status_show_phase","workspace_id":"cargo-cicd@/home/user/cargo-cicd","oracle_key_fingerprint":"SHA256:Bz3k...","schema_version":"2.0"}
```

### JSONL Field Mapping to XES Attributes

| JSONL Field | XES Attribute | Notes |
|-------------|---------------|-------|
| `event_id` | `concept:name` (event) | Unique event identifier |
| `timestamp` | `time:timestamp` | ISO 8601 with milliseconds |
| `command` | `cargoCI:command` (trace) | From trace, repeated per event |
| `lifecycle_transition` | `lifecycle:transition` | "start" or "complete" |
| `verdict_claimed` | `cargoCI:verdict_claimed` | cargo-cicd's self-assessment |
| `duration_ms` | `cargoCI:duration_ms` | Only on "complete" events |
| `trace_class` | `cargoCI:trace_class` | "live_workspace" or "pipeline_run" |
| `case_id` | `concept:name` (trace) | Groups events into a trace |
| `workspace_id` | `cargoCI:workspace_id` (log) | From log attributes |
| `oracle_key_fingerprint` | `cargoCI:oracle_key_fingerprint` (trace) | "pending" until adjudicated |
| `schema_version` | `xes.version` (log) | Evidence schema version |

### Emission Timing

Both files are written atomically at the same time:

```rust
// In src/evidence.rs
pub fn emit_evidence(event: &ProcessEvent) -> Result<EvidencePaths> {
    let base_name = format!(
        "evt-{}-{}Z",
        event.command.replace(' ', "-"),
        event.timestamp_iso.replace([':', '.'], "")
    );
    let evidence_dir = Path::new("target/cargo-cicd/evidence");
    std::fs::create_dir_all(evidence_dir)?;

    let xes_path = evidence_dir.join(format!("{}.xes", base_name));
    let jsonl_path = evidence_dir.join(format!("{}.jsonl", base_name));

    // Write XES (canonical)
    let xes_content = serialize_to_xes(event)?;
    write_atomic(&xes_path, &xes_content)?;

    // Write JSONL (companion)
    let jsonl_content = serialize_to_jsonl(event)?;
    write_atomic(&jsonl_path, &jsonl_content)?;

    Ok(EvidencePaths { xes_path, jsonl_path })
}
```

### JSONL After Oracle Adjudication

After oracle adjudication, both files are updated with the oracle verdict:

- XES: `cargoCI:oracle_key_fingerprint` attribute is updated from "pending" to the actual fingerprint.
- JSONL: The second-to-last line (the "complete" event line) has `oracle_key_fingerprint` updated.

This keeps both files consistent post-adjudication.

### JSONL for Pipeline Runs

For pipeline runs (`cargo cicd pipeline run`), multiple commands execute in sequence. The pipeline JSONL file accumulates all events across all commands:

```jsonl
{"event_id":"evt-pipeline-20260617-start","command":"pipeline run","lifecycle_transition":"start",...}
{"event_id":"evt-status-show-20260617-start","command":"status show","lifecycle_transition":"start",...}
{"event_id":"evt-status-show-20260617-complete","command":"status show","lifecycle_transition":"complete",...}
{"event_id":"evt-test-changed-20260617-start","command":"test changed","lifecycle_transition":"start",...}
{"event_id":"evt-test-changed-20260617-complete","command":"test changed","lifecycle_transition":"complete",...}
{"event_id":"evt-pipeline-20260617-complete","command":"pipeline run","lifecycle_transition":"complete",...}
```

This is the streaming-friendly representation of the pipeline trace. Real-time dashboards can `tail -f` the JSONL file to observe pipeline progress.

### Streaming Integration Example

```bash
# Real-time monitoring with jq
tail -f target/cargo-cicd/evidence/evt-pipeline-*.jsonl | \
  jq -r 'select(.lifecycle_transition=="complete") | "\(.command): \(.verdict_claimed) (\(.duration_ms)ms)"'

# Kafka producer (via kcat/kafkacat)
tail -f target/cargo-cicd/evidence/*.jsonl | \
  kcat -P -b localhost:9092 -t cargo-cicd-evidence

# Elasticsearch ingest
curl -X POST "localhost:9200/cargo-cicd-evidence/_bulk" \
  -H "Content-Type: application/x-ndjson" \
  --data-binary @target/cargo-cicd/evidence/evt-pipeline-*.jsonl
```

---

## Consequences

### Positive

1. **Streaming compatibility**: Real-time dashboards, Kafka producers, Logstash pipelines, and Fluent Bit collectors can consume JSONL without any conversion layer.

2. **CLI tooling**: `jq`, `mlr`, and similar tools work immediately on JSONL evidence. Ad-hoc analysis is possible without XML expertise.

3. **Analytics pipeline integration**: Elasticsearch, Grafana Loki, and similar tools ingest JSONL natively. The process mining dashboard's ingestion service is simplified.

4. **No conversion overhead**: JSONL is emitted at the same time as XES, at command execution. There is no post-hoc conversion job to schedule.

5. **Pairing is explicit**: Identical base filenames make XES↔JSONL pairing unambiguous. Orphaned XES files (missing JSONL companions) are detectable by `evidence doctor`.

6. **AI/ML compatibility**: JSONL is the standard format for LLM fine-tuning datasets and ML feature pipelines. Phase 3 anti-pattern detection (see `docs/PHASE-3-DESIGN.md`) can train directly on JSONL evidence logs.

### Negative

1. **Storage duplication**: Both XES and JSONL are written for every command. A typical command invocation produces ~2KB XES + ~300B JSONL = ~2.3KB total. At 1000 commands/day, this is ~2.3MB/day, doubling the XES-only storage. Mitigation: `target prune` cleans old evidence files.

2. **Write amplification**: Two file writes per command invocation instead of one. For high-frequency automated pipelines, this is measurable I/O overhead. Mitigation: Both writes are small (<5KB) and buffered; impact is negligible on any modern filesystem.

3. **Consistency requirement**: Both files must be kept in sync. If the XES is updated post-adjudication, the JSONL must also be updated. Implementation complexity is slightly higher. Mitigation: `emit_evidence()` handles both writes atomically; `update_oracle_fingerprint()` updates both files in the same transaction.

4. **Format divergence risk**: If JSONL and XES develop different schemas over time (feature added to one but not the other), consumers may encounter inconsistencies. Mitigation: The JSONL schema is defined as a strict projection of XES attributes; no field exists in JSONL that doesn't have an XES counterpart.

---

## Authoritative vs. Companion Summary

The JSONL companion is **never** submitted to the oracle. The following table clarifies which format is used for each purpose:

| Purpose | XES | JSONL |
|---------|-----|-------|
| Oracle adjudication (`wpm audit`) | ✓ Authoritative | ✗ Not submitted |
| ProM/Disco process mining | ✓ Direct import | ✗ Conversion needed |
| wpm receipt doctor | ✓ XES-backed | ✗ Not submitted |
| Regulatory audit filing | ✓ ISO standard | ✗ Non-standard |
| Real-time Kafka streaming | ✗ XML overhead | ✓ Preferred |
| `jq`/`mlr` CLI analysis | ✗ Requires xq | ✓ Native |
| Elasticsearch/Loki ingest | ✗ Converter needed | ✓ Native |
| Phase 3 ML training | ✗ Parser needed | ✓ Native |
| Long-term archival | ✓ ISO standard | ✓ Companion |

---

## References

- ADR-011: XES v2 Format (authoritative format decision)
- NDJSON specification: https://ndjson.org/
- Kafka JSONL producer pattern: https://kafka.apache.org/documentation/
- Elasticsearch bulk API: https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-bulk.html
- `jq` manual: https://jqlang.github.io/jq/manual/

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial draft |
