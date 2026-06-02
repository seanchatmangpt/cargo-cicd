---
artifact: COMBINATORIAL_MAXIMALIST_TEST_PLAN
date: 2026-06-02
version: 26.6.2
---

# Combinatorial Maximalist Test Plan

## Law

The test unit is not a command. The test unit is a capability.
The proof unit is a combination. The completion unit is a receipt.

## Coverage Bands

| Band | Purpose | Example |
|------|---------|--------|
| Singleton | Each capability alone | target show on small target |
| Pairwise | Two-way interactions | dirty git + publish |
| Critical 3-wise | Dangerous triangles | dirty + trybuild + git close |
| Boundary/Refusal | Unsafe states refuse | corrupted cicd.toml |
| Metamorphic | Repeated runs converge | publish twice = stable |

## Capability Dimensions

1. Command surface: status, target show/prune, test changed, trybuild changed, git status/close, publish, workspace doctor
2. Feature flags: default, process-data, autonomic, wasm4pm
3. Workspace shape: single, workspace, missing manifest, virtual, nested
4. Toolchain: stable, nightly, pinned, missing file, mismatch
5. Git state: clean, dirty tracked, untracked, staged, ahead, behind, detached
6. Target: absent, small, over limit, stale profiles, release artifacts
7. Test state: none, unit, integration, changed source, changed test
8. Trybuild: no fixtures, changed, changed stderr, snapshot mismatch, huge set
9. cicd.toml: absent, valid, stale, corrupted, partial
10. Autonomic: disabled, suggest, plan, apply-forbidden
11. wasm4pm: not found, scan only, file exchange, shell-out, local adapter, deferred
