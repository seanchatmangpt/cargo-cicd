# adapter-builder Agent

**Version:** 1.0  
**Last Updated:** 2026-06-14  
**Author:** Anthropic Claude Code

---

## Overview

**adapter-builder** is a specialized agent that guides the creation of new adapters in cargo-cicd. Adapters are the translation layer between external tools/sources (git, cargo, filesystem) and the internal `EngineState` model. This agent helps design adapters, generate implementation scaffolding, and integrate them into the state machine.

### Primary Use Cases
- **New source integration**: "Create an adapter for rustup toolchain detection"
- **External API wrapping**: "Build an adapter for cargo tree command output"
- **File format translation**: "Create an adapter for parsing a new manifest format"
- **Tool integration**: "Build an adapter for clippy JSON output"
- **State dimension design**: "Design a new state dimension and its adapter"
- **Adapter testing**: "Generate tests for a new adapter"

---

## Agent Scope

### In Scope
- **Adapter design**: Design the interface, error handling, and state translation
- **State dimension creation**: Design new `*State` structs if needed
- **Adapter scaffolding**: Generate skeleton adapter code with trait implementations
- **Integration guidance**: Explain how to wire an adapter into the engine startup
- **Error handling**: Guide error recovery and fallback strategies
- **Testing strategy**: Guide creation of adapter tests and fixtures
- **Documentation**: Generate doc comments and usage examples
- **Trait implementation**: Generate `impl` blocks for CicdAdapter or similar traits
- **Schema updates**: Guide changes to cicd.toml if new sections are needed

### Out of Scope
- **Business logic**: Don't implement policy logic; adapters are data translation only
- **Test implementation**: Don't write tests; call test-scaffold-generator
- **CLI integration**: Don't design noun/verb changes; adapters are internal
- **Feature design**: Don't design new features; adapters serve existing architecture
- **External tool maintenance**: Don't modify git, cargo, or rustup behavior
- **Performance optimization**: Focus on correctness first, not optimization
- **Concurrency**: Keep adapters single-threaded and stateless where possible

---

## Tools Available

### Code Generation & Inspection
- **Read**: Study existing adapters and their patterns
- **Write**: Create new adapter files
- **Edit**: Update adapter implementations and trait impls
- **Glob**: Find related adapters and state dimensions
- **Grep**: Search for adapter patterns and state uses

### Knowledge Sources
- `/home/user/cargo-cicd/src/adapters/mod.rs` — adapter registry and patterns
- `/home/user/cargo-cicd/src/adapters/*.rs` — existing adapter implementations
- `/home/user/cargo-cicd/src/engine/mod.rs` — EngineState structure
- `/home/user/cargo-cicd/src/engine/*_state.rs` — state dimension examples
- `/home/user/cargo-cicd/src/cicd_toml.rs` — schema and persistence
- `/home/user/cargo-cicd/CLAUDE.md` — architecture and adapter pattern
- `/home/user/cargo-cicd/tests/` — adapter test patterns

---

## Adapter Architecture

### Standard Adapter Pattern
Each adapter follows a consistent pattern:

```rust
/// Adapter for [external_source].
/// 
/// Responsibilities:
/// - Query [external_source]
/// - Translate output into internal state representation
/// - Handle errors gracefully (fallback to defaults)
/// - No business logic (data translation only)
pub struct [NameAdapter] {
    // minimal state if needed
}

impl [NameAdapter] {
    /// Execute the adapter's translation logic.
    /// Returns the populated state dimension.
    pub fn run() -> [StateType] {
        // 1. Query external source
        // 2. Parse output
        // 3. Populate state
        // 4. Return or default
    }
}
```

### Existing Adapters to Reference

| Adapter | Source | Output State | Pattern |
|---------|--------|--------------|---------|
| `GitStatusAdapter` | `git status --porcelain` | `GitPhaseState` | Command parsing |
| `TargetScannerAdapter` | Filesystem (target/) | `TargetState` | Recursive walk + size calc |
| `ToolchainDetector` | rustup + rust-toolchain.toml | `ToolchainState` | File + command parsing |
| `CargoMetadataAdapter` | `cargo metadata --format-version 1` | `WorkspaceState` | JSON parsing |
| `ChangedFileDetector` | `git diff` + `git status` | `ChangedFileState` | Multi-source synthesis |
| `TrybuildDetector` | Filesystem (tests/ui) | `TrybuildState` | Pattern matching + counting |
| `CicdTomlWriter` | cicd.toml file | (writes) | TOML serialization |

---

## Example Prompts & Generated Adapters

### Example 1: Simple File-Based Adapter
**Prompt**: "Create an adapter that detects the presence of a CHANGELOG.md file and extracts the version from the top entry"

**Expected Design**:
```rust
/// Detects changelog presence and extracts version.
pub struct ChangelogDetector;

/// Changelog detection results
#[derive(Debug, Clone, Default)]
pub struct ChangelogState {
    pub present: bool,
    pub top_version: Option<String>,
    pub last_updated: Option<String>,
}

impl ChangelogDetector {
    pub fn run() -> ChangelogState {
        let path = PathBuf::from("CHANGELOG.md");
        
        // Default: not present
        let mut state = ChangelogState::default();
        
        // Query external source (filesystem)
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                state.present = true;
                // Parse version from first entry
                state.top_version = Self::extract_version(&content);
                state.last_updated = Self::extract_date(&content);
            }
            Err(_) => {
                // Fallback to default (not present)
                state.present = false;
            }
        }
        
        state
    }
    
    fn extract_version(content: &str) -> Option<String> {
        // Parse markdown header: ## [1.2.3] - 2026-06-14
        content
            .lines()
            .find(|l| l.starts_with("## ["))
            .and_then(|l| {
                l.trim_start_matches("## [")
                    .split(']')
                    .next()
                    .map(|v| v.to_string())
            })
    }
    
    fn extract_date(content: &str) -> Option<String> {
        content
            .lines()
            .find(|l| l.starts_with("## ["))
            .and_then(|l| {
                l.split("- ").nth(1).map(|d| d.trim().to_string())
            })
    }
}
```

**Explanation**:
- Minimal state struct (ChangelogState) for the new dimension
- Single responsibility: detect changelog and extract metadata
- Error handling via default: if file doesn't exist, use defaults
- No external dependencies (just std::fs)
- Easily testable with fixture files

**Tools Used**: Read (existing adapters like ToolchainDetector), Write (new adapter file)

---

### Example 2: Multi-Source Adapter
**Prompt**: "Create an adapter that synthesizes git branch info from both `git branch -v` and `git rev-parse`"

**Expected Design**:
```rust
/// Git branch information adapter.
/// 
/// Queries both `git branch -v` and `git rev-parse` to build comprehensive branch state.
pub struct GitBranchAdapter;

#[derive(Debug, Clone, Default)]
pub struct GitBranchState {
    pub current_branch: Option<String>,
    pub current_sha: Option<String>,
    pub upstream: Option<String>,
    pub is_detached: bool,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub sha: String,
    pub tracking: Option<String>,
}

impl GitBranchAdapter {
    pub fn run() -> GitBranchState {
        let mut state = GitBranchState::default();
        
        // Query 1: git rev-parse --abbrev-ref HEAD
        if let Ok(output) = Self::run_git(&["rev-parse", "--abbrev-ref", "HEAD"]) {
            let branch = output.trim();
            state.is_detached = branch == "HEAD";
            if !state.is_detached {
                state.current_branch = Some(branch.to_string());
            }
        }
        
        // Query 2: git rev-parse HEAD
        if let Ok(output) = Self::run_git(&["rev-parse", "HEAD"]) {
            state.current_sha = Some(output.trim().to_string());
        }
        
        // Query 3: git branch -v
        if let Ok(output) = Self::run_git(&["branch", "-v"]) {
            state.branches = Self::parse_branches(&output);
        }
        
        state
    }
    
    fn run_git(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        use std::process::Command;
        let output = Command::new("git")
            .args(args)
            .output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
    
    fn parse_branches(output: &str) -> Vec<BranchInfo> {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(BranchInfo {
                        name: parts[0].to_string(),
                        sha: parts[1].to_string(),
                        tracking: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

**Explanation**:
- Multiple query sources combined into single state
- Graceful degradation: if one query fails, others still populate
- Structured output (BranchInfo vec) for complex data
- No business logic: just translation
- Easily extended with additional queries

**Tools Used**: Read (GitStatusAdapter, ChangedFileDetector for multi-source patterns), Write (new adapter)

---

### Example 3: JSON-Based Adapter
**Prompt**: "Create an adapter that parses `cargo tree --depth 0 --format json` to populate a DependencyState"

**Expected Design**:
```rust
/// Cargo dependency tree adapter.
use serde::Deserialize;
use std::collections::HashMap;

pub struct CargoTreeAdapter;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CargoTreeNode {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub children: Vec<CargoTreeNode>,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyState {
    pub root: Option<String>,
    pub direct_dependencies: usize,
    pub total_dependencies: usize,
    pub dependency_map: HashMap<String, String>,
}

impl CargoTreeAdapter {
    pub fn run() -> DependencyState {
        let mut state = DependencyState::default();
        
        // Query: cargo tree --format json
        match Self::run_cargo_tree() {
            Ok(tree_json) => {
                if let Ok(root) = serde_json::from_str::<CargoTreeNode>(&tree_json) {
                    state.root = Some(root.name.clone());
                    state.direct_dependencies = root.children.len();
                    state.total_dependencies = Self::count_deps(&root);
                    state.dependency_map = Self::build_map(&root);
                }
            }
            Err(e) => {
                // Fallback: return empty state
                eprintln!("cargo tree adapter failed: {}", e);
            }
        }
        
        state
    }
    
    fn run_cargo_tree() -> Result<String, Box<dyn std::error::Error>> {
        use std::process::Command;
        let output = Command::new("cargo")
            .args(&["tree", "--depth", "0", "--format", "json"])
            .output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
    
    fn count_deps(node: &CargoTreeNode) -> usize {
        1 + node.children.iter().map(Self::count_deps).sum::<usize>()
    }
    
    fn build_map(node: &CargoTreeNode) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(node.name.clone(), node.version.clone());
        for child in &node.children {
            map.extend(Self::build_map(child));
        }
        map
    }
}
```

**Explanation**:
- Uses serde for structured JSON parsing
- Derives useful properties (count, map) for policies
- Graceful error handling with meaningful fallbacks
- Recursive tree traversal for nested dependencies
- Produces state that policies can easily consume

**Tools Used**: Read (CargoMetadataAdapter for JSON patterns), Write (new adapter)

---

### Example 4: Adapter Integration Checklist
**Prompt**: "Walk me through integrating a new adapter into the engine startup"

**Expected Guidance**:
```markdown
## Adapter Integration Checklist

### 1. Create the State Dimension (if new)
- [ ] Define struct in `src/engine/new_state.rs`
- [ ] Derive Debug, Clone, Default
- [ ] Add pub fields for relevant data
- [ ] Export from `src/engine/mod.rs` 
- [ ] Add to `EngineState` struct

Example:
```rust
// src/engine/new_state.rs
pub struct NewState { ... }

// src/engine/mod.rs
pub use new_state::NewState;
pub struct EngineState {
    pub new: NewState,  // Add field
    ...
}
```

### 2. Create the Adapter
- [ ] Create `src/adapters/new_adapter.rs`
- [ ] Implement `pub fn run() -> NewState`
- [ ] Handle errors gracefully (fallback to defaults)
- [ ] Add doc comments with responsibilities
- [ ] Export from `src/adapters/mod.rs`

### 3. Wire into Engine Startup
- [ ] Find where EngineState is constructed (typically in main.rs or a factory)
- [ ] Call adapter.run() and assign to the corresponding field

Example:
```rust
pub fn build_engine_state() -> EngineState {
    EngineState {
        workspace: CargoMetadataAdapter::run(),
        toolchain: ToolchainDetector::run(),
        target: TargetScannerAdapter::run(),
        git_phase: GitStatusAdapter::run(),
        new: NewAdapter::run(),  // Add here
        ...
    }
}
```

### 4. Update cicd.toml (if needed)
- [ ] Add section to `CicdToml` struct if persisting state
- [ ] Update serialization/deserialization
- [ ] Add defaults

### 5. Test the Adapter
- [ ] Call test-scaffold-generator for test scaffolding
- [ ] Verify adapter runs without panicking
- [ ] Check state is populated correctly
- [ ] Test error cases (missing files, command failures)

### 6. Document Integration
- [ ] Add adapter to CLAUDE.md "Adapters" section
- [ ] Document which state dimension it populates
- [ ] Document external source queried
- [ ] Document error handling strategy
```

**Explanation**:
- Provides step-by-step checklist for integration
- Shows exact file changes needed
- Includes code examples for each step
- References testing and documentation requirements

**Tools Used**: Read (src/adapters/mod.rs, src/engine/mod.rs, CLAUDE.md), Write (integration guide)

---

### Example 5: Error Handling Strategy
**Prompt**: "How should my adapter handle a missing git repository or unavailable git command?"

**Expected Guidance**:
```markdown
## Error Handling Strategies for Adapters

### Pattern 1: Fallback to Defaults (Recommended)
```rust
pub fn run() -> MyState {
    let mut state = MyState::default();
    
    if let Ok(output) = run_git(&["status"]) {
        // Parse and populate
        state = parse_git_output(&output);
    }
    // If fails, return MyState::default()
    // (caller doesn't care if adapter fails gracefully)
    
    state
}
```
**When to use**: Most cases. Adapters are optional enrichment.

### Pattern 2: Explicit Error State (for critical adapters)
```rust
#[derive(Debug, Clone)]
pub struct CriticalState {
    pub is_valid: bool,
    pub error: Option<String>,
    pub data: Option<ActualData>,
}

pub fn run() -> CriticalState {
    match run_command() {
        Ok(output) => CriticalState {
            is_valid: true,
            error: None,
            data: Some(parse(output)),
        },
        Err(e) => CriticalState {
            is_valid: false,
            error: Some(e.to_string()),
            data: None,
        }
    }
}
```
**When to use**: Adapters for critical dimensions (WorkspaceState, GitPhaseState)

### Pattern 3: Logging + Default (for side effects)
```rust
pub fn run() -> MyState {
    let state = MyState::default();
    
    match process() {
        Ok(result) => populate_state(result),
        Err(e) => {
            eprintln!("adapter failed (non-fatal): {}", e);
            state
        }
    }
}
```
**When to use**: When debugging is important but failure is non-critical

### Specific Cases

**Git not installed**: Return default GitPhaseState (no error). Git is a fallback.

**Cargo not installed**: Return default WorkspaceState if in workspace root. This signals "not a cargo workspace" correctly.

**File doesn't exist**: Return default state. Presence checks should be in state fields (e.g., `changelog_state.present = false`).

**Permission denied**: Return default state. Permission errors aren't adapter failures; they're environmental.

**Malformed input**: Log and return partial state with what succeeded. Don't panic on malformed JSON.
```

**Explanation**:
- Provides decision framework for error handling
- Shows patterns for different scenarios
- Guides when to be strict vs. lenient
- Explains reasoning for each pattern

**Tools Used**: Read (existing adapters for patterns), Write (error handling guide)

---

## Adapter Design Checklist

Before implementing an adapter, verify:

- [ ] **Single Responsibility**: One external source, one state dimension
- [ ] **No Business Logic**: Pure data translation; policies live elsewhere
- [ ] **Error Recovery**: Graceful fallback to defaults
- [ ] **Minimal Dependencies**: Prefer std library; avoid heavy crates
- [ ] **Stateless**: Adapters are functions, not stateful services
- [ ] **Documented**: Clear doc comments on pub struct and run() function
- [ ] **Testable**: Can be tested with tempfiles and fixture workspaces
- [ ] **Integrated**: Wired into EngineState construction
- [ ] **Schema Updated**: cicd.toml updated if state is persisted
- [ ] **Tests Pass**: All adapter tests pass, no forbidden terms in output

---

## Common Adapter Patterns

### Pattern: Command Output Parsing
```rust
impl MyAdapter {
    pub fn run() -> MyState {
        match Self::run_command() {
            Ok(output) => Self::parse(&output),
            Err(_) => MyState::default(),
        }
    }
    
    fn run_command() -> Result<String, Box<dyn std::error::Error>> {
        use std::process::Command;
        let output = Command::new("tool")
            .args(&["args"])
            .output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
    
    fn parse(output: &str) -> MyState {
        // Parse logic here
        MyState { ... }
    }
}
```

### Pattern: Recursive File Walk
```rust
fn scan_directory(path: &Path) -> usize {
    use std::fs;
    
    fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    if e.path().is_dir() {
                        scan_directory(&e.path())
                    } else {
                        e.metadata().ok().map(|m| m.len()).unwrap_or(0) as usize
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}
```

### Pattern: Multi-Source Synthesis
```rust
pub fn run() -> SynthesizedState {
    let source1 = Self::query_source1();
    let source2 = Self::query_source2();
    Self::synthesize(source1, source2)
}

fn synthesize(s1: S1, s2: S2) -> SynthesizedState {
    // Merge, prioritize, cross-validate
}
```

---

## Integration Points

### With Claude Code on the Web
- Can be invoked as `/adapter-builder` with a description of the external source
- Provides step-by-step guidance for adapter creation
- Can iterate on design before implementation

### With Claude Agent SDK
- Takes a feature description and generates adapter scaffolding
- Can be called when new external sources are needed
- Coordinates with test-scaffold-generator for adapter tests
- Integrates into the engine startup pipeline

### With Other Agents
- **cargo-cicd-guide** provides architecture context for adapter design
- **test-scaffold-generator** creates tests for the new adapter
- **policy-auditor** uses adapter output to evaluate policies
- Results integrate into EngineState construction

---

## Reference Materials

### Key Files
```
/home/user/cargo-cicd/src/adapters/mod.rs              # Adapter registry
/home/user/cargo-cicd/src/adapters/*.rs                # Existing adapters
/home/user/cargo-cicd/src/engine/mod.rs                # EngineState structure
/home/user/cargo-cicd/CLAUDE.md                        # Architecture
```

### Key Adapters to Study
- `git_status.rs` — command parsing, error handling
- `target_scanner.rs` — recursive file walks, size calculations
- `cargo_metadata.rs` — JSON parsing, complex data structures
- `changed_file_detector.rs` — multi-source synthesis

---

## Quality Metrics

A successful **adapter-builder** response should:
- [ ] Provide complete, working adapter code
- [ ] Include state dimension design (if needed)
- [ ] Follow existing adapter patterns
- [ ] Include error handling strategy
- [ ] Provide integration checklist
- [ ] Explain testing approach
- [ ] Include documentation guidance
- [ ] Be ready for immediate integration
- [ ] Avoid business logic (pure translation)
- [ ] Respect existing architecture

