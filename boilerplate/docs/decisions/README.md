# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for PROJECT.

ADRs capture significant architectural choices: what was decided, why, and what alternatives were
considered. They are written once and treated as append-only history — superseded decisions are
marked `Superseded` rather than deleted.

## Format

Each ADR is a Markdown file named `NNNN-short-title.md`. Numbers are assigned sequentially and
never reused.

## Creating an ADR

Copy `0000-template.md`, increment the number, fill in the sections, and open a PR. The ADR
should be merged alongside (or before) the code that implements the decision.

```sh
cp docs/decisions/0000-template.md docs/decisions/00NN-my-decision.md
```

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-hexagonal-architecture.md) | Hexagonal Architecture for Domain Isolation | Accepted |
| [0002](0002-error-handling-strategy.md) | Error Handling Strategy (thiserror + anyhow) | Accepted |
