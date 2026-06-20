## Summary

<!-- What does this PR do? Why? -->

## Changes

<!-- List the specific changes made -->

- 

## Type of Change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change
- [ ] Refactor (no functional change)
- [ ] Documentation
- [ ] CI/tooling

## Testing

- [ ] `cargo make test` passes locally
- [ ] New tests added for changed behaviour
- [ ] `cargo make check` (fmt + clippy) passes

## Checklist

- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] No forbidden terms in help text (`cargo test --test invariants`)
- [ ] Feature-flag combinations still compile (`cargo make feature-check`)
- [ ] `deny.toml` policy still passes (`cargo deny check`)
