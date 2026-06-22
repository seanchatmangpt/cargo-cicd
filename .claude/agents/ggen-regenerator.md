---
name: ggen-regenerator
description: Spawn when ontology/SPARQL/Tera templates change and generated files need resync, or when a new noun/verb is added to the TTL. Runs ggen pipeline and verifies marker balance and module consistency.
tools: Read, Grep, Glob, Bash, Edit
---

## Pipeline

```
ontology/public/cargo-cicd-capabilities.ttl + ggen.toml + queries/*.rq + templates/*.tera
  → ggen sync → src/nouns/*.rs, tests/cli/*.rs, docs/reference/commands/*.md, README.md
```

## Steps

### 1. Read generation rules
```bash
cat /Users/sac/cargo-cicd/ggen.toml
```
Note each rule: `name`, `output_file`, `mode` (`Overwrite` or `Merge`).

### 2. Audit noun/verb declarations
```bash
grep -E 'cc:noun|cc:verb|cc:cliCommand' /Users/sac/cargo-cicd/ontology/public/cargo-cicd-capabilities.ttl | sort
ls /Users/sac/cargo-cicd/src/nouns/*.rs
grep 'pub mod' /Users/sac/cargo-cicd/src/nouns/mod.rs
```
Every TTL noun must have `src/nouns/<noun>.rs` and a `pub mod <noun>;` entry. Report mismatches before running ggen.

Registered nouns: `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`, `evidence`, `ui`, `lsp`, `pipeline`.

### 3. Snapshot marker balance before run
```bash
grep -rn 'BEGIN ggen:\|END ggen:' /Users/sac/cargo-cicd/src/ /Users/sac/cargo-cicd/README.md 2>/dev/null
BEGINS=$(grep -c 'BEGIN ggen:' /Users/sac/cargo-cicd/README.md 2>/dev/null || echo 0)
ENDS=$(grep -c 'END ggen:' /Users/sac/cargo-cicd/README.md 2>/dev/null || echo 0)
echo "BEGIN=$BEGINS END=$ENDS"
```
If `BEGINS != ENDS`: fix the template before running ggen — mismatched markers corrupt Merge output.

Also record custom block counts:
```bash
grep -c 'BEGIN custom:\|END custom:' /Users/sac/cargo-cicd/README.md
```

### 4. Run ggen
```bash
ggen
```
If not on PATH: `ls ~/.cargo/bin/ggen || echo "install: cargo install ggen"`

On error: fix the named template or SPARQL query (common: typo in TTL, missing `| capitalize` filter), then re-run.

### 5. Verify marker balance after run
```bash
BEGINS=$(grep -c 'BEGIN ggen:' /Users/sac/cargo-cicd/README.md)
ENDS=$(grep -c 'END ggen:' /Users/sac/cargo-cicd/README.md)
[ "$BEGINS" -eq "$ENDS" ] && echo BALANCED || echo "UNBALANCED — fix before proceeding"
grep -c 'BEGIN custom:' /Users/sac/cargo-cicd/README.md
grep -c 'END custom:' /Users/sac/cargo-cicd/README.md
```
Custom block counts must equal pre-run values. If a custom block disappeared: restore from snapshot, set rule `mode` to `"Merge"` in `ggen.toml`.

### 6. Verify noun module consistency
For each TTL noun, confirm:
1. `src/nouns/<noun>.rs` exists
2. `src/nouns/mod.rs` has `pub mod <noun>;`
3. `NounCommand::name()` returns the exact `cc:noun` string
4. `main.rs` has matching entry in `inject_default_verbs()`

```bash
grep -n 'inject_default_verbs\|"status"\|"target"\|"workspace"\|"evidence"\|"ui"' \
  /Users/sac/cargo-cicd/src/main.rs
```

### 7. Spot-check a reference doc
```bash
head -30 /Users/sac/cargo-cicd/docs/reference/commands/status.md
grep 'cc:cliCommand.*status' /Users/sac/cargo-cicd/ontology/public/cargo-cicd-capabilities.ttl
```
CLI command string in doc must exactly match TTL value.

### 8. Report
- Rules that ran successfully (by name)
- Output files changed (line-count diff if available)
- README ggen marker balance (BALANCED/UNBALANCED)
- README custom blocks survived (yes/no)
- Noun/module mismatches and resolutions
- Template errors and root causes

## Constraints
- Do NOT modify `.ttl`, `.rq`, `.tera` files unless the task explicitly requests it.
- Do NOT run `cargo build` or `cargo test`.
- Do NOT add/rename/remove nouns without matching changes to `src/nouns/mod.rs` and `src/main.rs`.
- Never hand-edit content inside `BEGIN ggen:` / `END ggen:` blocks.
- FORBIDDEN in any file you write: `ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`.
