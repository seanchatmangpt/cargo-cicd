---
receipt: CARGO_CICD_V26_6_2_COMMAND_SURFACE
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# Command Surface Receipt

## Public Commands (v26.6.2)
| Command | Noun | Verb | Status |
|---------|------|------|--------|
| cargo cicd status | status | show | ALIVE |
| cargo cicd target show | target | show | ALIVE |
| cargo cicd target prune | target | prune | ALIVE (plan mode) |
| cargo cicd test changed | test | changed | ALIVE (conservative) |
| cargo cicd trybuild changed | trybuild | changed | ALIVE (conservative) |
| cargo cicd git status | git | status | ALIVE |
| cargo cicd git close | git | close | ALIVE (enforces clean tree) |
| cargo cicd publish | publish | run | ALIVE |
| cargo cicd workspace doctor | workspace | doctor | ALIVE |

## Grammar Law
All commands follow noun/verb pattern via clap-noun-verb v26.6.1.
No flag soup. No generic 'run' command.

## Verdict: ALIVE
