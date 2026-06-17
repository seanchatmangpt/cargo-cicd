# Claude Code Hooks & Settings for cargo-cicd

This guide covers configuring Claude Code (claude.ai/code) for optimal development experience with cargo-cicd. It includes SessionStart hooks, available settings, recommended configurations, real-world examples, and troubleshooting.

---

## Table of Contents

1. [Overview](#overview)
2. [SessionStart Hook](#sessionstart-hook)
3. [Available Settings](#available-settings)
4. [Common Configurations](#common-configurations)
5. [Hook Examples](#hook-examples)
6. [IDE Integration Hooks](#ide-integration-hooks)
7. [Advanced Hook Patterns](#advanced-hook-patterns)
8. [Complete Working Examples](#complete-working-examples)
9. [Troubleshooting Settings](#troubleshooting-settings)
10. [Best Practices](#best-practices)

---

## Overview

Claude Code supports **hooks** and **settings** that enable automated workflows:

- **Hooks**: JSON-configured automation that runs at specific points (SessionStart, before-commit, post-test, etc.)
- **Settings**: Configuration stored in `.claude/settings.json` (project-level) and `~/.claude/settings.json` (user-level)
- **Permissions**: Fine-grained control over which tools and file operations are allowed

### Project Structure

```
cargo-cicd/
├── .claude/
│   ├── settings.json          # Project-level hook & permission config
│   └── settings.local.json    # Local overrides (git-ignored)
├── CLAUDE.md                   # Project guidance for Claude Code
├── Cargo.toml                  # Workspace manifest
├── cicd.toml                   # Cargo-cicd workspace state (auto-generated)
└── crates/
    ├── cargo-cicd/
    ├── cargo-cicd-core/
    └── cargo-cicd-lsp/
```

### File Hierarchy & Precedence

Settings are merged in this order (later overrides earlier):
1. User defaults (`~/.claude/settings.json`)
2. Project settings (`.claude/settings.json`)
3. Project local settings (`.claude/settings.local.json`) — not committed
4. Environment variables

---

## SessionStart Hook

The **SessionStart hook** configures Claude Code to automatically set up the project when you start a web session (claude.ai/code). This is the most important hook for cargo-cicd.

### Purpose

Ensure the workspace is ready for development in a fresh Claude Code session by:
- Installing Rust toolchain
- Installing cargo-make (for build tasks)
- Enabling feature flags
- Running linter/checker to surface issues
- Setting environment variables

### Basic SessionStart Configuration

Create `.claude/settings.json` in the project root:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Initialize cargo-cicd workspace for Claude Code web session",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "rustup update stable && rustup component add rustfmt clippy"],
          "description": "Update Rust toolchain"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo install cargo-make"],
          "description": "Install cargo-make for build automation"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check"],
          "description": "Lint and type-check without building"
        }
      ]
    }
  }
}
```

### SessionStart with Feature Flags

If working with autonomic policies or evidence gates:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Initialize cargo-cicd with advanced features",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "rustup update stable"],
          "description": "Update Rust toolchain"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check --features autonomic"],
          "description": "Check with autonomic policies enabled"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants --features process-data"],
          "description": "Verify core invariants"
        }
      ]
    }
  }
}
```

### SessionStart with Environment Setup

For teams using environment-specific tooling:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Full web session setup with dependencies",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "if ! command -v cargo-make &> /dev/null; then cargo install cargo-make; fi"],
          "description": "Install cargo-make if needed"
        },
        {
          "command": "bash",
          "args": ["-c", "rustup toolchain install stable && rustup default stable"],
          "description": "Ensure stable Rust is available"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check 2>&1"],
          "description": "Run linter and type-checker"
        }
      ]
    }
  },
  "env": {
    "RUST_BACKTRACE": "1",
    "CARGO_INCREMENTAL": "0"
  }
}
```

### Validation

After setting up SessionStart, you can test it locally:

```bash
# Simulate what Claude Code will do on web session start
bash -c "rustup update stable && rustup component add rustfmt clippy"
cargo install cargo-make
cargo make check
```

---

## Available Settings

### Top-Level Structure

```json
{
  "permissions": {},        // Fine-grained tool/file permissions
  "env": {},               // Environment variables
  "features": [],          // Cargo feature flags to enable by default
  "hooks": {},             // Automation hooks (SessionStart, pre-commit, etc.)
  "ignorePatterns": []     // File patterns Claude Code should not read
}
```

### Permissions

Permissions control which tools Claude Code can invoke and which files it can modify. They are organized by tool or operation.

#### Bash Permissions

Allow specific bash commands or command patterns:

```json
{
  "permissions": {
    "bash": {
      "allowlist": [
        "cargo build",
        "cargo test",
        "cargo make check",
        "cargo make lint",
        "cargo cicd status",
        "git status",
        "git log",
        "rustup update"
      ],
      "blockingPatterns": [
        "rm -rf /",
        "git push --force",
        "cargo clean && cargo build"
      ]
    }
  }
}
```

#### File Read/Write Permissions

Allow Claude to read and write specific file patterns:

```json
{
  "permissions": {
    "files": {
      "read": [
        "Cargo.toml",
        "src/**/*.rs",
        "tests/**/*.rs",
        "CLAUDE.md",
        "cicd.toml"
      ],
      "write": [
        "src/**/*.rs",
        "tests/**/*.rs",
        "Cargo.toml"
      ],
      "blocked": [
        ".git/**",
        "target/**",
        ".env",
        ".env.local",
        "**/*.lock"
      ]
    }
  }
}
```

#### MCP Tool Permissions

If using Claude Code's MCP (Model Context Protocol) integrations:

```json
{
  "permissions": {
    "mcp": {
      "enabled": ["github", "git"],
      "disabled": ["slack", "gmail"]
    }
  }
}
```

### Environment Variables

Set environment variables for Claude Code sessions:

```json
{
  "env": {
    "RUST_BACKTRACE": "1",
    "CARGO_INCREMENTAL": "0",
    "RUST_LOG": "warn",
    "CARGO_TARGET_DIR": "target",
    "CARGO_NET_RETRY": "3"
  }
}
```

#### Feature Flags as Environment

For LSP and advanced features:

```json
{
  "env": {
    "CARGO_FEATURE_AUTONOMIC": "1",
    "CARGO_FEATURE_PROCESS_DATA": "1",
    "CARGO_FEATURE_WASM4PM": "0"
  }
}
```

---

## Common Configurations

### Minimal Setup (New Web Session)

For first-time Claude Code web users, a minimal SessionStart:

```json
{
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "cargo",
          "args": ["build", "--release"],
          "description": "Build cargo-cicd in release mode"
        }
      ]
    }
  },
  "env": {
    "RUST_BACKTRACE": "1"
  }
}
```

### Development Setup (Local Iteration)

For developers making frequent changes:

```json
{
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo install cargo-make"],
          "description": "Install cargo-make"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check"],
          "description": "Lint and type-check"
        }
      ]
    }
  },
  "permissions": {
    "bash": {
      "allowlist": [
        "cargo test",
        "cargo make",
        "cargo fmt",
        "cargo clippy",
        "cargo cicd",
        "git status",
        "git diff"
      ]
    }
  },
  "env": {
    "CARGO_INCREMENTAL": "1",
    "RUST_BACKTRACE": "1"
  }
}
```

### Release/Testing Setup

When preparing a release and validating feature gates:

```json
{
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo make test"],
          "description": "Run all tests"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants"],
          "description": "Validate public boundary invariants"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test feature_projection"],
          "description": "Verify feature flag contract"
        }
      ]
    }
  },
  "env": {
    "RUST_BACKTRACE": "full",
    "CARGO_INCREMENTAL": "0"
  }
}
```

### LSP Development Setup

When working on the LSP (cargo-cicd-lsp) crate:

```json
{
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cd crates/cargo-cicd-lsp && cargo build"],
          "description": "Build LSP server"
        },
        {
          "command": "bash",
          "args": ["-c", "cd crates/cargo-cicd-lsp && cargo test"],
          "description": "Test LSP diagnostics"
        }
      ]
    }
  }
}
```

### Autonomic Policies Setup

When testing autonomic policy recommendations:

```json
{
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --features autonomic"],
          "description": "Test with autonomic policies enabled"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo cicd status"],
          "description": "Show current workspace status with policy verdicts"
        }
      ]
    }
  },
  "env": {
    "CARGO_FEATURE_AUTONOMIC": "1"
  }
}
```

---

## Hook Examples

### Before-Commit Linting Hook

Automatically check and format code before committing:

```json
{
  "hooks": {
    "before-commit": {
      "description": "Lint and format before git commit",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo fmt --all -- --check"],
          "description": "Check code formatting",
          "failureStrategy": "warn"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo clippy --all-targets -- -D warnings"],
          "description": "Run clippy linter",
          "failureStrategy": "warn"
        }
      ]
    }
  }
}
```

### Post-Test Summary Hook

Generate a summary after test runs:

```json
{
  "hooks": {
    "post-test": {
      "description": "Summarize test results",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test -- --nocapture 2>&1 | tail -20"],
          "description": "Show last 20 lines of test output"
        },
        {
          "command": "bash",
          "args": ["-c", "echo 'Test run complete. Review cicd.toml events:' && tail -5 cicd.toml"],
          "description": "Show latest events"
        }
      ]
    }
  }
}
```

### Pre-Push Validation Hook

Validate workspace health before pushing to remote:

```json
{
  "hooks": {
    "pre-push": {
      "description": "Validate workspace before push",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants"],
          "description": "Verify public boundary invariants"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo cicd workspace doctor"],
          "description": "Run workspace health check"
        },
        {
          "command": "bash",
          "args": ["-c", "git diff origin/main --name-only | wc -l"],
          "description": "Show number of changed files"
        }
      ]
    }
  }
}
```

### Changed-Tests Hook

Run only tests for modified crates:

```json
{
  "hooks": {
    "on-file-change": {
      "description": "Run changed tests on file modification",
      "filePatterns": ["src/**/*.rs", "tests/**/*.rs"],
      "debounceMs": 1000,
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo cicd test changed"],
          "description": "Run tests for changed crates"
        }
      ]
    }
  }
}
```

### Workspace Doctor Hook

Periodic workspace diagnostics:

```json
{
  "hooks": {
    "hourly": {
      "description": "Check workspace health every hour",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo cicd workspace doctor"],
          "description": "Full workspace diagnostics"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo cicd target show"],
          "description": "Check target directory size"
        }
      ]
    }
  }
}
```

### Feature Projection Validation

Verify feature flags before release:

```json
{
  "hooks": {
    "before-release": {
      "description": "Validate all feature combinations",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --test feature_projection"],
          "description": "Verify feature flag contracts"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --no-default-features"],
          "description": "Test with no features"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --all-features"],
          "description": "Test with all features enabled"
        }
      ]
    }
  }
}
```

---

## IDE Integration Hooks

### VS Code Integration

For developers using VS Code with the Rust Analyzer extension, configure Claude Code to sync with the IDE:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "VS Code workspace setup",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "command -v cargo-make || cargo install cargo-make"],
          "description": "Ensure cargo-make is available"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check"],
          "description": "Prime Rust Analyzer with type-check"
        }
      ]
    },
    "before-commit": {
      "description": "Format and lint before VS Code commit",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo fmt --all"],
          "description": "Auto-format on save"
        }
      ]
    }
  },
  "env": {
    "RUST_ANALYZER_DEBUG": "false",
    "RA_LOG": "off"
  }
}
```

### Neovim/Helix Integration

For terminal-based editors, disable heavy background tasks in SessionStart:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Lightweight setup for terminal editors",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "rustup update stable"],
          "description": "Update Rust (quick check)"
        }
      ]
    }
  },
  "env": {
    "RUST_BACKTRACE": "1"
  }
}
```

### GitHub Codespaces Integration

For cloud-based development environments:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Codespaces-optimized setup",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "apt-get update -qq && apt-get install -y cargo 2>/dev/null || true"],
          "description": "Ensure Rust is available (if not pre-installed)"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo install cargo-make --quiet"],
          "description": "Install cargo-make quietly"
        },
        {
          "command": "bash",
          "args": ["-c", "timeout 300 cargo make check || true"],
          "description": "Check with 5-minute timeout (Codespaces slower)"
        }
      ]
    }
  },
  "env": {
    "CARGO_INCREMENTAL": "1",
    "CARGO_BUILD_JOBS": "2"
  }
}
```

---

## Advanced Hook Patterns

### Conditional Hook Execution

Run hooks only in specific environments:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Environment-aware setup",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "if [[ \"$CI\" != \"true\" ]]; then cargo make check; fi"],
          "description": "Skip check in CI environments"
        },
        {
          "command": "bash",
          "args": ["-c", "if [[ -n \"$GITHUB_ACTIONS\" ]]; then cargo test --test invariants; fi"],
          "description": "Run tests only in GitHub Actions"
        }
      ]
    }
  }
}
```

### Parallel Command Execution

Run independent commands concurrently:

```json
{
  "hooks": {
    "SessionStart": {
      "description": "Parallel workspace setup",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo fmt --all & cargo clippy --all -- -D warnings & wait"],
          "description": "Format and lint in parallel"
        }
      ]
    }
  }
}
```

### Deferred Hooks (Fire-and-Forget)

Run background tasks without blocking:

```json
{
  "hooks": {
    "post-test": {
      "description": "Background telemetry (non-blocking)",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "nohup cargo make test &> /tmp/background-test.log &"],
          "description": "Queue test run for later"
        }
      ]
    }
  }
}
```

### Hook Chaining with Failure Strategies

Control behavior when commands fail:

```json
{
  "hooks": {
    "pre-push": {
      "description": "Pre-push validation with fallback",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants"],
          "description": "Run invariants",
          "failureStrategy": "fail",
          "failureMessage": "Invariants failed; push blocked"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test feature_projection"],
          "description": "Run feature tests",
          "failureStrategy": "warn",
          "failureMessage": "Feature tests failed; push allowed with warning"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo fmt --all"],
          "description": "Auto-format",
          "failureStrategy": "continue",
          "failureMessage": "Format failed; continuing anyway"
        }
      ]
    }
  }
}
```

### Scheduled Hook Examples

Run tasks on intervals:

```json
{
  "hooks": {
    "hourly": {
      "description": "Hourly workspace diagnostics",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo cicd target show"],
          "description": "Check target directory size hourly"
        }
      ]
    },
    "daily": {
      "description": "Daily workspace cleanup",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo clean && cargo build --release 2>&1 | head -10"],
          "description": "Nightly clean rebuild"
        }
      ]
    }
  }
}
```

### File Watch Hooks

React to file changes intelligently:

```json
{
  "hooks": {
    "on-file-change": {
      "description": "Run tests on Rust file change",
      "filePatterns": ["src/**/*.rs", "tests/**/*.rs"],
      "debounceMs": 2000,
      "maxConcurrent": 1,
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --lib"],
          "description": "Run unit tests on change"
        }
      ]
    }
  }
}
```

---

## Complete Working Examples

### Example 1: Full Development Setup

Complete `.claude/settings.json` for active cargo-cicd development:

```json
{
  "permissions": {
    "bash": {
      "allowlist": [
        "cargo build",
        "cargo build --release",
        "cargo test",
        "cargo test --all-features",
        "cargo test --no-default-features",
        "cargo test --test *",
        "cargo make *",
        "cargo fmt *",
        "cargo clippy *",
        "cargo cicd *",
        "git status",
        "git log",
        "git diff",
        "git diff origin/main",
        "rustup update",
        "rustup component add",
        "rustup toolchain install",
        "cargo install cargo-make",
        "cargo metadata --format-version 1"
      ],
      "blockingPatterns": [
        "rm -rf /",
        "git push --force",
        "git reset --hard",
        "sudo"
      ]
    },
    "files": {
      "read": [
        "**/*.rs",
        "**/*.toml",
        "**/*.md",
        "CLAUDE.md",
        ".claude/**",
        "docs/**",
        "src/**",
        "tests/**",
        "crates/**"
      ],
      "write": [
        "src/**/*.rs",
        "tests/**/*.rs",
        "Cargo.toml",
        "crates/**/Cargo.toml",
        ".claude/settings.json",
        "docs/**"
      ],
      "blocked": [
        ".git/**",
        ".git",
        "target/**",
        "**/*.lock",
        ".env*",
        "credentials*"
      ]
    }
  },
  "env": {
    "RUST_BACKTRACE": "1",
    "CARGO_INCREMENTAL": "1",
    "RUST_LOG": "warn",
    "CARGO_NET_RETRY": "3"
  },
  "hooks": {
    "SessionStart": {
      "description": "Full cargo-cicd development environment",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "rustup update stable && rustup component add rustfmt clippy"],
          "description": "Update Rust toolchain",
          "timeout": 300000
        },
        {
          "command": "bash",
          "args": ["-c", "command -v cargo-make &>/dev/null || cargo install cargo-make"],
          "description": "Ensure cargo-make is installed",
          "timeout": 600000
        },
        {
          "command": "bash",
          "args": ["-c", "cargo make check"],
          "description": "Type-check and lint",
          "timeout": 180000
        }
      ]
    },
    "before-commit": {
      "description": "Format and lint before commit",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo fmt --all"],
          "description": "Auto-format code",
          "failureStrategy": "warn"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo clippy --all -- -D warnings"],
          "description": "Run clippy linter",
          "failureStrategy": "warn"
        }
      ]
    },
    "pre-push": {
      "description": "Validate before push",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants"],
          "description": "Verify public boundary invariants",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test feature_projection"],
          "description": "Verify feature flag contracts",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo cicd workspace doctor"],
          "description": "Workspace health check",
          "failureStrategy": "warn"
        }
      ]
    }
  }
}
```

### Example 2: CI/CD Pipeline Setup

For continuous integration environments:

```json
{
  "env": {
    "CI": "true",
    "RUST_BACKTRACE": "full",
    "CARGO_INCREMENTAL": "0"
  },
  "hooks": {
    "SessionStart": {
      "description": "CI pipeline initialization",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "rustup update stable"],
          "description": "Update toolchain",
          "timeout": 300000
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --all-features --no-fail-fast 2>&1 | tee test-output.log"],
          "description": "Run all tests",
          "timeout": 1800000
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants --test feature_projection"],
          "description": "Run critical tests",
          "failureStrategy": "fail"
        }
      ]
    }
  },
  "permissions": {
    "bash": {
      "allowlist": [
        "cargo build",
        "cargo test",
        "rustup update",
        "cargo metadata"
      ]
    }
  }
}
```

### Example 3: Release Validation Setup

For pre-release checks:

```json
{
  "hooks": {
    "before-release": {
      "description": "Pre-release validation checklist",
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "cargo test --no-default-features"],
          "description": "Test with no features",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --all-features"],
          "description": "Test with all features",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --features autonomic"],
          "description": "Test autonomic mode",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --features wasm4pm"],
          "description": "Test wasm4pm integration",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo test --test invariants --test feature_projection --test cli"],
          "description": "Run acceptance tests",
          "failureStrategy": "fail"
        },
        {
          "command": "bash",
          "args": ["-c", "cargo build --release"],
          "description": "Build release binary",
          "failureStrategy": "fail",
          "timeout": 600000
        }
      ]
    }
  }
}
```

---

## Troubleshooting Settings

### Common Issues & Solutions

#### Problem: "SessionStart hook doesn't run"

**Cause**: Hook is misconfigured or not recognized by Claude Code.

**Solution**:
1. Verify `.claude/settings.json` exists and is valid JSON:
   ```bash
   cat .claude/settings.json | jq .
   ```

2. Check that `SessionStart` is spelled correctly (case-sensitive):
   ```json
   {
     "hooks": {
       "SessionStart": {}  // Correct capitalization
     }
   }
   ```

3. Ensure hook commands are in the allowlist. Add to permissions:
   ```json
   {
     "permissions": {
       "bash": {
         "allowlist": [
           "cargo install cargo-make",
           "cargo make check",
           "rustup update stable"
         ]
       }
     }
   }
   ```

4. Test locally by running commands manually:
   ```bash
   rustup update stable
   cargo install cargo-make
   cargo make check
   ```

#### Problem: "Permission denied" errors during hook execution

**Cause**: Commands are not in the allowlist, or permissions are too restrictive.

**Solution**:

1. Check `.claude/settings.json` and `.claude/settings.local.json`:
   ```bash
   ls -la .claude/
   cat .claude/settings.json
   ```

2. Add the failing command to `permissions.bash.allowlist`:
   ```json
   {
     "permissions": {
       "bash": {
         "allowlist": [
           "the exact command that failed"
         ]
       }
     }
   }
   ```

3. Use environment-based allowlists for complex commands:
   ```json
   {
     "permissions": {
       "bash": {
         "allowlist": [
           "cargo *",
           "rustup *",
           "git status"
         ]
       }
     }
   }
   ```

#### Problem: "Hook runs but doesn't complete"

**Cause**: Commands are timing out or hanging.

**Solution**:

1. Check command duration:
   ```bash
   time cargo make check
   ```

2. Add timeout to hook:
   ```json
   {
     "hooks": {
       "SessionStart": {
         "commands": [
           {
             "command": "cargo",
             "args": ["make", "check"],
             "timeout": 300000,
             "description": "Check with 5-minute timeout"
           }
         ]
       }
     }
   }
   ```

3. Use simpler commands in SessionStart:
   ```json
   {
     "hooks": {
       "SessionStart": {
         "commands": [
           {
             "command": "bash",
             "args": ["-c", "cargo --version"],
             "description": "Quick check that cargo works"
           }
         ]
       }
     }
   }
   ```

#### Problem: "Settings don't apply to web session"

**Cause**: Local settings (`.claude/settings.local.json`) aren't synced, or settings aren't committed to remote.

**Solution**:

1. Ensure `.claude/settings.json` is committed:
   ```bash
   git add .claude/settings.json
   git commit -m "chore: configure Claude Code hooks for web session"
   ```

2. Never commit `.claude/settings.local.json` (add to `.gitignore`):
   ```bash
   echo ".claude/settings.local.json" >> .gitignore
   git add .gitignore && git commit -m "chore: ignore local Claude Code settings"
   ```

3. Push to remote and pull in web session:
   ```bash
   git push origin main
   ```

4. Verify settings are readable:
   ```bash
   jq . .claude/settings.json
   ```

#### Problem: "Hooks interfere with each other"

**Cause**: Multiple hooks are running simultaneously or in unexpected order.

**Solution**:

1. **Hook Execution Order**:
   - SessionStart runs once at session start
   - before-commit runs before git commit
   - pre-push runs before git push
   - on-file-change runs when files matching patterns change
   - hourly/daily/weekly run on schedule

2. **Prevent Conflicts**:
   ```json
   {
     "hooks": {
       "SessionStart": {
         "commands": [
           {
             "command": "bash",
             "args": ["-c", "flock .claude-lock cargo make check"],
             "description": "Use file lock to prevent parallel execution"
           }
         ]
       }
     }
   }
   ```

3. **Use Failure Strategy**:
   ```json
   {
     "hooks": {
       "SessionStart": {
         "commands": [
           {
             "command": "cargo",
             "args": ["make", "check"],
             "failureStrategy": "continue",
             "description": "Continue even if this fails"
           }
         ]
       }
     }
   }
   ```

#### Problem: "Feature flags not recognized during tests"

**Cause**: Feature flags aren't enabled in test environment.

**Solution**:

1. Enable features in hook:
   ```json
   {
     "hooks": {
       "SessionStart": {
         "commands": [
           {
             "command": "bash",
             "args": ["-c", "cargo test --features autonomic,process-data"],
             "description": "Test with features enabled"
           }
         ]
       }
     }
   }
   ```

2. Set feature environment variables:
   ```json
   {
     "env": {
       "CARGO_FEATURE_AUTONOMIC": "1",
       "CARGO_FEATURE_PROCESS_DATA": "1"
     }
   }
   ```

3. Verify feature gates in Cargo.toml:
   ```bash
   cargo check --features autonomic
   cargo test --test feature_projection
   ```

### Hook Execution Order & Precedence

Understanding when hooks run helps predict behavior:

| Hook | When | Runs | Example |
|------|------|------|---------|
| `SessionStart` | Web session begins | Once | `cargo make check` |
| `before-commit` | `git commit` invoked | Before commit | Linting, formatting |
| `pre-push` | `git push` invoked | Before push | Test suite, validation |
| `post-test` | Tests complete | After test run | Summary, report generation |
| `on-file-change` | Files matching pattern change | On each change | Incremental rebuild, lint |
| `hourly` | Every hour | Periodic | Health checks, cleanup |

### Permission Precedence

Permissions are evaluated in this order (first match wins):

1. **Blocking patterns** (most restrictive):
   ```json
   { "bash": { "blockingPatterns": ["rm -rf /", "git push --force"] } }
   ```

2. **Allowlist** (whitelist approach):
   ```json
   { "bash": { "allowlist": ["cargo test", "cargo build"] } }
   ```

3. **Glob patterns** (file-based):
   ```json
   { "files": { "write": ["src/**/*.rs"], "blocked": ["**/*.lock"] } }
   ```

4. **User defaults** (fallback):
   - Check `~/.claude/settings.json` for inherited permissions

### Debugging Settings

Enable verbose logging to understand what's happening:

```json
{
  "debug": true,
  "logLevel": "debug",
  "hooks": {
    "SessionStart": {
      "commands": [
        {
          "command": "bash",
          "args": ["-c", "set -x; cargo make check"],
          "description": "Run with verbose shell output"
        }
      ]
    }
  }
}
```

View logs:
```bash
# Check Claude Code logs (location depends on platform)
# macOS
tail -f ~/Library/Logs/claude-code.log

# Linux
tail -f ~/.cache/claude-code/logs/claude-code.log

# Windows
tail -f %APPDATA%\Claude Code\logs\claude-code.log
```

### Schema Validation

Validate `.claude/settings.json` structure:

```bash
# Using jq
jq . .claude/settings.json

# Or with Python
python3 -m json.tool .claude/settings.json | head -20
```

Expected top-level keys:
```json
{
  "permissions": { "bash": {}, "files": {}, "mcp": {} },
  "env": {},
  "hooks": {
    "SessionStart": {},
    "before-commit": {},
    "pre-push": {}
  },
  "ignorePatterns": [],
  "debug": false
}
```

---

## Best Practices

### General Guidelines

1. **Start simple**: Use minimal SessionStart first, expand as needed. Add complexity incrementally.
2. **Test locally**: Always run hook commands manually before adding to config. Verify behavior before committing.
3. **Commit carefully**: Only commit `.claude/settings.json`, not `.local` variants. Add `.claude/settings.local.json` to `.gitignore`.
4. **Use timeouts**: Prevent infinite waits with reasonable timeout values (prefer 5-10 minutes for heavy tasks).
5. **Document intentions**: Add clear descriptions to every hook command. Future maintainers will thank you.
6. **Monitor permissions**: Review allowlists quarterly; avoid overly broad wildcards like `cargo *`.
7. **Version-pin tools**: Specify exact versions for reproducible setups (e.g., `cargo install cargo-make@0.36.0`).

### Performance Optimization

- **Minimize SessionStart overhead**: Keep SessionStart commands under 60 seconds if possible. Lazy-load heavy dependencies.
- **Fail fast on critical checks**: Use `failureStrategy: "fail"` for invariants and safety checks; use `"warn"` for quality checks.
- **Parallelize where safe**: Use `&` and `wait` to run independent commands concurrently (e.g., format + lint).
- **Cache dependencies**: Avoid re-downloading tools in every SessionStart; check if already installed first.

### Security Considerations

- **Minimal permissions**: Only allowlist commands you actually need. Avoid `cargo *` wildcards.
- **Block destructive patterns**: Always include blocking patterns for `rm -rf`, `git push --force`, `git reset --hard`.
- **Protect sensitive files**: Block `.env*`, `credentials*`, `.git/**` in file permissions.
- **Audit MCP integrations**: Only enable MCP tools (GitHub, Git) that are explicitly needed; disable others.
- **Environment variable caution**: Avoid leaking secrets in `env` section. Use `.claude/settings.local.json` for local overrides.

### Maintenance & Troubleshooting

- **Use verbose output**: Add `set -x` to bash commands during debugging; remove for production.
- **Leverage `.local` overrides**: Use `.claude/settings.local.json` for environment-specific tweaks without committing.
- **Version control hook changes**: Treat `.claude/settings.json` as you would code; document changes in commit messages.
- **Monitor execution time**: Periodically review hook execution times; optimize or move long-running tasks to scheduled hooks.
- **Test on multiple platforms**: If team members use macOS, Linux, and Windows, test hooks on all three.

### cargo-cicd-Specific Best Practices

- **Always enable invariants**: Include `cargo test --test invariants` in pre-push hooks to catch public boundary violations early.
- **Feature flag validation**: Test with `--no-default-features`, `--all-features`, and specific flags (`--features autonomic`, etc.).
- **CLAUDE.md alignment**: Keep hook commands aligned with build/test commands documented in `CLAUDE.md`.
- **cicd.toml awareness**: Remember that `cicd.toml` is auto-generated; don't commit it, but do review its events section during debugging.
- **Evidence validation**: If using wasm4pm features, include evidence-gate tests in pre-push hooks.

---

## Settings Schema Reference

This section provides a comprehensive JSON schema for `.claude/settings.json`.

### Root Settings Object

```json
{
  "permissions": {
    "bash": {
      "allowlist": ["string[]"],
      "blockingPatterns": ["string[]"]
    },
    "files": {
      "read": ["string[]"],
      "write": ["string[]"],
      "blocked": ["string[]"]
    },
    "mcp": {
      "enabled": ["string[]"],
      "disabled": ["string[]"]
    }
  },
  "env": {
    "KEY": "value"
  },
  "features": ["string[]"],
  "hooks": {
    "SessionStart": { "commands": [] },
    "before-commit": { "commands": [] },
    "pre-push": { "commands": [] },
    "post-test": { "commands": [] },
    "on-file-change": { "filePatterns": [], "commands": [] },
    "hourly": { "commands": [] },
    "daily": { "commands": [] }
  },
  "ignorePatterns": ["string[]"],
  "debug": "boolean",
  "logLevel": "debug|info|warn|error"
}
```

### Hook Command Object

Each command in a hook's `commands` array has this structure:

```json
{
  "command": "bash|cargo|git",
  "args": ["arg1", "arg2"],
  "description": "Human-readable description",
  "timeout": 60000,
  "failureStrategy": "fail|warn|continue",
  "failureMessage": "Custom error message (optional)",
  "workingDirectory": "/path/to/dir (optional)",
  "env": {
    "VARIABLE": "value"
  }
}
```

### Detailed Field Descriptions

#### `permissions.bash.allowlist`

Commands that Claude Code is allowed to run. Supports exact matches and wildcards.

```json
{
  "allowlist": [
    "cargo build",           // Exact match
    "cargo test *",          // Wildcard (ends with test)
    "cargo *",               // All cargo commands
    "git status",
    "rustup update"
  ]
}
```

#### `permissions.bash.blockingPatterns`

Patterns that will always be blocked, overriding allowlist. Use for safety.

```json
{
  "blockingPatterns": [
    "rm -rf /",
    "git push --force",
    "git reset --hard",
    "*sudo*",
    "*rm -rf*"
  ]
}
```

#### `permissions.files.read|write|blocked`

File patterns controlling which files Claude Code can access.

```json
{
  "read": [
    "src/**/*.rs",           // Read all Rust files in src/
    "**/*.toml",             // All TOML files
    "CLAUDE.md",             // Specific file
    "docs/**"                // All files in docs/
  ],
  "write": [
    "src/**/*.rs",           // Allow write to source files
    "Cargo.toml"             // Allow editing manifest
  ],
  "blocked": [
    ".git/**",               // Always block git internals
    "target/**",             // Block build output
    "**/*.lock",             // Block lock files
    ".env*"                  // Block environment files
  ]
}
```

#### `env`

Environment variables available to all hooks and Claude Code sessions.

```json
{
  "env": {
    "RUST_BACKTRACE": "1",
    "CARGO_INCREMENTAL": "1",
    "RUST_LOG": "warn",
    "CARGO_NET_RETRY": "3"
  }
}
```

**Common values for cargo-cicd:**

| Variable | Value | Purpose |
|----------|-------|---------|
| `RUST_BACKTRACE` | `1` or `full` | Show full error traces |
| `CARGO_INCREMENTAL` | `0` or `1` | Incremental compilation (1=faster, 0=cleaner) |
| `RUST_LOG` | `debug\|info\|warn\|error` | Logging level |
| `CARGO_NET_RETRY` | `1-10` | Retry failed downloads |
| `CARGO_INCREMENTAL` | `0` | Disable in CI for clean builds |
| `CARGO_FEATURE_AUTONOMIC` | `1` | Enable autonomic features at compile time |
| `CARGO_FEATURE_PROCESS_DATA` | `1` | Enable process-data features |

#### `hooks`

All available hook types:

| Hook | When | Example Use |
|------|------|-------------|
| `SessionStart` | Web session begins | Install dependencies, run linter |
| `before-commit` | Before `git commit` | Format, lint |
| `pre-push` | Before `git push` | Tests, validation |
| `post-test` | After tests complete | Summary, reporting |
| `on-file-change` | Files matching pattern change | Incremental build, lint |
| `hourly` | Every hour (if session active) | Background checks |
| `daily` | Every day (if session active) | Cleanup, optimization |

#### `hooks[hook].commands[].failureStrategy`

Controls behavior when a command fails:

- **`fail`**: Stop execution, fail the entire hook. Use for critical checks.
- **`warn`**: Log warning, continue to next command. Use for quality checks.
- **`continue`**: Silent failure, continue. Use for optional commands.

```json
{
  "commands": [
    {
      "command": "cargo",
      "args": ["test"],
      "failureStrategy": "fail",
      "failureMessage": "Tests failed; push blocked"
    }
  ]
}
```

### Complete Schema Validation

Validate your settings file structure:

```bash
# Using jq (install with apt-get install jq, brew install jq)
jq . .claude/settings.json

# Check for common errors
jq 'keys' .claude/settings.json  # Show top-level keys

# Validate hook structure
jq '.hooks.SessionStart.commands | length' .claude/settings.json
```

### Schema Evolution

Future versions of Claude Code may add new fields. To stay future-compatible:

1. **Use optional fields**: Don't require unknown fields in your schema checks.
2. **Version your config**: Add a comment noting which Claude Code version the config targets.
3. **Test compatibility**: Periodically test your config on the latest Claude Code version.

---

## See Also

- **CLAUDE.md** — Project guidance for Claude Code in this repository
- **docs/lsp/EDITOR_INTEGRATION.md** — LSP configuration for editors (VS Code, Neovim, Helix)
- **Cargo.toml** — Workspace and feature flag definitions
- **cicd.toml** — Workspace state and policy configuration
- **tests/** — Integration test examples for validation hooks

---

## Questions?

For issues with Claude Code hooks and settings:
1. Check the troubleshooting section above
2. Review project-specific guidance in `CLAUDE.md`
3. Validate JSON syntax in `.claude/settings.json`
4. Test commands manually before adding to hooks
