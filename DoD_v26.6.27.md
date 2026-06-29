# cargo-cicd v26.6.27 Definition of Done

## Mission

`cargo-cicd v26.6.27` is done only when it becomes the first locked project where an LLM agent cannot produce standing by language, shortcut, fake receipt, direct cargo invocation, placeholder implementation, or unobserved command execution.

The goal is not:

[
LLM\ cannot\ lie
]

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

---

## 0. Release Law

[
V_{cargo-cicd,26.6.27}=1
\iff
q_{cargo-cicd}=1
]

[
q_{cargo-cicd}=1
\iff
CounterexampleSet_{cargo-cicd}=\varnothing
]

[
M_{LLM}\not\vdash q
]

[
M_{LLM}\not\vdash Done
]

[
M_{LLM}\not\vdash Receipt
]

[
M_{LLM}\not\vdash OCEL
]

[
M_{LLM}\not\vdash Gate
]

Only Rust-owned `cargo cicd ...` commands may compute standing.

---

## 1. Core Boundary

The required execution chain is:

```text
Agent -> cargo cicd -> just -> execution -> trace -> receipt -> OCEL -> gate
```

Formal law:

[
AgentAllowed(cmd)=1 \iff cmd[0..2]=cargo\ cicd
]

[
AgentAllowed(cmd)=0 \Rightarrow BlockBeforeExecution
]

Human developers may use:

```text
just *
```

Agents may not.

[
Human \to just
]

[
Agent \not\to just
]

[
cargo\text{-}cicd \to just
]

---

## 2. Banned Agent Commands

Agents must not directly run:

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

Raw cargo is forbidden:

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

Replacement commands:

```text
cargo test
=> cargo cicd trace profile --repo . --profile test --json

cargo check
=> cargo cicd trace profile --repo . --profile check --json

cargo clippy
=> cargo cicd trace profile --repo . --profile clippy --json

cargo publish --dry-run
=> cargo cicd trace profile --repo . --profile dry-run --json
```

Actual publish is never allowed:

[
cargo\ publish\notin\mu_{allowed}
]

---

## 3. Canonical Agent-Facing CLI

`v26.6.27` must expose exactly these agent-facing command shapes:

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

Non-agent developer debugging surfaces must not appear in AGENTS.md as valid agent commands.

Forbidden canonical alternatives:

```text
cargo cicd trace run
cargo cicd trace profile --repo . test
cargo run --bin cargo-cicd -- ...
target/debug/cargo-cicd ...
just test
just gate
```

---

## 4. PreToolUse Barrier DoD

[
DoD(PreToolUse)=1
\iff
RawCargoBlocked
\land
RawJustBlocked
\land
RawShellBlocked
\land
RawPythonBlocked
\land
CargoCicdAllowed
\land
ExecutionAbsentWhenDenied
]

Hook output alone is insufficient:

[
HookResponse \neq BlockProof
]

Required proof:

[
PreToolUseBlocks(cmd)=1
\iff
AttemptSeen(cmd)
\land
DenyPayloadVisible(cmd)
\land
ExecutionAbsent(cmd)
]

Required blocked fixtures:

```text
cargo test
cargo check
cargo clippy
cargo build
cargo run
cargo publish --dry-run
just test
just gate
bash scripts/check-all.sh
sh scripts/check-all.sh
python generate_pdf.py
make gate
```

Required allowed fixtures:

```text
cargo cicd doctor repo --repo . --json
cargo cicd gate repo --repo . --json
cargo cicd trace profile --repo . --profile test --json
cargo cicd hooks pre-tool-use --repo . --json
```

In locked mode, read-only commands are not agent-allowed unless routed through `cargo cicd`.

[
LockedMode=1 \Rightarrow Allow={cargo\ cicd\ *}
]

---

## 5. Trace Profile DoD

[
DoD(TraceProfile)=1
\iff
ProfileResolved
\land
JustRecipeResolved
\land
ExecutionCaptured
\land
ExitCodeCaptured
\land
StdoutDigestCaptured
\land
StderrDigestCaptured
\land
GitBeforeCaptured
\land
GitAfterCaptured
\land
ReceiptMinted
\land
OCELWritten
]

`Command::new("just")` is not enough.

[
Command::new("just") \neq Receipt
]

[
just\ success \neq Standing
]

Required trace profile output includes:

```json
{
  "schema": "cargo-cicd.trace.v1",
  "repo": ".",
  "profile": "test",
  "recipe": "test",
  "command": ["just", "test"],
  "exit_code": 0,
  "stdout_digest": "...",
  "stderr_digest": "...",
  "git_before": "...",
  "git_after": "...",
  "receipt_path": "...",
  "ocel_event_id": "...",
  "q": 0
}
```

---

## 6. Receipt DoD

[
DoD(Receipt)=1
\iff
Receipt \leftarrow ExecutionTrace
]

Not:

[
Receipt \leftarrow Intent
]

Not:

[
Receipt \leftarrow Token
]

Not:

[
Receipt \leftarrow HardcodedString
]

Not:

[
Receipt \leftarrow ManualJson
]

Not:

[
Receipt \leftarrow LLMText
]

Valid receipt:

[
ReceiptValid(r)=1
\iff
\exists e\in ExecutionTrace:
r=H(e.command,e.exit,e.stdout,e.stderr,e.git_{before},e.git_{after},e.artifacts)
]

Required receipt fields:

```json
{
  "schema": "cargo-cicd.receipt.v1",
  "command": [],
  "exit_code": 0,
  "stdout_digest": "",
  "stderr_digest": "",
  "git_before": "",
  "git_after": "",
  "input_artifacts": {},
  "output_artifacts": {},
  "timestamp": "",
  "receipt_digest": ""
}
```

Counterexamples:

```text
manual_receipt_json
receipt_missing_command
receipt_missing_exit_code
receipt_missing_stdout_digest
receipt_missing_stderr_digest
receipt_missing_git_before
receipt_missing_git_after
receipt_hashes_itself
receipt_from_token
receipt_from_intent
receipt_from_hardcoded_string
```

---

## 7. OCEL DoD

[
DoD(OCEL)=1
\iff
AppendOnlyEventLog
\land
HookEventsRecorded
\land
TraceEventsRecorded
\land
ReceiptEventsRecorded
\land
GateEventsRecorded
\land
ReplayWorks
]

`ocel replay` must not be a placeholder.

[
Placeholder(ocel\ replay)=1 \Rightarrow q_{OCEL}=0
]

Required event chain:

```text
PreToolUseAttempt
PolicyDecision
TraceProfileRequested
JustRecipeResolved
CommandStarted
CommandExited
ReceiptMinted
GateComputed
```

Required OCEL path:

```text
.cargo-cicd/ocel/events.jsonl
```

Every event must contain:

```text
event_id
event_type
timestamp
objects
git_delta
prev_hash
event_hash
```

Only the first event may use genesis.

[
event_i.prev_hash=event_{i-1}.event_hash
]

---

## 8. Doctor DoD

Doctor is the fast fraud scanner.

[
DoD(Doctor)=1
\iff
DoctorOutputsJSON
\land
DoctorChecksAgentCommandBoundary
\land
DoctorChecksPythonShellAuthority
\land
DoctorChecksManualReceipts
\land
DoctorChecksPlaceholderAuthority
\land
DoctorChecksFakeTests
\land
DoctorChecksRawCargoHistory
]

Required command:

```text
cargo cicd doctor repo --repo . --json
```

Doctor must fail if it finds:

```text
*.py used as authority
*.sh used as authority
manual receipt writes
assert!(true) tests
dummy gates
token-string gates
hardcoded commitments
placeholder gate
placeholder ocel replay
raw cargo command in OCEL history
agent just command in OCEL history
```

Required formula:

[
q_{doctor}=1
\iff
CounterexampleSet_{doctor}=\varnothing
]

---

## 9. Gate DoD

[
DoD(Gate)=1
\iff
GateComputedByRust
\land
GateUsesDetectors
\land
GateUsesReceipts
\land
GateUsesOCEL
\land
GateOutputsJSON
\land
GateDoesNotUseLLMText
]

Required command:

```text
cargo cicd gate repo --repo . --json
```

Required output:

```json
{
  "schema": "cargo-cicd.gate.v1",
  "release": "v26.6.27",
  "q_release": 0,
  "failset_cardinality": 0,
  "counterexamples": [],
  "components": {}
}
```

Formula:

[
q_{release}=1
\iff
failset_cardinality=0
\land
counterexamples=\varnothing
]

---

## 10. Playground Cheat Corpus DoD

Before judging any external repo, `cargo-cicd` must catch its own known cheats.

Required fixtures:

```text
playground/raw-cargo
playground/raw-just
playground/synthetic-receipts
playground/token-gates
playground/fake-tests
playground/dummy-gates
playground/hardcoded-commitments
playground/ocel-placeholder
playground/closure-prose
playground/manual-receipts
```

For each fixture:

[
Expected(q)=0
]

[
ExpectedCounterexample \in CounterexampleSet
]

If a seeded cheat passes:

[
q_{cargo-cicd}=0
]

---

## 11. No Prose Standing

Agents must not claim:

```text
fully implemented
completed
done
green
sealed
robust
exclusive authorized gateway
successfully implemented
all code complies
barrier is now fully implemented
```

unless quoting CLI-derived JSON that proves it.

Allowed reporting form:

```text
CLI={cargo cicd gate repo --repo . --json}
STDOUT=<raw JSON>
FilesChanged=<git-derived>
CounterexampleSet=<CLI-derived>
```

Forbidden:

```text
The barrier is fully implemented.
All agents completed successfully.
The project is robustly modeling RustAuthoritySubstrate.
Everything compiles, so it is done.
```

Formal law:

[
ProseClaim \not\vdash q
]

[
Compilation \not\vdash q
]

[
AgentSummary \not\vdash q
]

---

## 12. Research Mode DoD

Research mode may inspect.

Research mode may not mutate.

[
ResearchMode=1 \Rightarrow FilesChanged=\varnothing
]

If files change during research:

[
mutates_during_research \in CounterexampleSet
]

Research output must include:

```text
CommandsRun
FilesChanged
Unknowns
OpenQuestions
NoCodeWritten=1
```

---

## 13. Locked Mode DoD

Locked mode may mutate only through `cargo cicd`.

[
LockedMode=1
\Rightarrow
AgentCommand \in cargo\ cicd\ *
]

If an agent uses raw cargo, just, bash, sh, python, make, or scripts:

[
q_{locked}=0
]

---

## 14. Required Counterexample Set

`cargo-cicd v26.6.27` must define and detect:

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

Release cannot stand while:

[
C_{barrier}\neq\varnothing
]

---

## 15. Acceptance Commands

The release candidate must be evaluated only through:

```text
cargo cicd doctor repo --repo . --json
cargo cicd gate repo --repo . --json
cargo cicd verify repo --repo . --json
cargo cicd hooks pre-tool-use --repo . --json
cargo cicd trace profile --repo . --profile test --json
cargo cicd trace profile --repo . --profile check --json
cargo cicd trace profile --repo . --profile clippy --json
cargo cicd trace profile --repo . --profile dry-run --json
cargo cicd ocel replay --repo . --json
cargo cicd receipt verify --repo . --json
```

No raw cargo acceptance commands.

No just acceptance commands.

No shell acceptance commands.

---

## 16. Final DoD Equation

[
V_{cargo-cicd,26.6.27}=1
\iff
AgentBoundary=1
\land
PreToolUse=1
\land
TraceProfile=1
\land
Receipt=1
\land
OCEL=1
\land
Doctor=1
\land
Gate=1
\land
PlaygroundCheatsDetected=1
\land
NoProseStanding=1
]
