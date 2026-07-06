# cargo-cicd Troubleshooting Guide

A comprehensive guide to diagnosing and fixing common issues with cargo-cicd.

**Version:** 26.6.19

## Table of Contents

1. [Installation & Setup Issues](#installation--setup-issues)
2. [Command Execution Problems](#command-execution-problems)
3. [Workspace Issues](#workspace-issues)
4. [Git-Related Problems](#git-related-problems)
5. [Target Directory Issues](#target-directory-issues)
6. [Evidence & Oracle Problems](#evidence--oracle-problems)
7. [Testing & Fixture Issues](#testing--fixture-issues)
8. [Advanced Diagnostics](#advanced-diagnostics)

---

## Installation & Setup Issues

### "command not found: cargo cicd"

**Problem:** The command is not recognized after installation.

**Causes:**
- cargo-cicd is not installed
- ~/.cargo/bin is not on PATH
- Shell needs to reload PATH

**Solutions:**

1. **Verify installation:**
   ```sh
   cargo install cargo-cicd
   ```

2. **Check if ~/.cargo/bin is on PATH:**
   ```sh
   echo $PATH | grep cargo
   # Should output something like /home/user/.cargo/bin
   ```

3. **Add to PATH if missing:**
   ```sh
   export PATH="$HOME/.cargo/bin:$PATH"
   # Add to ~/.bashrc or ~/.zshrc for persistence
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

4. **Try the full path:**
   ```sh
   ~/.cargo/bin/cargo-cicd --version
   ```

5. **Update cargo and reinstall:**
   ```sh
   rustup update
   cargo install --force cargo-cicd
   ```

### "version mismatch" or "corrupted installation"

**Problem:** cargo-cicd runs but reports errors or behaves erratically.

**Solutions:**

1. **Reinstall cleanly:**
   ```sh
   cargo uninstall cargo-cicd
   cargo install cargo-cicd
   ```

2. **Verify installation:**
   ```sh
   cargo cicd --version
   # Should output: cargo-cicd 26.6.19
   ```

3. **Check for conflicts (multiple versions):**
   ```sh
   which cargo-cicd
   which -a cargo-cicd  # Shows all matches
   ```

4. **Wipe local build cache if reinstalling from source:**
   ```sh
   cargo clean
   cargo install cargo-cicd --force
   ```

---

## Command Execution Problems

### "not a valid noun" or "unknown command"

**Problem:** The noun or verb you're using is not recognized.

**Causes:**
- Typo in noun/verb name
- Using outdated syntax
- Using a verb that doesn't exist for that noun

**Solutions:**

1. **Check available commands:**
   ```sh
   cargo cicd --help
   ```

2. **Check nouns:**
   ```sh
   cargo cicd status --help
   cargo cicd target --help
   cargo cicd git --help
   cargo cicd test --help
   cargo cicd trybuild --help
   cargo cicd publish --help
   cargo cicd workspace --help
   cargo cicd evidence --help
   cargo cicd pipeline --help
   ```

3. **Review command syntax:**
   ```sh
   # Correct syntax
   cargo cicd <noun> [verb] [flags]
   
   # Examples
   cargo cicd status show            # correct
   cargo cicd status                 # correct (show is default)
   cargo cicd status clean           # WRONG (clean is not a verb)
   cargo cicd clean status           # WRONG (clean is not a noun)
   ```

4. **Common typos to check:**
   - `test run` → should be `test changed`
   - `target clean` → should be `target prune`
   - `publish check` → should be `publish run`

### "execution error" or "command failed"

**Problem:** Command starts but returns a non-zero exit code.

**Solutions:**

1. **Get more details:**
   ```sh
   cargo cicd <command> 2>&1 | tee /tmp/cicd-debug.log
   # Read the log for error details
   cat /tmp/cicd-debug.log
   ```

2. **Check stderr separately:**
   ```sh
   cargo cicd <command> 2>&1 >/dev/null
   # Shows only errors
   ```

3. **Add verbose output (if supported):**
   ```sh
   # Some commands support environment variables for debugging
   RUST_LOG=debug cargo cicd <command>
   ```

4. **Test with a simpler command:**
   ```sh
   # Start with read-only commands that can't fail
   cargo cicd status           # Should always succeed
   cargo cicd target show      # Should always succeed
   cargo cicd workspace        # Will show issues
   ```

---

## Workspace Issues

### "not a workspace" or "Cargo.toml not found"

**Problem:** cargo-cicd can't find the workspace manifest.

**Causes:**
- You're in the wrong directory
- Cargo.toml is missing or named incorrectly
- It's a package, not a workspace

**Solutions:**

1. **Verify you're in the right directory:**
   ```sh
   ls -la Cargo.toml
   # Should exist and be readable
   ```

2. **Check Cargo.toml content:**
   ```sh
   cat Cargo.toml | head -20
   # Should start with [package] or [workspace]
   ```

3. **Navigate to workspace root:**
   ```sh
   # For a workspace, go to the root directory
   cd /path/to/my-workspace
   ls Cargo.toml  # This should exist at root
   ```

4. **Check for workspace definition:**
   ```sh
   # If Cargo.toml doesn't have [workspace], create one:
   cat Cargo.toml
   # Look for [workspace] or [[workspace.members]]
   ```

5. **Run from workspace root:**
   ```sh
   pwd
   # Should be the directory containing Cargo.toml
   ```

### "workspace doctor" reports failures

**Problem:** `cargo cicd workspace doctor` shows [FAIL] or [WARN] statuses.

**Solutions by status:**

**[FAIL] Cargo.toml:**
```sh
# Ensure Cargo.toml exists at workspace root
ls -la Cargo.toml

# Check if it's valid TOML
cargo check  # Will validate Cargo.toml
```

**[FAIL] git repository:**
```sh
# Initialize git if needed
git init
git add .
git commit -m "Initial commit"

# Or check if .git exists
ls -la .git
```

**[WARN] rust-toolchain file:**
```sh
# Optional, but recommended: create a toolchain pinning file
cat > rust-toolchain.toml <<EOF
[toolchain]
channel = "stable"
EOF

# Or for a specific version:
cat > rust-toolchain.toml <<EOF
[toolchain]
channel = "1.75.0"
EOF
```

**[WARN] cicd.toml (missing):**
```sh
# Generate it by publishing current state
cargo cicd publish run
```

**Autonomic policy warnings:**
- Check `workspace doctor` output for specific recommendations
- Follow the "repair" suggestions in the autonomic policy results

---

## Git-Related Problems

### "git repository not found"

**Problem:** cargo-cicd can't interact with git.

**Causes:**
- Not in a git repository
- .git directory is corrupted
- Git is not installed

**Solutions:**

1. **Verify git repository:**
   ```sh
   git status
   # Should show branch and working tree status
   ```

2. **Check .git directory:**
   ```sh
   ls -la .git
   # Should exist and be readable
   ```

3. **Reinitialize git if corrupted:**
   ```sh
   # WARNING: This is destructive. Only do if .git is unrecoverable.
   rm -rf .git
   git init
   git add .
   git commit -m "Restore repository"
   ```

4. **Ensure git is installed:**
   ```sh
   git --version
   # Should output a version number
   ```

5. **Check git configuration:**
   ```sh
   git config user.name
   git config user.email
   # Both should be set for commits
   ```

### "git status" shows unexpected results

**Problem:** `cargo cicd git status` shows different state than `git status`.

**Common discrepancies:**

1. **Staged files not appearing:**
   ```sh
   # cargo cicd counts staged files from git
   git status --short
   # Compare with output of:
   cargo cicd git status
   ```

2. **Untracked files counted differently:**
   ```sh
   # Ensure .gitignore is applied
   git status --ignored
   # Look for ignored files that might show as untracked
   ```

3. **Ahead/behind count differs:**
   ```sh
   # Make sure remote is set correctly
   git remote -v
   # Should show origin and/or other remotes
   
   # Update local tracking
   git fetch origin
   git status
   ```

### "git close refuses phase closure"

**Problem:** `cargo cicd git close` fails even though tree appears clean.

**Causes:**
- Untracked files exist
- Modification times trigger false positives
- Staged files present
- Hidden/special files present

**Solutions:**

1. **Check what git sees as dirty:**
   ```sh
   git status --porcelain
   # Each line is a file, first char is status
   # M = modified, ? = untracked, A = added, D = deleted
   ```

2. **Stage all changes:**
   ```sh
   git add .
   git status
   # Now git close should allow closure
   ```

3. **Handle untracked files:**
   ```sh
   # Either commit them
   git add <untracked-file>
   git commit -m "Add file"
   
   # Or add to .gitignore
   echo "untracked-file" >> .gitignore
   git add .gitignore
   git commit -m "Ignore file"
   ```

4. **Force refresh of file index:**
   ```sh
   git update-index --refresh
   git status
   ```

5. **Check for hidden files:**
   ```sh
   ls -la
   # Look for files starting with . that might be uncommitted
   ```

---

## Target Directory Issues

### "target directory too large"

**Problem:** `cargo cicd target show` reports size exceeds 20 GB limit.

**Causes:**
- Many debug builds accumulated
- Dependencies not optimized
- Long build history
- Large test artifacts

**Solutions:**

1. **Check current size:**
   ```sh
   cargo cicd target show
   # Shows total size and breakdown
   ```

2. **Preview cleanup:**
   ```sh
   cargo cicd target prune
   # Shows what would be deleted without --apply
   ```

3. **Execute cleanup (safe):**
   ```sh
   cargo cicd target prune --apply
   # Deletes debug incremental artifacts only
   ```

4. **More aggressive cleanup:**
   ```sh
   # Rebuild from scratch
   cargo cicd target prune --apply
   cargo clean  # Removes entire target/
   cargo build  # Rebuilds everything
   ```

5. **Understand the breakdown:**
   ```sh
   du -sh target/debug/*
   du -sh target/release/*
   # Shows size of each subdirectory
   
   du -sh target/debug/incremental
   du -sh target/debug/.fingerprint
   du -sh target/debug/deps
   ```

### "target prune does not free enough space"

**Problem:** Running prune doesn't free expected space.

**Causes:**
- Release artifacts are protected (never deleted)
- Dependencies are large
- Test artifacts can't be selectively removed

**Solutions:**

1. **Understand what can and can't be deleted:**
   ```sh
   # Can be deleted safely:
   target/debug/incremental
   target/debug/.fingerprint
   target/debug/deps
   
   # NEVER deleted automatically:
   target/release/*
   ```

2. **Delete release artifacts manually if needed:**
   ```sh
   # WARNING: Next release build will be slow
   rm -rf target/release
   ```

3. **Use cargo clean for nuclear option:**
   ```sh
   cargo clean
   # Removes entire target/ directory
   # Rebuilds everything on next cargo build
   ```

4. **Identify large dependencies:**
   ```sh
   cargo tree --depth 1
   # Shows top-level dependencies, sorted by size
   ```

---

## Evidence & Oracle Problems

### "wpm oracle not found"

**Problem:** Commands show "BLOCKED: wpm oracle not found" or "wasm4pm oracle unavailable".

**Causes:**
- wasm4pm is not installed
- wpm binary is not on PATH
- WPM_PATH environment variable not set

**Solutions:**

1. **Install wasm4pm:**
   ```sh
   # Follow installation from: https://github.com/seanchatmangpt/wasm4pm
   # Generally: git clone + cargo install
   ```

2. **Verify wpm is available:**
   ```sh
   wpm --version
   # Should output version number
   ```

3. **Set WPM_PATH if in non-standard location:**
   ```sh
   export WPM_PATH=/path/to/wpm
   cargo cicd evidence doctor
   
   # Persist in ~/.bashrc or ~/.zshrc:
   echo 'export WPM_PATH=/path/to/wpm' >> ~/.bashrc
   ```

4. **Test wpm directly:**
   ```sh
   wpm audit --help
   wpm receipt doctor --help
   # If these fail, wasm4pm may be corrupted
   ```

5. **wpm is optional:**
   - cargo-cicd continues with warnings if wpm not found
   - For validation, install wasm4pm but it's not required for basic operation

### "evidence:audit REFUSE" or "receipt doctor refused"

**Problem:** Oracle refuses the evidence or receipt.

**Causes:**
- Runtime trace doesn't meet oracle's TRUTHFUL threshold
- Receipt format is invalid
- Oracle detects policy violations
- XES fitness score too low

**Solutions:**

1. **Check what was refused:**
   ```sh
   cargo cicd evidence doctor 2>&1 | tee /tmp/oracle-refusal.log
   # Look for error messages from wpm
   ```

2. **Verify evidence was properly emitted:**
   ```sh
   ls -la target/cargo-cicd/evidence/
   # Should contain events.jsonl, events.xes, receipts/
   ```

3. **Validate evidence file format:**
   ```sh
   cat target/cargo-cicd/evidence/events.jsonl | head -1
   # Should be valid JSON (one event per line)
   ```

4. **Re-emit evidence:**
   ```sh
   # Clear and restart fresh trace
   rm -rf target/cargo-cicd/evidence/
   cargo cicd pipeline run
   # This creates new evidence from scratch
   ```

5. **Check oracle configuration:**
   ```sh
   wpm receipt doctor --help
   wpm audit --help
   # Look for any configuration or thresholds
   ```

6. **Contact oracle maintainers:**
   - If refusals persist, report the evidence files for debugging
   - Include the receipt JSON and XES file in bug report

### "no evidence at <path>"

**Problem:** Evidence files are missing or in wrong location.

**Causes:**
- Commands haven't been run yet
- Evidence directory was deleted
- Wrong path being checked

**Solutions:**

1. **Verify evidence directory exists:**
   ```sh
   ls -la target/cargo-cicd/evidence/
   # May not exist if no commands have run yet
   ```

2. **Run commands to generate evidence:**
   ```sh
   cargo cicd status
   cargo cicd target show
   cargo cicd workspace
   # These emit evidence events
   ```

3. **Check evidence file location:**
   ```sh
   ls target/cargo-cicd/evidence/events.*
   # Should show events.jsonl and/or events.xes
   ```

4. **Rebuild evidence:**
   ```sh
   # Clear and run pipeline to regenerate
   rm -rf target/cargo-cicd/evidence/
   cargo cicd pipeline run
   ```

---

## Testing & Fixture Issues

### "test changed" shows no changed tests

**Problem:** `cargo cicd test changed` indicates no affected tests, but you changed test files.

**Causes:**
- Base ref is incorrect (default: origin/main)
- Changed files are not being detected
- Test file naming doesn't match patterns

**Solutions:**

1. **Verify base ref configuration:**
   ```sh
   cat cicd.toml | grep -A 3 "^\[test\]"
   # Should show base ref, default is origin/main
   ```

2. **Check what git considers changed:**
   ```sh
   git diff origin/main --name-only
   # Shows files changed since origin/main
   ```

3. **Ensure remote is up-to-date:**
   ```sh
   git fetch origin
   git diff origin/main --name-only
   # Compare before and after fetch
   ```

4. **Verify test file location:**
   ```sh
   # Test files should be:
   ls -R tests/*.rs
   find tests/ -name "*.rs"
   
   # Or in crates:
   find . -path "**/tests/*.rs"
   ```

5. **Check file detection logic:**
   ```sh
   # cargo-cicd looks for:
   # - Files in tests/ directory
   # - Files matching *_test.rs or *tests.rs pattern
   # - Integration tests directory
   ```

6. **Use full cargo test as fallback:**
   ```sh
   cargo test
   # If test changed is unreliable, run full suite
   ```

### "trybuild changed" detects no changes

**Problem:** `cargo cicd trybuild changed` finds no fixtures despite changes.

**Causes:**
- Fixture files not in `tests/ui/` directory
- Not relative to correct base ref
- Fixture detection pattern doesn't match

**Solutions:**

1. **Verify fixture directory:**
   ```sh
   ls -la tests/ui/
   # Should contain .rs files (compile-fail and compile-pass tests)
   ```

2. **Check what changed relative to main:**
   ```sh
   git diff origin/main -- tests/ui/
   # Shows only changes to fixtures
   ```

3. **Look for correct fixture files:**
   ```sh
   find tests -name "*.rs"
   # Should find files under tests/ui/
   ```

4. **Update snapshots with trybuild:**
   ```sh
   # cargo-cicd only plans; you run trybuild manually
   TRYBUILD=overwrite cargo test --test <test-name>
   ```

### "cargo test" passes but cargo-cicd tests fail

**Problem:** Tests pass when run with `cargo test`, but pipeline reports failures.

**Causes:**
- Environment variables differ
- Working directory context differs
- Test order matters
- Temporary files not cleaned up

**Solutions:**

1. **Run tests in same way as cargo-cicd:**
   ```sh
   # cargo-cicd runs as subprocess with fresh environment
   cargo test -- --test-threads=1
   # Run serially to isolate failures
   ```

2. **Check environment variables:**
   ```sh
   cargo cicd pipeline run 2>&1 | grep -i "env"
   # See what environment variables are set
   ```

3. **Check working directory:**
   ```sh
   cargo cicd test changed
   # This runs from current directory
   # Ensure you're in workspace root
   pwd  # Should show workspace root
   ```

4. **Run pipeline in isolation:**
   ```sh
   # Create temporary directory
   mkdir -p /tmp/test-pipeline
   cd /tmp/test-pipeline
   cp -r /path/to/workspace/* .
   cargo cicd pipeline run
   ```

---

## Advanced Diagnostics

### Capturing full debug output

**Problem:** Need more details for bug reporting.

**Solutions:**

1. **Capture all output:**
   ```sh
   cargo cicd <command> 2>&1 | tee /tmp/cicd-output.log
   cat /tmp/cicd-output.log
   ```

2. **Enable Rust logging:**
   ```sh
   RUST_LOG=trace cargo cicd <command> 2>&1 | tee /tmp/cicd-trace.log
   ```

3. **Inspect evidence files directly:**
   ```sh
   cat target/cargo-cicd/evidence/events.jsonl | jq .
   # Pretty-prints JSON event log
   ```

4. **Check receipt:**
   ```sh
   cat target/cargo-cicd/evidence/receipts/latest.json | jq .
   # Examine process receipt details
   ```

5. **Validate XES file:**
   ```sh
   file target/cargo-cicd/evidence/events.xes
   # Should be XML
   head -20 target/cargo-cicd/evidence/events.xes
   # Should start with XML declaration
   ```

### Checking system prerequisites

**Problem:** Determining if system is properly set up for cargo-cicd.

**Checklist:**

```sh
# 1. Rust is installed
rustc --version
cargo --version

# 2. cargo-cicd is installed
cargo cicd --version

# 3. Git is available
git --version

# 4. Working in a workspace
ls Cargo.toml

# 5. Working in a git repository
git status

# 6. (Optional) wasm4pm is available
wpm --version
```

### Reporting issues

**If you need to report a bug:**

1. **Gather information:**
   ```sh
   cargo cicd --version
   rustc --version
   cargo --version
   git --version
   uname -a
   ```

2. **Reproduce the issue:**
   ```sh
   cd /path/to/workspace
   cargo cicd <failing-command> 2>&1 | tee /tmp/issue.log
   ```

3. **Collect evidence:**
   - Full command output from `/tmp/issue.log`
   - Content of `cicd.toml` (sanitized)
   - Evidence files from `target/cargo-cicd/evidence/`
   - Output of `cargo cicd workspace doctor`

4. **Create minimal reproduction:**
   ```sh
   # If possible, create a minimal workspace that triggers the issue
   # This makes debugging faster
   ```

---

## Quick Reference by Error Message

| Error Message | Noun/Verb | Likely Cause | Solution |
|---------------|-----------|--------------|----------|
| "command not found" | any | Not installed or PATH issue | Reinstall; add ~/.cargo/bin to PATH |
| "not a valid noun" | any | Typo or unknown noun | Check with `cargo cicd --help` |
| "workspace not found" | any | Not in workspace directory | `cd` to directory with Cargo.toml |
| "git repository not found" | git | Not in git repo | `git init` or navigate to repo |
| "FAIL: tree is dirty" | git close | Uncommitted changes exist | `git add` and `git commit` manually |
| "oracle unavailable" | evidence, status audit | wpm not installed | Install wasm4pm or proceed without it |
| "receipt doctor refused" | publish, evidence | Oracle rejected receipt | Re-emit evidence; check oracle logs |
| "target too large" | target | Size exceeds 20 GB | `cargo cicd target prune --apply` |
| "no changed tests" | test changed | Base ref mismatch | Check cicd.toml [test] section |
| "no changed fixtures" | trybuild changed | Base ref mismatch | Ensure tests/ui/ exists; verify base ref |

---

## When All Else Fails

1. **Start fresh:**
   ```sh
   rm -rf target/cargo-cicd
   rm -f cicd.toml
   cargo cicd publish
   ```

2. **Check workspace health:**
   ```sh
   cargo cicd workspace doctor
   ```

3. **Run pipeline with verbosity:**
   ```sh
   cargo cicd pipeline run 2>&1 | tee /tmp/pipeline-output.log
   ```

4. **Consult documentation:**
   - [Quick Start](../tutorials/quick-start.md) — Installation and first steps
   - [Complete Command Reference](COMMANDS.md) — All commands explained
   - [Reference Index](CLI_REFERENCE_INDEX.md) — Common patterns

5. **Report the issue:**
   - GitHub repository: Include full output and evidence files
   - Include output of `cargo cicd workspace doctor`
   - Include `cicd.toml` (sanitized if needed)
   - Specify your OS, Rust version, and cargo-cicd version
