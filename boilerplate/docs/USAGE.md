# cargo-project Usage Guide

`cargo-project` keeps your Rust workspace clean, fast, and push-ready.

---

## Table of Contents

1. [Installation](#installation)
2. [First Run](#first-run)
3. [Commands Reference](#commands-reference)
   - [status](#status)
   - [workspace](#workspace)
   - [completions](#completions)
4. [Shell Completion Setup](#shell-completion-setup)
5. [Environment Variables](#environment-variables)
6. [Exit Codes](#exit-codes)
7. [Man Page](#man-page)

---

## Installation

### Via cargo install (recommended)

```sh
cargo install cargo-project
```

The binary is named `cargo-project` and registers itself as a Cargo subcommand,
so both `cargo project` and `cargo-project` work:

```sh
cargo project status      # as a cargo subcommand
cargo-project status      # direct binary invocation
```

### From a pre-built release binary

Download the binary for your platform from the
[releases page](https://github.com/seanchatmangpt/PROJECT/releases), then put
it on your `PATH`:

```sh
# Linux / macOS example
chmod +x cargo-project
mv cargo-project ~/.local/bin/
```

### From source

```sh
git clone https://github.com/seanchatmangpt/PROJECT
cd PROJECT
cargo build --release
# Binary is at target/release/cargo-project
```

---

## First Run

After installation, confirm everything works:

```sh
cargo project --version
# cargo-project 0.1.0

cargo project status
# ✔ my-workspace   [PASS]
#
#   ✔ git  branch=main dirty=0 staged=0 untracked=0
#
#   ℹ toolchain  1.86.0
```

You're done. The tool reads your workspace without any configuration file.

---

## Commands Reference

### status

Workspace health snapshot.  
Default verb: `show` — bare `cargo project status` is identical to `cargo project status show`.

#### status show

```
cargo project status show [--json] [--verbose]
```

Displays a one-screen summary of workspace health:

| Section | What is shown |
|---|---|
| Header | Workspace name + overall verdict badge |
| git | Branch name, dirty / staged / untracked file counts |
| toolchain | Active Rust version |

**Flags**

| Flag | Description |
|---|---|
| `--json` | Emit machine-readable JSON. |
| `--verbose`, `-v` | List each dirty / staged file below the summary. |

**Examples**

```sh
# Default human-readable output
cargo project status

# JSON output — useful in CI scripts and for piping to jq
cargo project status show --json

# Pipe JSON to jq for a specific field
cargo project status show --json | jq .verdict

# Full output with per-file detail
cargo project status show --verbose

# JSON + verbose combined
cargo project status show --json --verbose
```

**Sample JSON output**

```json
{
  "workspace": "my-workspace",
  "root_path": "/home/user/my-workspace",
  "git_phase": {
    "branch": "main",
    "dirty_files": 0,
    "staged_files": 0,
    "untracked_files": 1,
    "ahead": 0,
    "behind": 0
  },
  "verdict": "WARN"
}
```

---

### workspace

Workspace-wide diagnostics.  
Default verb: `doctor` — bare `cargo project workspace` is identical to `cargo project workspace doctor`.

#### workspace doctor

```
cargo project workspace doctor [--json]
```

Runs a suite of workspace health checks:

- `Cargo.toml` is present and well-formed.
- All workspace members listed under `[workspace.members]` resolve without errors.
- `rust-toolchain.toml` (if present) matches the active toolchain.
- No duplicate package names across members.

**Flags**

| Flag | Description |
|---|---|
| `--json` | Emit a JSON result object. |

**Examples**

```sh
# Interactive diagnostics output
cargo project workspace

# Pipe the result into another tool
cargo project workspace doctor --json | jq .status
```

**Sample JSON output**

```json
{
  "workspace": "my-workspace",
  "status": "OK"
}
```

---

### completions

Generate a shell tab-completion script and write it to stdout.

```
cargo project completions --shell <bash|zsh|fish|powershell|elvish>
```

**Flag**

| Flag | Description |
|---|---|
| `--shell <name>` | Required. Shell to generate completions for. |

See [Shell Completion Setup](#shell-completion-setup) for per-shell install paths.

---

## Shell Completion Setup

### Automated installer (recommended)

The repo ships `scripts/install-completions.sh`, which auto-detects your
shell and writes the completion file to the correct location:

```sh
# Auto-detect shell from $SHELL
./scripts/install-completions.sh

# Specify shell explicitly
./scripts/install-completions.sh bash
./scripts/install-completions.sh zsh
./scripts/install-completions.sh fish
```

### Manual setup — Bash

```sh
# 1. Create the per-user completion directory (if absent)
mkdir -p ~/.bash_completion.d

# 2. Generate and save the completion script
cargo project completions --shell bash > ~/.bash_completion.d/cargo-project

# 3. Source it in this session
source ~/.bash_completion.d/cargo-project

# 4. Make it permanent — add to ~/.bashrc
echo 'source ~/.bash_completion.d/cargo-project' >> ~/.bashrc
```

### Manual setup — Zsh

```sh
# 1. Create a completions directory
mkdir -p ~/.zsh/completions

# 2. Generate the completion function (_cargo-project is the zsh convention)
cargo project completions --shell zsh > ~/.zsh/completions/_cargo-project

# 3. Ensure the directory is in $fpath (add to ~/.zshrc before compinit)
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit

# 4. Reload completions in the current session
autoload -Uz compinit && compinit
```

### Manual setup — Fish

Fish automatically picks up completions files; no sourcing needed.

```sh
cargo project completions --shell fish \
    > ~/.config/fish/completions/cargo-project.fish
```

Open a new fish session and tab-completions are active.

### Manual setup — PowerShell

```powershell
# Append to your PowerShell profile so it loads on every start
cargo project completions --shell powershell | Out-File -Append $PROFILE

# Reload the profile in the current session
. $PROFILE
```

### Manual setup — Elvish

```sh
mkdir -p ~/.config/elvish/completions
cargo project completions --shell elvish \
    > ~/.config/elvish/completions/cargo-project.elv
```

Then add to `~/.config/elvish/rc.elv`:

```elvish
use ~/.config/elvish/completions/cargo-project
```

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `RUST_LOG` | `warn` | Structured-logging filter. Set to `debug` for trace-level adapter diagnostics. Example: `RUST_LOG=debug cargo project status`. |
| `APP_ENV` | _(unset)_ | Set to `ci` to disable ANSI colour and emit plain text suitable for log aggregators. Automatically implied when stdout is not a TTY. |
| `NO_COLOR` | _(unset)_ | When set to any non-empty value, all ANSI colour output is suppressed (per [no-color.org](https://no-color.org)). |
| `CARGO_TERM_COLOR` | _(unset)_ | Standard Cargo colour control. Set to `never` to suppress colour in any `cargo` invocations that `cargo-project` shells out to. |

**Example — run status in CI mode with debug logging**

```sh
RUST_LOG=debug APP_ENV=ci cargo project status show --json
```

---

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success. Workspace is healthy (PASS or WARN). |
| `1` | Error or unhealthy workspace (FAIL). |
| `2` | Bad command-line arguments (clap error). |

In CI scripts you can use the exit code as a gate:

```sh
cargo project status || echo "Workspace unhealthy — blocking push"
```

---

## Man Page

A man page is included in `docs/man/cargo-project.1`.  View it with:

```sh
man ./docs/man/cargo-project.1
```

To regenerate the man page (requires `help2man` or falls back to the
hand-maintained file):

```sh
./scripts/generate-man.sh
```

To install the man page system-wide:

```sh
# Linux
sudo cp docs/man/cargo-project.1 /usr/local/share/man/man1/
sudo mandb

# macOS (Homebrew prefix)
cp docs/man/cargo-project.1 /opt/homebrew/share/man/man1/
```
