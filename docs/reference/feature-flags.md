# Feature Flags Reference

`cargo-cicd` uses Cargo feature flags to gate optional capabilities.
The default feature set covers the full public CLI surface.

## Feature overview

| Feature | Default | Description |
|---------|---------|-------------|
| `default` | yes | All nine public CLI commands |
| `process-data` | no | XES/OCEL evidence emission |
| `autonomic` | no | Policy suggestions (implies `process-data`) |
| `wasm4pm` | no | External oracle integration (implies `process-data`) |

## default

Always enabled. Provides all nine CLI commands:

- `status show`
- `target show` / `target prune`
- `test changed`
- `trybuild changed`
- `git status` / `git close`
- `publish run`
- `workspace doctor`

No opt-in required.

## process-data

Enables emission of structured process-data events in XES/OCEL format.
Each command emits an event when it completes, recording inputs, outputs,
and timing.

Enable in `Cargo.toml`:

```toml
[dependencies]
cargo-cicd = { version = "...", features = ["process-data"] }
```

Or when installing:

```sh
cargo install cargo-cicd --features process-data
```

Events are written to the path configured in `cicd.toml`:

```toml
[process_data]
output_path = ".cicd/events.xes"
```

See [Evidence Format](evidence-format.md) for the XES schema.

## autonomic

Enables policy suggestions: after each command run, cargo-cicd analyses
recent event history and suggests configuration changes that would reduce
friction (e.g., "your target directory averages 8 GB — consider lowering
`max_size_gb` to 6").

Implies `process-data`. Requires `process-data` event history to generate
suggestions.

Enable in `Cargo.toml`:

```toml
[dependencies]
cargo-cicd = { version = "...", features = ["autonomic"] }
```

Configure suggestion mode in `cicd.toml`:

```toml
[autonomic]
enabled = true
mode = "suggest"   # "suggest" prints suggestions; "apply" writes them to cicd.toml
```

## wasm4pm

Enables integration with an external process-mining oracle (wasm4pm). When
enabled, events emitted by `cargo-cicd` can be forwarded to the oracle for
conformance checking against a declared process model.

Implies `process-data`. The oracle endpoint is configured separately; see the
wasm4pm documentation for setup.

This feature requires the `wasm4pm` binary to be available in `PATH`. If the
binary is not found, commands run normally without oracle integration — the
feature degrades gracefully.
