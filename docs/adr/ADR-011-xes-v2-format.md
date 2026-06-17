# ADR-011: Adopt XES 2.0 as Canonical Evidence Format

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** evidence, format, process-mining, interoperability

---

## Context

cargo-cicd emits process evidence after every command execution. This evidence is the primary artifact consumed by the wasm4pm oracle for adjudication. As cargo-cicd evolves toward an ecosystem platform under Vision 2030, the evidence format must:

1. **Be interoperable** with external process mining tools (ProM, Disco, Celonis, RapidMiner Process Mining).
2. **Be formally specified** by an international standard so that third-party verifiers can validate evidence without cargo-cicd-specific tooling.
3. **Support trace semantics** — events must be groupable into traces (case instances) to enable conformance checking against declared process models.
4. **Be extensible** to accommodate future event attributes (provenance classification, oracle key fingerprints, AI attribution).
5. **Support long-term archival** — evidence files may need to be readable 10-20 years in the future.

Several candidate formats were evaluated:

### Candidate A: JSON-LD (Linked Data)

JSON-LD uses RDF semantics expressed in JSON syntax. It offers strong interoperability for linked data ecosystems and is used in W3C provenance (PROV-O) contexts.

**Pros:**
- Native support for `@context` references, enabling semantic linking to ontologies.
- Human-readable JSON syntax.
- Strong W3C specification backing.
- PROV-O vocabulary maps naturally to process lifecycle events.

**Cons:**
- No native concept of "trace" or "case": conformance checking requires custom tooling.
- Process mining tools (ProM, Disco) do not natively import JSON-LD — a converter layer would be required.
- JSON-LD context resolution introduces network dependencies for strict validation.
- No ISO standard with an accreditation path.

### Candidate B: OpenTelemetry Traces

OpenTelemetry (OTel) provides a vendor-neutral observability format for distributed tracing. It is widely adopted in cloud-native contexts.

**Pros:**
- Massive ecosystem of exporters (Jaeger, Zipkin, Prometheus, Grafana Tempo).
- Strong tooling for span correlation and latency analysis.
- Native concept of spans and traces maps partially to process events.

**Cons:**
- OTel is an observability format, not a process mining format. Conformance checking is not a first-class concern.
- No support for case-based trace semantics required by process mining conformance checking algorithms (BESeP, token-based replay).
- Oracle adjudication of OTel traces is not part of any existing standard.
- ProM/Disco do not import OTel natively.

### Candidate C: OCEL 2.0 (Object-Centric Event Log)

OCEL 2.0 is a newer extension of XES designed for object-centric process mining, where events can relate to multiple objects simultaneously.

**Pros:**
- Models complex multi-object relationships naturally (e.g., a pipeline run touching multiple crates).
- Growing tool support in academic process mining community.

**Cons:**
- Not yet ISO-standardized (as of 2026).
- ProM/Disco support is partial and experimental.
- Significantly more complex to emit and validate than XES.
- Premature for stable ecosystem adoption.

### Candidate D: Custom JSONL Format

A custom newline-delimited JSON format designed specifically for cargo-cicd's needs.

**Pros:**
- Minimal schema; easy to emit and parse.
- Streaming-friendly (one event per line).
- No parser dependencies.

**Cons:**
- Not interoperable with any existing process mining tool.
- No formal specification; future verifiers must reverse-engineer the schema.
- No accreditation path for regulatory compliance.
- Requires custom tooling for all downstream consumers.

### Candidate E: XES 2.0 (ISO/IEC 20880:2013 + Extensions)

XES (eXtensible Event Stream) is the ISO/IEC 20880:2013 standard for process event logs. It is the lingua franca of process mining research and tooling.

**Pros:**
- ISO/IEC 20880:2013 formal standard — accreditation bodies recognize it.
- Native support in ProM (University of Eindhoven), Disco (Fluxicon), Celonis, and all major academic process mining tools.
- First-class trace/case semantics: events are grouped into `<trace>` elements identified by `case_id`.
- Extensible via `<extension>` elements — custom attributes can be added without breaking parsers.
- Long archival track record (used in process mining research since 2009).
- XES 2.0 draft extensions add streaming support and structured attribute nesting.

**Cons:**
- XML verbosity — XES files are significantly larger than equivalent JSON.
- Requires XML parsing rather than line-by-line JSONL streaming.
- Some streaming pipelines (Kafka consumers, log shippers) prefer JSONL.

---

## Decision

**Adopt XES 2.0 (ISO/IEC 20880:2013) as the canonical evidence format for all cargo-cicd process events.**

The canonical format is XES. A JSONL companion file is emitted alongside every XES file (see ADR-015: JSONL Companion Format) to support streaming parsers, but XES is authoritative for:

- Oracle adjudication (`wpm audit <file.xes>`).
- Conformance checking against declared process models.
- Long-term archival.
- Regulatory compliance filings.
- Import into ProM, Disco, and Celonis.

### Canonical XES Structure for cargo-cicd

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0" xes.features="nested-attributes"
     xmlns="http://www.xes-standard.org/">

  <!-- Standard XES extensions in use -->
  <extension name="Lifecycle" prefix="lifecycle"
             uri="http://www.xes-standard.org/lifecycle.xesext"/>
  <extension name="Time" prefix="time"
             uri="http://www.xes-standard.org/time.xesext"/>
  <extension name="Concept" prefix="concept"
             uri="http://www.xes-standard.org/concept.xesext"/>

  <!-- cargo-cicd custom extension -->
  <extension name="CargoCI" prefix="cargoCI"
             uri="https://cargo-cicd.rs/xes-extensions/v2/cargoCI.xesext"/>

  <!-- Log-level attributes: metadata about this evidence batch -->
  <string key="cargoCI:workspace_id" value="cargo-cicd@/home/user/cargo-cicd"/>
  <string key="cargoCI:schema_version" value="2.0"/>
  <date key="cargoCI:emitted_at" value="2026-06-17T14:00:00.000Z"/>

  <!-- One <trace> per case (e.g., one command invocation) -->
  <trace>
    <string key="concept:name" value="status_show_phase"/>
    <string key="cargoCI:command" value="status show"/>
    <string key="cargoCI:workspace_id" value="cargo-cicd@/home/user/cargo-cicd"/>
    <string key="cargoCI:oracle_key_fingerprint"
            value="SHA256:abc123..."/>  <!-- See ADR-013 -->

    <!-- Start event -->
    <event>
      <string key="concept:name" value="evt-status-show-20260617140000000Z-start"/>
      <string key="lifecycle:transition" value="start"/>
      <date key="time:timestamp" value="2026-06-17T14:00:00.000Z"/>
      <string key="cargoCI:verdict_claimed" value="PASS"/>
      <string key="cargoCI:trace_class" value="live_workspace"/>
    </event>

    <!-- Complete event -->
    <event>
      <string key="concept:name" value="evt-status-show-20260617140001234Z-complete"/>
      <string key="lifecycle:transition" value="complete"/>
      <date key="time:timestamp" value="2026-06-17T14:00:01.234Z"/>
      <string key="cargoCI:verdict_claimed" value="PASS"/>
      <string key="cargoCI:duration_ms" value="1234"/>
      <string key="cargoCI:trace_class" value="live_workspace"/>
    </event>
  </trace>
</log>
```

### Key Structural Decisions Within XES

1. **One file per command invocation**: Each verb execution produces one XES file in `target/cargo-cicd/evidence/`. Batch files (multiple traces) are used only for pipeline runs.

2. **Lifecycle extension**: All events use the XES Lifecycle extension (`start`/`complete` transitions), enabling standard conformance checkers to evaluate the lifecycle model.

3. **Concept extension**: The `concept:name` attribute on traces carries the human-readable command name. This is the standard XES field used by ProM/Disco for activity naming.

4. **cargoCI extension**: A custom extension URI registers cargo-cicd-specific attributes (`verdict_claimed`, `trace_class`, `oracle_key_fingerprint`, etc.) without polluting the standard namespace.

5. **Oracle key fingerprint**: The public key fingerprint of the adjudicating oracle is embedded in the trace (see ADR-013 for rationale).

---

## Consequences

### Positive

1. **Process mining tool compatibility**: Evidence files can be imported directly into ProM, Disco, and Celonis without transformation. This is the primary enabler of the Vision 2030 process mining dashboard (see `docs/process-mining-architecture.md`).

2. **ISO accreditation path**: XES is ISO/IEC 20880:2013. Regulatory frameworks (DO-178C, FDA 21 CFR Part 11) that require auditable process logs can reference the standard. See Phase 3 design (`docs/PHASE-3-DESIGN.md`).

3. **Conformance checking**: ProM's BESeP algorithm, token-based replay, and alignment-based conformance all operate natively on XES. No conversion required.

4. **Long-term archival**: XES is a stable standard. Evidence emitted today should be readable by future tools in 10-20 years with no migration.

5. **Research community adoption**: Process mining researchers publish datasets in XES format. cargo-cicd evidence can be contributed to academic datasets.

6. **Ecosystem confidence**: Third-party verifiers can inspect evidence without cargo-cicd-specific tooling — they only need an XML parser and knowledge of the XES schema.

### Negative

1. **XML verbosity**: XES files are significantly larger than equivalent JSONL. A single command invocation that produces two events (start + complete) generates ~2KB of XML vs ~300 bytes of JSONL. At high frequency (1000+ commands/day), storage costs are meaningful. Mitigation: JSONL companion (ADR-015) is used for real-time streaming; XES is archived.

2. **Streaming parsers**: Most Kafka consumers, Logstash pipelines, and Fluent Bit configurations natively handle JSONL, not XML. Mitigation: JSONL companion is emitted alongside every XES file.

3. **XML parser dependency**: Rust's XML parsing ecosystem (quick-xml, xml-rs) is mature but adds to the dependency tree. Mitigation: Parsing is only needed for conformance checking, not emission. Emission uses a purpose-built serializer.

4. **Custom extension registration**: The `cargoCI` extension URI requires hosting the extension definition at `https://cargo-cicd.rs/xes-extensions/v2/cargoCI.xesext`. This is an ongoing maintenance obligation. Mitigation: The URI is resolvable as a URL but validators may operate in offline mode.

### Neutral

- XES 2.0 streaming extensions are not yet finalized by the XES Standards Committee (as of 2026-06). cargo-cicd uses the stable XES 1.0 structure with 2.0 `xes.features` attributes for forward compatibility.

---

## Alternatives Considered and Rejected

| Format | Rejection Reason |
|--------|-----------------|
| JSON-LD | No process mining tool support; no trace semantics; no ISO standard |
| OpenTelemetry | Observability format, not process mining; no conformance checking |
| OCEL 2.0 | Not ISO-standardized; immature tool support; premature complexity |
| Custom JSONL | No interoperability; no specification; no accreditation path |

---

## References

- ISO/IEC 20880:2013: Information technology — Process mining — Event log
- XES Standard: https://www.xes-standard.org/
- ProM Framework: http://www.promtools.org/
- Disco Process Mining: https://fluxicon.com/disco/
- Van der Aalst, W.M.P. (2016). *Process Mining: Data Science in Action* (2nd ed.). Springer.
- cargo-cicd Evidence Emission: `src/evidence.rs`
- cargo-cicd XES Schema: `docs/wasm4pm/xes-format.md`

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial draft for Phase 1 Weeks 9-12 |
