# Why Local-First CI/CD

## The problem with remote-first pipelines

Most CI/CD tools are designed to run on a server after you push. This means:

1. You push code.
2. You wait for the remote pipeline to queue, provision, and run.
3. You find out it failed — often 10–20 minutes later.
4. You fix the problem and push again.

This feedback loop is long. It discourages small, focused commits and
encourages "big bang" pushes that are harder to review and debug.

## What local-first means

`cargo-cicd` runs on your machine, before you push. Each command is designed
to be fast enough to run habitually — after every meaningful change, not just
before a PR.

Local-first has three properties:

1. **Immediate feedback.** You know within seconds, not minutes.
2. **No queue.** Your machine is always available; a CI server is shared.
3. **No secrets exposure.** Credentials and workspace state stay on your
   machine.

## The incremental principle

Remote pipelines typically run everything from scratch on every push because
they have no persistent state between runs. This is correct for remote CI
(you want a clean environment) but wasteful for local development.

`cargo-cicd` is stateful by design. It records what it did and when in
`cicd.toml`, so the next run can skip unchanged work. `test changed` only
runs tests for crates whose source changed. `target prune` only removes
artefacts older than the configured threshold.

The goal is a tool you run dozens of times a day without thinking about it —
not a ceremony you perform before a PR.

## Relationship to remote CI

`cargo-cicd` does not replace remote CI. Remote CI provides:

- A clean, reproducible environment
- Cross-platform testing
- Integration with your deployment pipeline
- A shared source of truth for your team

`cargo-cicd` catches the problems that would have wasted remote CI capacity.
When your push reaches the remote pipeline, it should be clean.
