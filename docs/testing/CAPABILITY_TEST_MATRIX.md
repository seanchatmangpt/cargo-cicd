---
artifact: CAPABILITY_TEST_MATRIX
date: 2026-06-02
---

# Capability Test Matrix

## Singleton Tests

| Command | Workspace | Git | Toolchain | Target | Expected |
|---------|-----------|-----|-----------|--------|----------|
| status | workspace | clean | pinned nightly | small | pass |
| status | workspace | dirty | pinned nightly | small | warn |
| target show | workspace | clean | nightly | over limit | warn |
| target prune | workspace | any | any | over limit | suggest plan |
| test changed | workspace | dirty | nightly | small | conservative plan |
| trybuild changed | workspace | dirty | nightly | small | changed-only plan |
| git status | workspace | dirty | any | any | report dirty |
| git close | workspace | clean | any | any | no-op pass |
| git close | workspace | dirty unrelated | any | any | refuse |
| publish | workspace | clean | nightly | small | create cicd.toml |
| workspace doctor | missing manifest | any | any | any | fail explain |

## Critical 3-Wise Cases

| Case | State 1 | State 2 | State 3 | Expected |
|------|---------|---------|---------|----------|
| dirty+trybuild+close | dirty git | changed fixture | git close | refuse close |
| mismatch+changed+pdata | toolchain mismatch | changed tests | process-data on | warn + emit event |
| overlimit+release+prune | target over limit | release artifacts | prune | preserve release |
| corrupt+publish+auto | corrupted cicd.toml | publish | autonomic on | refuse/repair |
| wasm4pm_missing+feature+publish | wasm4pm absent | wasm4pm feature | publish | PARTIAL, no fake |
