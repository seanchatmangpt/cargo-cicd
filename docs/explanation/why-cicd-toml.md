# Why cicd.toml

## The state problem

CI/CD tools need state. Without it, every command must start from scratch:
re-scanning the workspace, re-running all tests, re-checking every condition.

Most CI/CD tools solve this with a remote database or artifact cache. This
works well for remote pipelines but is inappropriate for a local tool:

- A remote database requires connectivity.
- An artifact cache requires a cache server.
- Both require infrastructure you have to maintain.

`cargo-cicd` solves the state problem locally with `cicd.toml`.

## What cicd.toml carries

`cicd.toml` is a TOML file at the workspace root that serves two purposes:

1. **Configuration** — user-controlled settings that tell each command how
   to behave. These are read at startup and never overwritten by the command.

2. **State** — machine-written records of the last run of each command:
   timestamps, outcomes, metrics. These are written after a command completes.

This dual role means you have one file to inspect, one file to `.gitignore`,
and one file to reason about.

## The carrier pattern

`cicd.toml` is designed as a _carrier_: it carries structured information
forward from one command invocation to the next, making each command aware
of what the others did.

For example:

- `status show` reads the last `test changed` verdict from `cicd.toml` to
  report whether tests are pending.
- `publish run` reads the `dirty` flag from `cicd.toml` before attempting
  to publish.
- `git close` reads the `publish_ready` flag before merging.

Without `cicd.toml`, each command would have to re-derive this information
from scratch on every run. The carrier pattern makes commands composable.

## Why not a database or hidden directory?

A database would require a schema migration strategy and a query interface.
A hidden directory (`.cicd/`) would scatter state across multiple files.

TOML is human-readable, diff-friendly, and editable by hand when you need to
override a stale state entry. The tradeoff is that TOML is less suited to
high-frequency append-only writes — which is why the `[[events]]` array in
`cicd.toml` is truncated after a configurable limit rather than growing
indefinitely.

## What to do with cicd.toml

- **Add it to `.gitignore`.** It contains machine-local state paths and
  timing data that will conflict across team members.
- **Do not hand-edit the `[state]` section.** Let commands write it.
- **Do hand-edit the `[target]`, `[git.phase]`, and other policy sections**
  to tune command behavior to your workspace.
