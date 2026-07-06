# cargo cicd workspace doctor

Diagnose workspace health.

## Usage

```bash
cargo cicd workspace doctor
```

## Checks performed

| Check | Pass condition |
|-------|---------------|
| Root `Cargo.toml` present | File exists and is parseable |
| Workspace members resolve | All members listed in `[workspace]` exist on disk |
| Git repository | `.git/` present at workspace root |
| Git HEAD valid | HEAD points to a valid branch or commit |
| Toolchain present | `rustup` reports an active toolchain |
| `rust-toolchain.toml` match | Active toolchain matches file if present |
| `cicd.toml` present | File exists (warning if absent, not error) |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All checks pass |
| 1 | One or more checks failed |

## Example output

```
[pass] Cargo.toml found
[pass] workspace members: 8 crates resolved
[pass] git repository present
[pass] HEAD: main
[pass] toolchain: nightly-2025-01-15
[warn] rust-toolchain.toml not found — toolchain match skipped
[warn] cicd.toml not found — run: cargo cicd publish
verdict: pass (2 warnings)
```

## Notes

- Warnings do not cause a non-zero exit. Only failures do.
- `cicd.toml` absence is always a warning, never a failure.
- For a quick summary view, use `cargo cicd status`. Use `doctor` when diagnosing a broken workspace setup.
