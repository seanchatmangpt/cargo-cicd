# cargo-cicd-lsp

cargo-cicd-lsp surfaces local Rust workspace readiness problems in the editor before CI fails.

It is a read-only Language Server Protocol (LSP) server that observes your workspace and
publishes diagnostics when manufacturing state drifts from admissible closure conditions.

## What it observes

- Git working tree state
- Process evidence freshness and structure
- Publish readiness indicators
- Public boundary safety
- Rendered source surface drift
- wasm4pm / wpm capability availability
- Target directory growth
- Changed test coverage

## What it does not do

- Execute commands automatically
- Commit, publish, prune, or modify source
- Judge evidence (that is wpm's role)
- Replace CI

## Install

```sh
cargo install cargo-cicd-lsp
```

## Editor setup

See [EDITOR_INTEGRATION.md](EDITOR_INTEGRATION.md)

## Usage

```sh
cargo cicd lsp serve
cargo cicd lsp doctor
cargo cicd lsp explain <CODE>
```

## See Also

- [DIAGNOSTICS.md](DIAGNOSTICS.md) — Full catalog of diagnostic codes by family
- [LIFECYCLE.md](LIFECYCLE.md) — How diagnostics are raised, routed, and cleared
- [EDITOR_INTEGRATION.md](EDITOR_INTEGRATION.md) — VS Code, Neovim, Helix, Zed setup
