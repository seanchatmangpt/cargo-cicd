# Custom Subagent Design Index

**Version:** 1.0  
**Last Updated:** 2026-06-14  
**Author:** Anthropic Claude Code

---

## Overview

This document provides a unified index of 5 custom subagents designed for cargo-cicd task automation. Each agent specializes in a specific domain within the cargo-cicd architecture and workflow. Together, they form a complete system for development, testing, policy validation, and release certification.

**Location**: `/home/user/cargo-cicd/AGENT_DESIGN_*.md`

---

## The 5 Agents

### 1. **cargo-cicd-guide** — Architecture & Configuration Reference
**File**: `AGENT_DESIGN_cargo-cicd-guide.md`

**Purpose**: Answer questions about cargo-cicd architecture, configuration, and usage.

**Specialization**:
- Architecture questions (EngineState, adapters, policies, state model)
- Configuration guidance (cicd.toml, feature flags, autonomic policies)
- CLI usage (noun-verb structure, command patterns)
- Troubleshooting and debugging
- Test patterns and integration guides

**Tools**: Read, Grep, Glob, Bash (git commands)

**Primary Use Cases**:
- "How does EngineState work?"
- "What are feature flags?"
- "How do I configure autonomic policies?"
- "Why is my workspace showing dirty?"
- "How do I write a new integration test?"

---

### 2. **test-scaffold-generator** — Test Code & Fixture Creation
**File**: `AGENT_DESIGN_test-scaffold-generator.md`

**Purpose**: Generate test fixtures, test code, and test infrastructure.

**Specialization**:
- FixtureWorkspace variants (create realistic test scenarios)
- Integration test scaffolding (assert_cmd + tempfile patterns)
- Smoke tests (CLI parsing, schema validation)
- Evidence-gate test structures (XES emission, wpm adjudication)
- Invariant checks (public boundary validation)
- Mock data generation (Cargo.toml, cicd.toml, etc.)
- Test utilities and assertion patterns

**Tools**: Write, Edit, Read, Glob, Grep

**Primary Use Cases**:
- "Generate a fixture for a workspace with 50GB target directory"
- "Create an integration test for this adapter"
- "Generate an invariant check for a new command"
- "Create an evidence-gate test scaffold"
- "Write a helper function for common test patterns"

---

### 3. **adapter-builder** — External Source Integration Guidance
**File**: `AGENT_DESIGN_adapter-builder.md`

**Purpose**: Guide creation of new adapters for external sources.

**Specialization**:
- Adapter design and architecture
- State dimension creation (new *State structs)
- Adapter scaffolding (trait implementation, error handling)
- Integration into EngineState startup pipeline
- Error recovery strategies
- Testing approach for adapters
- Schema updates (cicd.toml)
- Common adapter patterns (command parsing, file walks, JSON parsing, multi-source synthesis)

**Tools**: Read, Write, Edit, Glob, Grep

**Primary Use Cases**:
- "Create an adapter for rustup toolchain detection"
- "Build an adapter for cargo tree command output"
- "Design a new state dimension and its adapter"
- "Walk me through integrating this adapter"
- "How should my adapter handle missing tools?"

---

### 4. **policy-auditor** — Policy Analysis & Improvement
**File**: `AGENT_DESIGN_policy-auditor.md`

**Purpose**: Analyze autonomic policies for correctness and quality.

**Specialization**:
- Policy code review (correctness, edge cases)
- Verdict correctness analysis (true positives, false positives/negatives)
- Recommendation quality evaluation (specific, actionable, safe)
- Safety audit (Suggest mode, no mutations, no side effects)
- Coverage analysis (what workspace states aren't evaluated)
- Test gap identification
- Performance implications
- Cross-policy interaction analysis

**Tools**: Read, Grep, Glob, Bash (run tests)

**Primary Use Cases**:
- "Review this policy for correctness and edge cases"
- "What workspace conditions aren't covered by policies?"
- "Verify all policies are safe and run in Suggest mode"
- "Are policy recommendations actionable and clear?"
- "Do any policies conflict with each other?"

---

### 5. **wasm4pm-evidence-validator** — Evidence Format & Correctness
**File**: `AGENT_DESIGN_wasm4pm-evidence-validator.md`

**Purpose**: Validate process evidence for wasm4pm adjudication.

**Specialization**:
- XES (XML Event Stream) format validation
- JSONL companion format validation
- Field completeness and format verification (ISO-8601 timestamps, proper enums)
- Trace grouping and event sequencing
- Receipt doctor compliance and signature validation
- wpm oracle compatibility
- Verdict adjudication validation
- Evidence mutation detection
- Batch validation of evidence directories
- XES ↔ JSONL consistency verification

**Tools**: Read, Grep, Glob, Bash (xmllint, jq)

**Primary Use Cases**:
- "Is this XES evidence file valid and complete?"
- "Do these XES and JSONL files match?"
- "Will this evidence pass `wpm receipt doctor --strict`?"
- "Are all events properly adjudicated by the oracle?"
- "Validate all evidence files in this directory"

---

## Agent Interaction Map

```
┌──────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT WORKFLOW                      │
└──────────────────────────────────────────────────────────────┘

NEW FEATURE REQUEST
        ↓
    [cargo-cicd-guide]  ← Understand architecture
        ↓
    Design Phase
        ├─→ [adapter-builder]    ← Need new external source?
        ├─→ [policy-auditor]     ← Need new policy?
        └─→ [test-scaffold-gen]  ← Design test strategy
        ↓
    Implementation Phase
        ├─→ [adapter-builder]    ← Implement adapter
        ├─→ Implement policy/noun/verb
        └─→ [test-scaffold-gen]  ← Generate test scaffolding
        ↓
    Testing Phase
        ├─→ [test-scaffold-gen]  ← Run smoke/integration tests
        ├─→ [policy-auditor]     ← Review policy behavior
        └─→ [cargo-cicd-guide]   ← Debug failures
        ↓
    Release Phase
        ├─→ [test-scaffold-gen]  ← Generate evidence-gate tests
        └─→ [wasm4pm-validator]  ← Validate XES/JSONL/receipts
        ↓
    RELEASE READY
```

---

## Workflow Examples

### Example 1: Add a New Policy
**Scenario**: You want to add a TargetStalenessPolicy that warns when target/ artifacts are old.

**Agent Workflow**:
1. **cargo-cicd-guide**: Ask about PolicyState structure and existing policies
   - Understand PolicyMode, PolicyVerdict, PolicyResult
   - See examples of GitPhaseDirtyPolicy, TargetPressurePolicy
   - Learn how policies integrate into autonomic evaluation

2. **adapter-builder** (optional): Ask if you need to extend TargetState
   - See if existing TargetState has artifact age metadata
   - If not, design new TargetState field and TargetStalenessAdapter

3. **test-scaffold-generator**: Ask for test case scaffolding
   - Generate FixtureWorkspace variant with stale target/
   - Generate policy test patterns
   - Generate integration test for policy evaluation

4. **policy-auditor**: Ask to review your draft policy
   - Check for false positives/negatives
   - Verify recommendation clarity
   - Ensure Suggest mode is used
   - Identify test gaps

5. **test-scaffold-generator**: Generate evidence-gate test scaffold
   - Create test that emits evidence for the new policy
   - Verify XES format and fields

6. **wasm4pm-evidence-validator**: Validate emitted evidence
   - Check XES well-formedness
   - Verify all fields present
   - Confirm wpm oracle compatibility

---

### Example 2: Integrate a New External Tool
**Scenario**: You want to integrate `cargo-audit` output into EngineState.

**Agent Workflow**:
1. **cargo-cicd-guide**: Understand adapter architecture
   - Learn adapter pattern: query source → translate → populate state
   - Study existing adapters (GitStatusAdapter, CargoMetadataAdapter)

2. **adapter-builder**: Design the new adapter
   - Design CargoAuditState dimension
   - Plan error handling for audit failures
   - Outline integration into EngineState startup
   - Provide complete scaffold code

3. **test-scaffold-generator**: Generate adapter tests
   - Create fixtures that trigger different audit conditions
   - Generate integration tests for adapter output
   - Generate smoke tests for state structure

4. **cargo-cicd-guide**: Debug adapter issues (if needed)
   - Trace execution through adapter
   - Verify state population

5. **policy-auditor** (optional): If adding a policy that uses audit state
   - Review policy logic
   - Ensure recommendations are clear

---

### Example 3: Debug a Failing Test
**Scenario**: An integration test is failing; you need to understand why.

**Agent Workflow**:
1. **cargo-cicd-guide**: Understand test structure
   - Ask about test hierarchy (smoke vs. integration vs. evidence-gate)
   - Ask about fixture patterns
   - Ask about invariants and public boundaries

2. **test-scaffold-generator**: Ask for debug help
   - What fixtures are available for this scenario?
   - What assertions should I use?
   - How do I inspect fixture state?

3. **cargo-cicd-guide**: Ask about state flow
   - Trace which adapter populates the failing state
   - Understand what the adapter queries

4. **adapter-builder** (if adapter is at fault):
   - Review adapter logic
   - Suggest error handling improvements

5. **test-scaffold-generator**: Generate additional test cases
   - Create fixture that isolates the issue
   - Generate assertions that expose the root cause

---

### Example 4: Prepare for Release
**Scenario**: You're ready to release v27.0.0 and need to validate all evidence.

**Agent Workflow**:
1. **test-scaffold-generator**: Run evidence-gate tests
   - Generate evidence for all major features
   - Ensure all tests pass locally

2. **wasm4pm-evidence-validator**: Validate all evidence files
   - Check XES format and completeness
   - Verify JSONL consistency
   - Validate receipt signatures
   - Confirm wpm oracle acceptance

3. **policy-auditor** (if new policies):
   - Audit all policies for safety and correctness
   - Verify coverage and recommendations

4. **cargo-cicd-guide**: Final review
   - Confirm no forbidden terms in help text
   - Verify CLI grammar correctness
   - Check commit message format

5. **wasm4pm-evidence-validator**: Final batch validation
   - Validate entire target/cargo-cicd/evidence/ directory
   - Confirm all events adjudicated as "Accept"
   - Sign off on release readiness

---

## Agent Capability Matrix

| Task | cargo-cicd-guide | test-scaffold-gen | adapter-builder | policy-auditor | wasm4pm-validator |
|------|:----:|:----:|:----:|:----:|:----:|
| **Architecture questions** | ✓✓✓ | ✓ | ✓ | ✓ | - |
| **CLI usage** | ✓✓✓ | - | - | - | - |
| **Configuration guidance** | ✓✓✓ | - | ✓ | - | - |
| **Troubleshooting** | ✓✓✓ | ✓ | - | ✓ | - |
| **Test generation** | - | ✓✓✓ | ✓ | ✓ | ✓ |
| **Adapter scaffolding** | - | - | ✓✓✓ | - | - |
| **Policy review** | - | - | - | ✓✓✓ | - |
| **Evidence validation** | - | - | - | - | ✓✓✓ |
| **Code examples** | ✓ | ✓✓✓ | ✓✓✓ | - | - |
| **Error diagnosis** | ✓ | ✓ | ✓ | ✓ | ✓✓✓ |

Legend: ✓✓✓ = Primary specialty, ✓ = Secondary capability, - = Out of scope

---

## Integration with Claude Code & Agent SDK

### On Claude Code Web
Each agent can be invoked as a slash command:
```
/cargo-cicd-guide How does EngineState work?
/test-scaffold-generator Generate a fixture for a workspace with circular dependencies
/adapter-builder Design an adapter for cargo-deny output
/policy-auditor Review GitPhaseDirtyPolicy for correctness
/wasm4pm-evidence-validator Validate target/cargo-cicd/evidence/
```

### With Claude Agent SDK
Each agent can be launched as a subagent with specific parameters:
```rust
Agent {
    description: "Understanding cargo-cicd architecture",
    subagent_type: "cargo-cicd-guide",
    prompt: "Explain how adapters populate EngineState dimensions"
}
```

### Integration with Build & Release Pipelines
- **CI/CD**: test-scaffold-generator generates test code; wasm4pm-validator validates evidence
- **Code Review**: policy-auditor reviews policy PRs; adapter-builder reviews adapter PRs
- **Release Certification**: wasm4pm-validator gates releases on evidence acceptance
- **Onboarding**: cargo-cicd-guide serves as interactive documentation

---

## Key Design Principles

### 1. **Separation of Concerns**
Each agent has a narrow, well-defined scope:
- cargo-cicd-guide: explains
- test-scaffold-generator: generates tests
- adapter-builder: guides adapter creation
- policy-auditor: audits policies
- wasm4pm-evidence-validator: validates evidence

### 2. **Non-Overlapping Responsibilities**
Agents coordinate through clear handoffs:
- cargo-cicd-guide provides context
- adapter-builder uses that context to design
- test-scaffold-generator generates tests for the design
- policy-auditor reviews the implementation
- wasm4pm-evidence-validator validates the final output

### 3. **Tool Specialization**
Each agent uses only tools suited to its task:
- Read-heavy agents (guide, auditor, validator) emphasize Grep and Read
- Code-generation agents (scaffold, builder) emphasize Write and Edit
- All agents use Glob for file discovery

### 4. **Safety-First Design**
All agents respect cargo-cicd's safety constraints:
- Forbidden terms never appear in generated code
- Policies never generate Apply-mode code
- Evidence validation is strict (XES schema, wpm compatibility)
- Tests verify invariants and boundaries

### 5. **Progressive Disclosure**
Agents provide information at multiple levels:
- Quick answers to specific questions
- Detailed explanations with examples
- Complete code scaffolding ready for use
- Step-by-step checklists for integration

---

## Future Extensions

### Potential Additions
1. **ci-cd-integrator** — Guides GitHub Actions workflow setup for cargo-cicd
2. **performance-profiler** — Analyzes adapter/policy performance and suggests optimizations
3. **ontology-editor** — Guides changes to ggen.toml and cargo-cicd.ttl
4. **feature-designer** — Helps design new nouns and verbs for the CLI
5. **dependency-auditor** — Analyzes Cargo.toml and suggests updates

### Expected Growth
- Each agent will accumulate domain knowledge through usage
- Pattern libraries will grow as more examples are generated
- Integration with CI/CD will automate validation workflows
- Evidence-gate validation will become automated release gate

---

## Quick Reference

### When to Use Each Agent

| Question Type | Agent |
|---------------|-------|
| "How do I...?" | cargo-cicd-guide |
| "What should I test?" | test-scaffold-generator |
| "How do I add...?" | adapter-builder (adapters) or cargo-cicd-guide (general) |
| "Is this policy correct?" | policy-auditor |
| "Is this evidence valid?" | wasm4pm-evidence-validator |
| "Why is my code failing?" | cargo-cicd-guide (architecture) → relevant agent (specific) |
| "Generate test code" | test-scaffold-generator |
| "Generate adapter code" | adapter-builder |
| "Review my PR" | policy-auditor (policies) or adapter-builder (adapters) |

---

## Documentation Files

All agent designs are documented in markdown files in the repository root:

```
/home/user/cargo-cicd/
├── AGENT_DESIGN_INDEX.md                      (this file)
├── AGENT_DESIGN_cargo-cicd-guide.md           (architecture reference)
├── AGENT_DESIGN_test-scaffold-generator.md    (test code generation)
├── AGENT_DESIGN_adapter-builder.md            (adapter creation)
├── AGENT_DESIGN_policy-auditor.md             (policy analysis)
└── AGENT_DESIGN_wasm4pm-evidence-validator.md (evidence validation)
```

---

## Implementation Status

**Current Status**: Design phase complete (documentation only)

**Next Steps**:
1. ✓ Design 5 agents (completed)
2. Create agent implementations using Claude Agent SDK
3. Integrate agents into Claude Code CLI
4. Add agents to project hooks (session-start-hook)
5. Build knowledge bases from codebase (CLAUDE.md, source files)
6. Test agents in real workflows
7. Gather feedback and iterate

**Expected Timeline**:
- Implementation: 1-2 sprints per agent
- Integration: 1 sprint
- Testing: 1 sprint
- Launch: ~2 months from implementation start

---

## Contact & Feedback

For questions or feedback on these agent designs:
- Review the individual agent design files
- File issues with specific agent concerns
- Propose new agents or modifications
- Share use cases and workflows

**Maintained by**: Anthropic Claude Code team

