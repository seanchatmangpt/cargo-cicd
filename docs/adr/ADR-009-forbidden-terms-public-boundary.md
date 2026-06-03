# ADR-009: Forbidden Terms at Public Boundary

**Status:** Accepted
**Date:** 2026-06-03

## Context

cargo-cicd has a dual identity: a public Rust CI/CD helper and a private Level 5 process-data engine. The internal manufacturing vocabulary includes terms that describe the private architecture and must not appear in any public-facing surface. These terms, if exposed, would reveal proprietary manufacturing concepts, create confusion for public users, and violate the separation between the public product and its private implementation.

## Decision

A set of 10 internal manufacturing terms is forbidden in all public surfaces — CLI help text, public API docs, README, crates.io description, error messages visible to end users, and any public documentation. The authoritative list is maintained in the project CLAUDE.md under "FORBIDDEN in public docs/CLI/help text".

These terms may appear in internal receipts, private architectural notes, and internal CLAUDE.md files that are not part of the public API surface. The ADR itself does not enumerate them to avoid triggering the guard that it describes.

The invariants test (`tests/invariants.rs`) enforces this boundary by scanning all public-facing files for forbidden terms.

## Rationale

Public users of cargo-cicd need to understand it as a CI/CD helper. Internal manufacturing vocabulary creates a confusing dual-language problem. More critically, some of these terms are proprietary identifiers for the private engine architecture that must not be disclosed in the public product.

## Consequences

- CI checks scan public docs, help text, and API docs for forbidden terms.
- Any PR that introduces a forbidden term into a public surface is blocked.
- The invariants test includes a `forbidden_terms_public_boundary` test case.
- Internal documentation (receipts, CLAUDE.md) is exempt from this rule.

## Violation

A forbidden term appearing in CLI help text, README, crates.io metadata, or any doc that a public user would read constitutes a defect. It is not a style issue — it is a boundary violation that must be fixed before release.
