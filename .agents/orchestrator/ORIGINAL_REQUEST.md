# Original User Request

## Initial Request — 2026-06-29T04:37:27Z

Update the documentation of the cargo-cicd project to position it as an operational substrate and specify the star-toml refactor path.

Working directory: /Users/sac/cargo-cicd
Integrity mode: development

## Requirements

### R1. Create Dedicated star-toml Refactor Documentation
- Create a new directory `docs/star-toml-refactor/`.
- In `docs/star-toml-refactor/PRD.md`, write the Product Requirements Document defining the vision, mission, position, product principles, supported surfaces, workflows, security philosophy, and success metrics for cargo-cicd as an operational substrate.
- In `docs/star-toml-refactor/ARD.md`, write the Architecture Requirements Document detailing the architecture diagram, layers (operational law, planning, execution, verification, standing), authority model, core invariants, security model, and Chatman's Law.
- In `docs/star-toml-refactor/REFACTOR.md`, write the guide for refactoring cargo-cicd to use star-toml (e.g. loader pipeline, strict admission mode, diagnostics, receipt verification loops).

### R2. Update Top-Level README.md
- Modify the main `README.md` of `cargo-cicd` to introduce the project's new positioning as an operational substrate and point to the new refactor documentation.

## Acceptance Criteria

### Refactor Documentation
- [ ] Directory `docs/star-toml-refactor/` exists.
- [ ] `docs/star-toml-refactor/PRD.md` contains the vision ("Make Cargo the operational substrate for software engineering"), mission, position, principles, workflows, and metrics.
- [ ] `docs/star-toml-refactor/ARD.md` contains the architectural layers, authority model, core invariants, and security model.
- [ ] `docs/star-toml-refactor/REFACTOR.md` contains the detailed guide for using star-toml (typestates, path policies, and receipts) inside cargo-cicd.

### Top-Level README
- [ ] Top-level `README.md` of `cargo-cicd` has been updated to reflect the operational substrate model and includes links to the new `docs/star-toml-refactor/` files.
