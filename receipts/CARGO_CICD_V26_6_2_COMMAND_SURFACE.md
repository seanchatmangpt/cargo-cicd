---
receipt: CARGO_CICD_V26_6_2_COMMAND_SURFACE
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# Command Surface Receipt

## Public Commands (v26.6.2) — 9 total

| # | Command | Noun module | Verb | Exists | Runs without error |
|---|---|---|---|---|---|
| 1 | `cargo cicd status show` | `src/nouns/status.rs` | show | yes | yes |
| 2 | `cargo cicd target show` | `src/nouns/target.rs` | show | yes | yes |
| 3 | `cargo cicd target prune` | `src/nouns/target.rs` | prune | yes | yes (plan mode) |
| 4 | `cargo cicd test changed` | `src/nouns/test.rs` | changed | yes | yes |
| 5 | `cargo cicd trybuild changed` | `src/nouns/trybuild.rs` | changed | yes | yes |
| 6 | `cargo cicd git status` | `src/nouns/git.rs` | status | yes | yes |
| 7 | `cargo cicd git close` | `src/nouns/git.rs` | close | yes | yes |
| 8 | `cargo cicd publish run` | `src/nouns/publish.rs` | run | yes | yes |
| 9 | `cargo cicd workspace doctor` | `src/nouns/workspace.rs` | doctor | yes | yes |

## Grammar Law
All commands follow noun/verb pattern via clap-noun-verb v26.6.1.
No flag soup. No generic `run` command that does everything.
`--introspect` flag available on every noun for LLM tool-calling schema export.

## Observed CLI Output Sample (`cargo-cicd --help`)
```
Usage: cargo-cicd [OPTIONS] [COMMAND]

Commands:
  target     Manage target directory
  trybuild   Manage trybuild fixtures
  git        Git phase management
  workspace  Workspace diagnostics
  publish    Publish cicd.toml with current workspace state
  test       Run changed tests
  status     Show workspace CI/CD status
  help       Print this message or the help of the given subcommand(s)
```

## Note on `status` noun
`cargo cicd status` has a top-level `show` verb. The command resolves as `status show` internally.
Running `cargo cicd status show` directly produces the full workspace status dashboard.

## Verdict: ALIVE
