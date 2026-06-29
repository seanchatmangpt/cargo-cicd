# Project: cargo-cicd Operational Substrate and star-toml Refactor Docs

## Architecture
- `docs/star-toml-refactor/PRD.md`: Defines the product vision ("Make Cargo the operational substrate for software engineering"), mission, positioning, principles, surfaces, workflows, security philosophy, and success metrics.
- `docs/star-toml-refactor/ARD.md`: Details the architectural layers (operational law, planning, execution, verification, standing), authority model, core invariants, security model, and Chatman's Law.
- `docs/star-toml-refactor/REFACTOR.md`: Specifies the refactor guide for using star-toml within cargo-cicd, covering the loader pipeline, strict admission mode, diagnostics, and receipt verification loops.
- `README.md`: Integrates the operational substrate concept into the project's front-facing surface and links to the refactor documentation.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Create PRD | Create directory `docs/star-toml-refactor/` and write `PRD.md` with complete vision and workflows | None | DONE |
| M2 | Create ARD | Write `ARD.md` detailing architectural layers, authority, invariants, and Chatman's Law | M1 | DONE |
| M3 | Create REFACTOR | Write `REFACTOR.md` specifying loader integration, strict admission, and receipts | M2 | DONE |
| M4 | Update README | Modify top-level `README.md` to reflect new positioning and add links to docs | M3 | DONE |
| M5 | Audit & Review | Verify all files exist, check content completeness, and run audit tools | M4 | DONE |

## Interface Contracts
- All documentation files must be written in Markdown format.
- Links between `README.md` and `docs/star-toml-refactor/*.md` must be correct, relative, and verified.
- The `docs/star-toml-refactor/` directory must be created.
