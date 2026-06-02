# cargo cicd trybuild changed

Run only the trybuild fixtures that changed since the base branch.

## Usage

```bash
cargo cicd trybuild changed
```

## How it works

1. Runs `git diff` against the base branch to find changed files under `tests/ui/` (or
   the configured fixture directory).
2. Selects the corresponding trybuild fixtures — both the `.rs` source and any paired
   `.stderr` snapshot file.
3. Runs trybuild against only the selected fixtures.

This avoids running the full fixture estate (which can involve hundreds of separate
`rustc` invocations) when only a small number of fixtures changed.

## Configuration

In `cicd.toml`:

```toml
[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"
```

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Whether changed-fixture selection is active |
| `snapshot_mode` | `"changed-only"` | `changed-only`: run only changed fixtures; `all`: run all fixtures (same as full suite) |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All selected fixtures pass |
| 1 | One or more fixtures failed, or fixture selection produced an error |

## Example output

```
changed fixtures: 4
  tests/ui/compile_fail/missing_final_marking.rs
  tests/ui/compile_fail/missing_final_marking.stderr
  tests/ui/compile_pass/lawful_petri_net.rs
  tests/ui/compile_pass/admitted_evidence.rs
running trybuild on 4 fixture(s)
verdict: pass
```

## Notes

- If no fixture files changed, exits 0 with `no changed trybuild fixtures to run`.
- Trybuild compile-fail fixtures require a paired `.stderr` file with the expected
  compiler diagnostic. A fixture that fails for the wrong reason is not a valid receipt.
- For a full fixture run, invoke trybuild directly via `cargo test --test ui_tests`.
- Changed fixture selection does not replace a full fixture run before releasing.

## Related commands

- `cargo cicd test changed` — same approach for unit and integration tests
- `cargo cicd status` — workspace overview including changed trybuild fixture count
