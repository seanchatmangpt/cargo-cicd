# Signing-key rotation (v26.9.24)

Recorded 2026-09-24 (fleet key scan after the single-repo migration). Base `7375116f23b0` of `cargo-cicd`.
Every private key listed here was committed to this repository and is therefore compromised: every receipt or
attestation signed with it carries no signing authority (standing REFUSED, broken_term R_missing_authority).
The keys leave the tree (history is not rewritten; no force-push), and each key directory's `.gitignore` now
covers both halves. Every checkout keeps its own pair: ggen generates one on first use, and a tracked public
half without its private half would make that first `ggen sync` refuse [FM-KEY-010/011]. The canonical
checkout's new public key is published below for anyone verifying its future receipts.

| key dir | removed private key sha256 | removed public key sha256 | new public key (canonical checkout) |
|---|---|---|---|
| `.ggen/keys` | `2507b5931108b6534d4bce8cdcc30b321cfb9404528de218e63ee0c7d2d78efa` | `cde310289254f34bd957977ca51a6f32622ea7c804efdb1b25d48f394a90b1a7` | `3ec269f8c2bd8b0888317f95d3a8bb64d88617033bf99a282bf87dcd714b6b2f` |
