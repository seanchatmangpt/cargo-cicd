# Claude Code Hooks & Settings for cargo-cicd

This guide covers configuring Claude Code (claude.ai/code) for optimal development experience with cargo-cicd. It includes SessionStart hooks, available settings, recommended configurations, real-world examples, and troubleshooting.

---

## Table of Contents

1. [Overview](#overview)
2. [SessionStart Hook](#sessionstart-hook)
3. [Available Settings](#available-settings)
4. [Common Configurations](#common-configurations)
5. [Hook Examples](#hook-examples)
6. [Troubleshooting Settings](#troubleshooting-settings)

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

1. **Start simple**: Use minimal SessionStart first, expand as needed.
2. **Test locally**: Always run hook commands manually before adding to config.
3. **Commit carefully**: Only commit `.claude/settings.json`, not `.local` variants.
4. **Use timeouts**: Prevent infinite waits with reasonable timeout values.
5. **Document intentions**: Add clear descriptions to every hook command.
6. **Monitor permissions**: Review allowlists quarterly; avoid overly broad wildcards.
7. **Version-pin tools**: Specify exact versions for reproducible setups (e.g., `cargo install cargo-make@0.36.0`).

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
