# cargo cicd test changed

Run only the tests relevant to files changed since the base branch.

## Usage

```bash
cargo cicd test changed
```

## How it works

1. Runs `git diff` against the base branch configured in `cicd.toml` (default: `origin/main`).
2. Classifies changed Rust source files by crate membership.
3. For each affected crate, runs `cargo test` scoped to that crate.
4. If exact test selection is not possible (e.g. a changed file is a shared module), emits
   a conservative plan — run all tests for the affected crate — and explains the reason.

## Configuration

In `cicd.toml`:

```toml
[test.changed]
enabled = true
base = "origin/main"
```

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `true` | Whether changed-test selection is active |
| `base` | `"origin/main"` | Git ref used as the diff base |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All selected tests pass |
| 1 | One or more tests failed, or test selection produced an error |

## Example output

```
changed files: 3
affected crates: my-crate, my-other-crate
running cargo test -p my-crate
running cargo test -p my-other-crate
verdict: pass
```

## Notes

- If no Rust files changed, exits 0 with `no changed tests to run`.
- Changed test selection does not replace a full `cargo test` in CI. Use it locally to
  shorten the dev loop; run the full suite before pushing.
- For trybuild fixture selection, use `cargo cicd trybuild changed`.

## Related commands

- `cargo cicd trybuild changed` — same approach for compile-fail / compile-pass fixtures
- `cargo cicd status` — workspace overview including changed-file count
