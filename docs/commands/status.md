# cargo cicd status

Show workspace CI/CD status: toolchain, target size, git state.

## Usage

```bash
cargo cicd status
```

## Output

Reports the following fields:

- **Toolchain** — active Rust toolchain (stable/nightly/version), compared against `rust-toolchain.toml` if present
- **Target size** — current size of the `target/` directory in GB, with verdict (pass/warn/fail) relative to configured max
- **Git state** — current branch, dirty file count, staged file count, untracked file count
- **Workspace health** — overall verdict based on all signals

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All signals pass |
| 1 | One or more signals in warn or fail state |

## Example output

```
workspace  my-workspace
toolchain  nightly (matches rust-toolchain.toml)
target     4.2 GB / 20.0 GB max  [pass]
git        main, clean
health     pass
```

## Related commands

- `cargo cicd target show` — detailed target directory breakdown
- `cargo cicd git status` — detailed git state
- `cargo cicd workspace doctor` — full workspace diagnostics
