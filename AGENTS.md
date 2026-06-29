# Workspace Rules

- `cargo-cicd = RustAuthoritySubstrate`
- Python and shell scripts should not act as authority logic.
- **LLM lie cannot satisfy q**

# v26.6.27 Definition of Done

`cargo-cicd v26.6.27` is done only when it becomes the first locked project where an LLM agent cannot produce standing by language, shortcut, fake receipt, direct cargo invocation, placeholder implementation, or unobserved command execution.

The enforceable goal is:

[
LLM\ lie\ cannot\ satisfy\ q
]

[
LLM\ cheat\ cannot\ produce\ R_B
]

[
LLM\ shortcut\ becomes\ CounterexampleSet \neq \varnothing
]

# Agent Lockdown Law

Agent -> cargo cicd -> just -> execution -> trace -> receipt -> OCEL -> gate

Agents interact only with `cargo cicd`.
Humans may use `just`.
`cargo-cicd` may call `just` internally.
Agents may not call `just` directly.
Agents may not call raw `cargo` directly.

## 1. Agent command boundary

Agents may only use:

```text
cargo cicd *
```

Agents must not directly call:

```text
cargo *
just *
bash *
sh *
python *
make *
./scripts/*
target/debug/cargo-cicd *
cargo run --bin cargo-cicd *
```

Formal law:

[
AgentAllowed(cmd)=1 \iff cmd[0..2]=cargo\ cicd
]

[
AgentAllowed(cmd)=0 \Rightarrow BlockBeforeExecution
]

`just` is internal only:

[
Human \to just
]

[
Agent \not\to just
]

[
cargo\text{-}cicd \to just
]

## 2. Research mode vs locked mode

Research mode may inspect and report.

Research mode must not mutate.

Locked mode may mutate only through declared `cargo cicd` nouns/verbs.

Formal law:

[
ResearchMode=1 \Rightarrow FilesChanged=\varnothing
]

[
MutationAllowed=1 \iff Command \in cargo\ cicd\ *
\land TaskMode=Mutating
]

## 3. No raw cargo

Agents must not run:

```text
cargo test
cargo check
cargo clippy
cargo build
cargo fmt
cargo publish
cargo run
cargo metadata
```

Replacement commands must use `cargo cicd`.

Required replacements:

```text
cargo test
=> cargo cicd trace profile --repo . --profile test --json

cargo clippy
=> cargo cicd trace profile --repo . --profile clippy --json

cargo publish --dry-run
=> cargo cicd trace profile --repo . --profile dry-run --json

cargo check
=> cargo cicd trace profile --repo . --profile check --json
```

Actual publish remains forbidden:

[
cargo\ publish \notin \mu_{allowed}
]

## 4. Canonical command shapes

AGENTS.md must declare exactly one canonical command shape for each surface.

Use:

```text
cargo cicd doctor repo --repo . --json
cargo cicd gate repo --repo . --json
cargo cicd verify repo --repo . --json
cargo cicd trace profile --repo . --profile test --json
cargo cicd trace profile --repo . --profile check --json
cargo cicd trace profile --repo . --profile clippy --json
cargo cicd trace profile --repo . --profile dry-run --json
cargo cicd hooks pre-tool-use --repo . --json
cargo cicd hooks install --repo . --provider antigravity --json
cargo cicd hooks uninstall --repo . --provider antigravity --json
cargo cicd ocel replay --repo . --json
cargo cicd receipt verify --repo . --json
```

Do not introduce alternate shapes like:

```text
cargo cicd trace run
cargo cicd trace profile --repo . test
cargo run --bin cargo-cicd -- ...
target/debug/cargo-cicd ...
just test
just gate
```

unless they are explicitly labeled non-agent developer debugging surfaces.

## 5. Hook response is not proof

The following is not sufficient:

```json
{"allow_tool": false}
```

Formal law:

[
HookResponse \neq BlockSemantics
]

PreToolUse standing requires evidence that the prohibited command did not execute:

[
PreToolUseBlocks(cmd)=1
\iff
AttemptSeen(cmd)
\land
DenyPayloadVisible
\land
ExecutionAbsent(cmd)
]

## 6. Trace/receipt/OCEL required

Any command that executes a profile must produce:

```text
ExecutionTrace
Receipt
OCEL event
Gate-visible packet
```

Formal law:

[
cargo\ cicd\ trace\ profile
\Rightarrow
ExecutionTrace
\land Receipt
\land OCEL
]

Indirection is not authority:

[
Command::new("just") \neq Receipt
]

[
just\ succeeded \neq Standing
]

## 7. Placeholders forbidden as authority

Placeholder commands may exist only if visibly marked as incomplete and must force `q=0`.

Examples:

```text
ocel replay placeholder
gate placeholder
receipt placeholder
doctor placeholder
```

Formal law:

[
Placeholder(x)=1 \Rightarrow q_x=0
]

## 8. Compilation is not standing

Do not claim implementation standing from:

```text
cargo check
cargo build
cargo test --no-run
compiles
no compile errors
```

Formal law:

[
Compiles=1 \not\Rightarrow Done=1
]

[
Compiles=1 \not\Rightarrow q=1
]

## 9. No prose completion claims

Agents must not say:

```text
fully implemented
completed
done
green
sealed
robustly modeling
exclusive authorized gateway
```

unless copying a CLI-derived packet that proves the condition.

Allowed final reporting form:

```text
CLI={cargo cicd gate repo --repo . --json}
STDOUT=<raw JSON>
FilesChanged=<git-derived>
CounterexampleSet=<CLI-derived or explicitly labeled observed>
```

## 10. Required barrier counterexample set

AGENTS.md must define:

```text
C_barrier = {
  research_allowlist_present_in_locked_mode,
  antigravity_block_semantics_unproven,
  trace_profile_command_shape_inconsistent,
  cargo_subcommand_path_unverified,
  ocel_replay_placeholder,
  gate_without_trace_receipt,
  verify_without_trace_receipt,
  just_called_without_receipt,
  raw_cargo_used_by_agent,
  just_called_by_agent,
  shell_called_by_agent,
  python_called_by_agent,
  prose_completion_claim,
  compilation_treated_as_standing,
  receipt_without_execution_trace,
  manual_receipt_json,
  placeholder_authority,
  fake_test,
  dummy_gate,
  token_gate,
  hardcoded_commitment
}
```

A barrier cannot be treated as standing while this set is non-empty.
