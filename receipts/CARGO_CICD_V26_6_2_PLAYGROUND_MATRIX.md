# Playground Matrix — cargo-cicd v26.6.2

**date:** 2026-06-03
**run:** playground/scripts/run-playground.sh

## Results

| Command | Exit | Result |
|---------|------|--------|
| `cargo-cicd status` | 0 | PASS |
| `cargo-cicd target show` | 0 | PASS |
| `cargo-cicd target prune --dry-run` | 1 | FAIL — `--dry-run` flag not implemented |
| `cargo-cicd test --changed` | 0 | PASS |
| `cargo-cicd trybuild --changed` | 0 | PASS |
| `cargo-cicd git status` | 0 | PASS |
| `cargo-cicd publish` | 0 | PASS |
| `cargo-cicd workspace doctor` | 0 | PASS |

**Total: 7 passed, 1 failed**

## Known Gap

`target prune --dry-run` is not yet implemented in the CLI argument parser.
The `prune` subcommand exists but does not accept `--dry-run`.
This is a known gap, not a regression.
