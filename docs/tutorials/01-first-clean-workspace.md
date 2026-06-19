# Tutorial: Your First Clean Workspace

By the end of this tutorial you will have run `cargo cicd status` in your own workspace and seen a snapshot telling you whether it is push-ready.

**Prerequisites:**

- Rust 1.85 or later
- A Cargo workspace with at least one crate (a new `cargo new my-project` works fine)
- cargo-cicd installed: `cargo install cargo-cicd --version 26.6.19`

---

## Step 1 — Install cargo-cicd

```sh
cargo install cargo-cicd --version 26.6.19
```

Verify it is available:

```sh
cargo cicd --help
```

You should see a list of subcommands including `status`, `target`, `git`, and others.

---

## Step 2 — Open a workspace

Navigate to any Cargo workspace you own. If you do not have one handy:

```sh
cargo new hello-cicd
cd hello-cicd
```

---

## Step 3 — Run status

```sh
cargo cicd status
```

You will see output like:

```
workspace : hello-cicd
branch    : main
toolchain : stable
status    : CLEAN
```

Or if you have uncommitted changes:

```
status    : DIRTY — 2 file(s) with uncommitted changes
  M src/main.rs
  ? notes.txt
```

---

## Step 4 — Understand the output

- **workspace** — the name in your root `Cargo.toml`
- **branch** — your current git branch
- **toolchain** — the active Rust toolchain
- **CLEAN** — no dirty, staged, or untracked files; the workspace is push-ready
- **DIRTY** — at least one file has uncommitted changes; the list shows which ones

---

## Step 5 — Run the example binary

The same information is available as a library. Clone cargo-cicd and run:

```sh
cargo run --example 01_first_clean
```

You should see identical output to `cargo cicd status`.

---

## What you have learned

- cargo-cicd queries git and Cargo.toml at the workspace root
- `status` is the fast daily-driver command: it exits 0 on CLEAN, 0 on DIRTY (it is informational, not a gate)
- The library and the CLI surface the same state

**Next:** [Tutorial 2 — Emit your first OCEL evidence record](02-ocel-evidence.md)
