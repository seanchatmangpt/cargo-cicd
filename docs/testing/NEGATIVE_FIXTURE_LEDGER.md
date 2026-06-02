---
artifact: NEGATIVE_FIXTURE_LEDGER
date: 2026-06-02
---

# Negative Fixture Ledger

| Fixture | Tests | Expected Verdict | Key Assertion |
|---------|-------|-----------------|---------------|
| clean_workspace | status, publish, workspace doctor | pass | all green |
| dirty_workspace | git close | refuse | no false close |
| missing_manifest | workspace doctor | fail | explains missing Cargo.toml |
| toolchain_mismatch | test changed | warn | mismatch detected |
| target_over_limit | target show, prune | warn/suggest | no accidental delete |
| trybuild_changed_only | trybuild changed | changed-only | not full estate |
| trybuild_huge_set | trybuild changed | changed-only | same fixture count |
| corrupted_cicd_toml | publish | refuse/repair | no silent overwrite |
| stale_cicd_toml | publish | regenerate | detected stale |
| git_unrelated_dirty | git close | refuse | unrelated files blocked |
| wasm4pm_missing | wasm4pm feature | partial/blocked | honest PARTIAL |
| release_artifacts | target prune | preserve | release never deleted |
