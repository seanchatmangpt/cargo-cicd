# Code Provenance

Code provenance tracks the authorship origin of code committed to a repository:
human-authored, AI-assisted, or AI-generated. For Vision 2030 process conformance,
cargo-cicd embeds provenance metadata in process evidence (XES traces) so the
wasm4pm oracle can apply provenance-aware adjudication policies.

---

## Why Provenance Matters

Provenance enables:

- **Process transparency** — reviewers know whether a change was human-written,
  AI-suggested, or AI-generated before they approve it.
- **Audit trails** — oracle receipts record the provenance tag so post-hoc
  investigations can distinguish human judgment from machine output.
- **Policy enforcement** — the wasm4pm oracle can require additional review gates
  for AI-generated code, or refuse to adjudicate evidence that lacks a declared
  provenance tag.

---

## Declaring Provenance

Set the `CICD_CODE_PROVENANCE` environment variable before running cargo-cicd:

```sh
# Entirely human-authored code
export CICD_CODE_PROVENANCE=human

# Human-led with AI suggestions (AI made proposals, human reviewed and accepted)
export CICD_CODE_PROVENANCE=ai-assisted:copilot
export CICD_CODE_PROVENANCE=ai-assisted:claude
export CICD_CODE_PROVENANCE=ai-assisted:codeium

# AI-generated, human reviewed and approved the output
export CICD_CODE_PROVENANCE=ai-generated:claude
export CICD_CODE_PROVENANCE=ai-generated:gpt-4
export CICD_CODE_PROVENANCE=ai-generated:gemini
```

If `CICD_CODE_PROVENANCE` is not set, cargo-cicd uses heuristic detection to
infer likely provenance from the source files in the current git diff (see
Heuristic Detection below).

---

## Provenance Classes

| Class | Tag format | Meaning |
|-------|-----------|---------|
| `Human` | `human` | Entirely human-authored. No AI tooling involved. |
| `AiAssisted` | `ai-assisted:<tool>` | Human-led; AI made suggestions, human reviewed. |
| `AiGenerated` | `ai-generated:<tool>` | AI-led; AI wrote most of it, human approved. |
| `Unknown` | `unknown` | Provenance not declared and could not be inferred. |

---

## Heuristic Detection

When `CICD_CODE_PROVENANCE` is not set, cargo-cicd scans the changed Rust source
files for patterns commonly produced by large language models. This is a heuristic,
not a deterministic classifier.

### Signals Checked

| Pattern | Confidence weight |
|---------|------------------|
| `// This function` | 0.10 |
| `// The following` | 0.10 |
| `// Implementation of` | 0.15 |
| `unwrap_or_else(|e| panic!` | 0.20 |
| `// TODO: implement` | 0.05 |
| `// Note:` | 0.05 |
| `// Helper function` | 0.10 |
| `/// # Examples` | 0.05 |
| `/// # Panics` | 0.05 |
| `/// # Errors` | 0.05 |
| `// SAFETY:` | 0.05 |
| `let result = ` | 0.05 |
| `// Handle the case where` | 0.10 |
| `// This is because` | 0.10 |

Confidence scores are summed across all matched lines and clamped to `[0.0, 1.0]`.

### Inference Rules

| Average confidence across files | Inferred tag |
|---------------------------------|--------------|
| ≥ 0.5 | `ai-generated:unknown` |
| ≥ 0.2 and < 0.5 | `ai-assisted:unknown` |
| < 0.2 | `human` |

---

## Limitations and False Positive Rates

Heuristic detection is intentionally conservative and will generate false positives:

- **False positives (human code tagged as AI):** Well-documented human code that
  follows conventional Rust documentation patterns (e.g. `/// # Examples`,
  `/// # Errors`) will accumulate signal weight. The threshold is set at 0.2 to
  tolerate moderate documentation density before tagging as AI-assisted.

- **False negatives (AI code tagged as human):** Minimally-commented AI output
  or AI code that has been significantly refactored by a human will score near zero.

- **Tool name accuracy:** The `tool` field in the tag comes from `CICD_CODE_PROVENANCE`.
  Heuristic detection cannot identify which AI tool produced the code and always
  uses `"unknown"` as the tool name.

The expected false positive rate for clean human code following standard Rust idioms
is less than 10%. Code with dense API documentation may score higher.

**Use explicit declaration** (`CICD_CODE_PROVENANCE=human`) to override the
heuristic when the inferred tag is incorrect.

---

## Human Review Gates for AI-Generated Code

Best practices for teams using AI code generation:

1. **Always set `CICD_CODE_PROVENANCE`** in your CI environment. Do not rely
   on heuristic detection for production evidence gates.

2. **Human review is required** before committing `ai-generated` code. The
   `AiGenerated` tag signals to reviewers that additional scrutiny is warranted.

3. **Tests for AI-generated code** should be written or reviewed by a human.
   AI-generated tests may miss edge cases or assert incorrect behavior.

4. **Never use `ai-generated` for security-critical paths** without independent
   human verification. The oracle may flag `ai-generated` receipts as requiring
   additional review.

---

## XES Trace Attribute Embedding

Provenance is embedded in XES evidence traces as string attributes on each
`<trace>` element:

```xml
<trace>
  <string key="concept:name" value="status_show_phase"/>
  <string key="code_provenance:tag" value="ai-generated"/>
  <string key="code_provenance:tool" value="claude"/>
  <string key="code_provenance:files_scanned" value="12"/>
  <string key="code_provenance:likely_llm_files" value="8"/>
  <string key="code_provenance:avg_confidence" value="0.63"/>
  <event>
    ...
  </event>
</trace>
```

The `code_provenance:tag` attribute is the primary field read by the wasm4pm oracle.

---

## API Reference

### `CodeProvenance` enum

```rust
pub enum CodeProvenance {
    Human,
    AiAssisted { tool: String },
    AiGenerated { tool: String },
    Unknown,
}
```

### Key functions

| Function | Purpose |
|----------|---------|
| `detect_llm_patterns(source: &str)` | Scan a Rust source string for LLM heuristic signals |
| `summarize_provenance(file_paths: &[String])` | Aggregate provenance across multiple files |
| `CodeProvenance::from_tag(tag: &str)` | Parse `CICD_CODE_PROVENANCE` env var value |
| `CodeProvenance::to_tag(&self)` | Serialize to XES attribute value |

### `ProvenanceSummary` struct

```rust
pub struct ProvenanceSummary {
    pub tag: String,             // Combined tag for the overall submission
    pub files_scanned: usize,    // Number of files analysed
    pub likely_llm_files: usize, // Files with confidence > 0.5
    pub avg_confidence: f32,     // Average LLM confidence (0.0–1.0)
}
```

---

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `CICD_CODE_PROVENANCE` | Declare code provenance (overrides heuristic) | `ai-generated:claude` |

---

## Example: CI Pipeline Configuration

```yaml
# GitHub Actions example
env:
  # Declare all code in this pipeline as AI-assisted via Claude Code
  CICD_CODE_PROVENANCE: "ai-assisted:claude"

steps:
  - name: Run cargo-cicd evidence gate
    run: cargo cicd evidence audit
```

```sh
# Local development example — human-authored
CICD_CODE_PROVENANCE=human cargo cicd evidence audit

# Local development example — AI pair-programming session
CICD_CODE_PROVENANCE=ai-assisted:claude cargo cicd evidence audit
```
