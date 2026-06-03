# Why Changed-File Test Planning

## The cost of running all tests every time

In a large Cargo workspace with many crates, `cargo test --workspace` can take
minutes even when you changed a single file in one crate. Most of that time is
spent testing crates that could not possibly be affected by your change.

This is wasteful in a tight local feedback loop. If you are iterating on a
parser, you do not need to run the tests for your HTTP client, your CLI
formatter, or your database layer every time you save a file.

## How changed-file scoping works

`cargo cicd test changed` restricts the test run to crates whose source files
have changed since the last recorded green commit in `cicd.toml`.

"Changed" is determined by comparing the git working tree against a base ref
(default: `origin/main`) and looking at which files have been modified, added,
or deleted. The files are then mapped to Cargo crates by their workspace
membership.

Only the crates that own changed files are tested. All other crates are
silently skipped.

## The tradeoff: soundness vs. speed

Changed-file scoping is an optimization, not a guarantee. If crate A depends
on crate B, and you change crate B, `test changed` will test crate B but not
necessarily crate A (unless A's source files also changed).

This means `test changed` can miss integration failures that only appear when
A is built against the new B. This is an intentional tradeoff: the goal is
fast local feedback, not comprehensive integration testing. Remote CI handles
the full integration test.

For a complete test run, use `cargo test --workspace` before pushing or before
running `cargo cicd publish run`.

## Why trybuild gets the same treatment

`trybuild changed` applies the same logic to trybuild fixtures. This is
particularly important for crates that derive macros: trybuild fixture suites
can be large and slow. Testing only the fixtures for changed derive crates
keeps the trybuild feedback loop short.

## Recording what was tested

Every `test changed` and `trybuild changed` run writes a `TestChangedEvent`
or `TrybuildChangedEvent` to `cicd.toml`, recording which crates were tested,
how many tests passed or failed, and the timestamp.

`status show` reads this record to answer: "are there pending tests?" If source
files have been modified since the last recorded test run, pending tests are
reported. This gives you an accurate readiness signal without re-running tests.
