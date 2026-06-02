---
receipt: CARGO_CICD_V26_6_2_AUTONOMIC_POLICIES
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# Autonomic CI/CD Policies Receipt

## Policies — all 4 registered, all suggest-mode

### 1. target_pressure
- **File:** `src/policies/target_pressure.rs`
- **Mode:** suggest
- **Signals:** target directory size (`total_size_gb`) vs `max_gb` (default 20.0)
- **Verdict thresholds:** `>= max_gb` → alert; `>= 70% of max_gb` → warn; else → pass
- **Recommendation template (alert):** `"target/ is {size:.1}GB (limit {max}GB) — run 'cargo cicd target prune'"`
- **Recommendation template (warn):** `"target/ is {size:.1}GB ({pct:.0}% of limit) — consider pruning soon"`

### 2. toolchain_mismatch
- **File:** `src/policies/toolchain_mismatch.rs`
- **Mode:** suggest
- **Signals:** active rustup toolchain vs `channel` in `rust-toolchain.toml`
- **Verdict thresholds:** mismatch → warn; no `rust-toolchain.toml` or match → pass
- **Recommendation template:** `"active toolchain '{active}' does not match required '{required}' — run 'rustup override set {required}'"`

### 3. trybuild_changed
- **File:** `src/policies/trybuild_changed.rs`
- **Mode:** suggest
- **Signals:** count of changed `.rs` files under `tests/ui/` (trybuild fixtures) vs `origin/main`
- **Verdict thresholds:** `>= 1` fixture changed → warn; 0 → pass
- **Recommendation template:** `"{n} trybuild fixture(s) changed — run 'cargo cicd trybuild changed' to test selectively"`

### 4. git_phase_dirty
- **File:** `src/policies/git_phase_dirty.rs`
- **Mode:** suggest
- **Signals:** `git status --porcelain` non-empty output
- **Verdict thresholds:** dirty → alert; clean → pass
- **Recommendation template:** `"working tree is dirty — commit or stash changes before CI run"`

## Rule
`apply` mode is not enabled by default. No policy modifies the repository without explicit opt-in.
All policies implement the `CicdPolicy` trait: `name()`, `enabled()`, `mode()`, `evaluate() -> PolicyResult`.

## Observed live evaluation (2026-06-02, `workspace doctor`)
```
[PASS] target_pressure
[PASS] toolchain_mismatch
[PASS] trybuild_changed
[SUGGEST] git_phase_dirty: Run cargo cicd git close to commit 2 dirty files
```

## Verdict: ALIVE
