<!-- BEGIN ggen:command-reference -->
<!-- Rendered from ontology/cargo-cicd-capabilities.ttl. Do not edit by hand. -->

# `cargo cicd test changed`

Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list.

**Noun:** `test` &nbsp;&nbsp; **Verb:** `changed`

<!-- END ggen:command-reference -->

<!-- BEGIN custom:examples -->
## Examples

```sh
cargo cicd test changed
```
<!-- END custom:examples -->
