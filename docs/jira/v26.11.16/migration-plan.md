# cargo-cicd migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/cargo-cicd
- Remote: https://github.com/seanchatmangpt/cargo-cicd.git
- Branch: main@6ab317f (tracks origin/main)
- Last commit: 2026-10-10 (chore(release): v26.10.10) | commits: 359 | size: 29 MB
- Languages/build: Rust workspace (Cargo.toml, crates/, benches/, deny.toml), Justfile,
  ggen ontology pipeline (.ggen, .ggen-v2), CLAUDE.md present
- Nested repos: none

## Observed state
- Dirty 0 / untracked 0 — clean tree
- ahead 0 / behind 0; stash count: 136 (census); unpushed local branches: 12
  (exp/trybuild-full-verb, security/rotate-signing-keys-v26.9.24, 5+ subagent-*-research-*
  branches all at 8748a56)
- Build/dep dirs: none (target/ absent)
- Submodules: none

## What it is
cargo-cicd is a local-first CI/CD executor for Rust workspaces: build/test/publish on the
operator machine, CI consumes receipts rather than secrets. Privately a noun-verb CLI
manufactured from an ontology via ggen, emitting XES evidence.

## Classification
ACTIVE — census value stands (released v26.10.10 a month ago, clean tree).

## Migration risk
MEDIUM — tree is clean and pushed, but 136 stashes and 12 unpushed branches would be lost.
The subagent-* branches all point at the same old commit (8748a56) and look disposable; the
security/rotate-signing-keys branch references key material — inspect before pushing.

## Recommended action
CLONE-FRESH (after triaging stashes and branches)
1. `cd /Users/sac/cargo-cicd && git stash list | wc -l` — 136 stashes. DECISION: these are
   almost certainly dead; preserve via `git bundle create ../cargo-cicd-stashes.bundle --all \
   $(git stash list --format=%H)` if any doubt, else accept loss.
2. `git push origin exp/trybuild-full-verb` if the experiment is wanted.
3. DECISION: security/rotate-signing-keys-v26.9.24 — confirm no private key is in the branch
   before `git push`; otherwise bundle it to the archive volume only.
4. Drop the 5 subagent-*-research-* branches (all at 8748a56, superseded by main).
5. On new machine: `git clone https://github.com/seanchatmangpt/cargo-cicd.git`.

## First commands on the new machine
- `cargo check` then `just check` (stated, not run)

## See Also
- ../v26.9.19/
- /Users/sac/docs/jira/v26.11.16/README.md
