---
receipt: CARGO_CICD_V26_6_2_AUTONOMIC_POLICIES
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# Autonomic CI/CD Policies Receipt

## Policies (all suggest-mode, no apply-mode default)
| Name | Enabled | Mode | Signal | Recommendation |
|------|---------|------|--------|----------------|
| target_pressure | true | suggest | target size vs max | run target prune |
| toolchain_mismatch | true | suggest | active vs required toolchain | switch toolchain |
| trybuild_changed | true | suggest | changed trybuild fixtures | run trybuild changed |
| git_phase_dirty | true | suggest | working tree dirty | commit or stash |

## Rule
apply-mode is not enabled by default. No policy applies changes without explicit opt-in.

## Verdict: ALIVE
