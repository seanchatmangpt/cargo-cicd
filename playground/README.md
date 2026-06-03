# cargo-cicd playground

The playground is a pre-publish proof cell that exercises every public command against a minimal workspace.

## What is here

- `crates/app/` — minimal Rust crate used as a stable target for all commands
- `scenarios/` — TOML test case definitions, one per public command group
- `scripts/` — runnable proof scripts
- `evidence/` — log output written by `run-playground.sh` (gitignored content, `.gitkeep` tracked)
- `expected/` — reference artifacts for comparison (`.gitkeep` tracked)

## Quick start

Build the binary first:

```bash
cargo build
```

Then run all test cases:

```bash
./playground/scripts/run-playground.sh
```

Each command writes a log to `playground/evidence/<name>.log`. The script prints PASS or FAIL per command and exits non-zero if any command fails.

## Individual scripts

| Script | Purpose |
|--------|---------|
| `run-playground.sh` | Run all commands, collect evidence |
| `run-matrix.sh` | Run 9-command matrix in isolated tmpdir workspaces, report PASS/FAIL/BLOCKED |
| `validate-with-wasm4pm.sh` | Check wpm available, run wpm doctor, report PASS/BLOCKED/FAIL verdict |
| `mutate-evidence.sh` | Write malformed evidence, verify wpm refuses it (or BLOCKED if wpm absent) |
| `clean.sh` | Remove generated evidence files |

## Scenarios

| File | Command |
|------|---------|
| `clean-workspace.toml` | `cargo cicd status` |
| `target-pressure.toml` | `cargo cicd target show` + prune dry-run |
| `changed-tests.toml` | `cargo cicd test changed` + trybuild changed |
| `publish.toml` | `cargo cicd publish` |
| `workspace-doctor.toml` | `cargo cicd workspace doctor` |

## Supplying a custom binary

```bash
BINARY=/path/to/cargo-cicd ./playground/scripts/run-playground.sh
```

## Oracle (optional)

If `wpm` is available the validate script will audit the XES evidence produced during a run:

```bash
./playground/scripts/validate-with-wasm4pm.sh
```

Set `WPM_BIN=/path/to/wpm` to override oracle discovery.
