# IDE Integration for cargo-cicd

Detailed integration guides for popular IDEs and editors.

**Version:** 26.6.19

## Table of Contents

1. [VS Code](#vs-code)
2. [JetBrains IDEs (IntelliJ, CLion, etc.)](#jetbrains-ides)
3. [Vim/Neovim](#vimneovim)
4. [Emacs](#emacs)
5. [Sublime Text](#sublime-text)
6. [General Editor Integration](#general-editor-integration)

---

## VS Code

### Setup

1. Ensure cargo-cicd is installed:
   ```bash
   cargo install cargo-cicd
   ```

2. Open your workspace in VS Code:
   ```bash
   code /path/to/workspace
   ```

3. Create `.vscode/tasks.json` with cargo-cicd commands (see below)

### Tasks Configuration

Create `.vscode/tasks.json` to define cargo-cicd tasks:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo-cicd: Status",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "status"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "cargo-cicd: Workspace Doctor",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "workspace"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true,
        "panel": "shared"
      }
    },
    {
      "label": "cargo-cicd: Test Changed",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "test", "changed"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "cargo-cicd: Git Status",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "git", "status"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false
      }
    },
    {
      "label": "cargo-cicd: Target Show",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "target", "show"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false
      }
    },
    {
      "label": "cargo-cicd: Target Prune (dry-run)",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "target", "prune"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    },
    {
      "label": "cargo-cicd: Target Prune (apply)",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "target", "prune", "--apply"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    },
    {
      "label": "cargo-cicd: Publish",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "publish"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    },
    {
      "label": "cargo-cicd: Full Pipeline",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "pipeline", "run"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true,
        "panel": "new"
      }
    },
    {
      "label": "cargo-cicd: Evidence Doctor",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "evidence", "doctor"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    }
  ]
}
```

### Keyboard Shortcuts

Create `.vscode/keybindings.json` to bind tasks to keyboard shortcuts:

```json
[
  {
    "key": "ctrl+shift+c",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Status"
  },
  {
    "key": "ctrl+shift+w",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Workspace Doctor"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Test Changed"
  },
  {
    "key": "ctrl+shift+g",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Git Status"
  },
  {
    "key": "ctrl+shift+p",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Full Pipeline"
  }
]
```

Add these to your keybindings file:
1. Open Command Palette: `Ctrl+Shift+P`
2. Run: "Preferences: Open Keyboard Shortcuts (JSON)"
3. Paste the above key bindings

### Task Shortcuts in VS Code

Alternatively, run tasks via Command Palette without keyboard shortcuts:

1. Press `Ctrl+Shift+P` to open Command Palette
2. Type "Tasks: Run Task"
3. Select the cargo-cicd task you want to run

### Status Bar Button

Add a button to VS Code status bar to run cargo-cicd:

Create `.vscode/extensions.json` to recommend extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "serayuzgur.crates",
    "tamasfe.even-better-toml"
  ]
}
```

Then create a custom extension or use the Command Palette for quick access.

### Watch Mode

Monitor workspace continuously and run checks:

Add to `.vscode/tasks.json`:

```json
{
  "label": "cargo-cicd: Watch Status",
  "type": "shell",
  "command": "bash",
  "args": [
    "-c",
    "while true; do clear; cargo cicd status; sleep 30; done"
  ],
  "isBackground": true,
  "problemMatcher": {
    "pattern": {
      "regexp": "^(.*)$",
      "file": 1
    }
  },
  "presentation": {
    "echo": true,
    "reveal": "always",
    "focus": false
  }
}
```

---

## JetBrains IDEs

Applies to IntelliJ IDEA, CLion, RustRover, and other JetBrains IDEs.

### External Tools Configuration

1. Go to **Settings** → **Tools** → **External Tools**
2. Click the `+` button to add a new tool

#### Tool 1: Status

- **Name:** cargo-cicd: Status
- **Program:** `cargo`
- **Arguments:** `cicd status`
- **Working directory:** `$ProjectFileDir$`
- **Output:** Show in console
- **Open console:** Always

#### Tool 2: Workspace Doctor

- **Name:** cargo-cicd: Workspace Doctor
- **Program:** `cargo`
- **Arguments:** `cicd workspace`
- **Working directory:** `$ProjectFileDir$`
- **Output:** Show in console
- **Open console:** Always

#### Tool 3: Test Changed

- **Name:** cargo-cicd: Test Changed
- **Program:** `cargo`
- **Arguments:** `cicd test changed`
- **Working directory:** `$ProjectFileDir$`
- **Output:** Show in console
- **Open console:** Always

#### Tool 4: Full Pipeline

- **Name:** cargo-cicd: Full Pipeline
- **Program:** `cargo`
- **Arguments:** `cicd pipeline run`
- **Working directory:** `$ProjectFileDir$`
- **Output:** Show in console
- **Open console:** Always

### Keyboard Shortcuts

1. Go to **Settings** → **Keymap**
2. Search for "External Tools"
3. Find your cargo-cicd tools and assign keyboard shortcuts:
   - `Ctrl+Alt+C` for Status
   - `Ctrl+Alt+W` for Workspace Doctor
   - `Ctrl+Alt+T` for Test Changed
   - `Ctrl+Alt+P` for Full Pipeline

### Run Configuration

Create a run configuration for cargo-cicd pipeline:

1. Go to **Run** → **Edit Configurations**
2. Click `+` to add new configuration
3. Select **Cargo**
4. Configure:
   - **Name:** cargo-cicd Pipeline
   - **Command:** `pipeline` with sub-command `run`

### Gutter Icons (RustRover)

For RustRover specifically, you can add gutter icons for quick access:

1. Install the "Rust" plugin (built-in)
2. Open any `.rs` file
3. Gutter shows test run buttons; you can extend this with custom plugins

### Build Configuration

Set cargo-cicd as a build step:

1. Go to **Settings** → **Build, Execution, Deployment** → **Compiler** (Rust)
2. Add a build profile that runs cargo-cicd before compilation:

```
Before Launch:
  ✓ Run External Tool: cargo-cicd: Workspace Doctor
  ✓ Run External Tool: cargo-cicd: Test Changed
```

---

## Vim/Neovim

### Using Vim-Make

Run tasks with `:make`:

```vim
" ~/.config/nvim/init.vim or ~/.vimrc

" Define cargo-cicd commands
command! CargoStatus :make status
command! CargoWorkspace :make workspace
command! CargoTestChanged :make test-changed
command! CargoPublish :make publish
command! CargoPipeline :make pipeline

" Set makeprg to use cargo-cicd
set makeprg=cargo\ cicd\ %*
```

### Lua Configuration (Neovim)

For Neovim with Lua configuration:

```lua
-- ~/.config/nvim/init.lua

local opts = { noremap = true, silent = true }

vim.keymap.set('n', '<leader>cs', ':!cargo cicd status<CR>', opts)
vim.keymap.set('n', '<leader>cw', ':!cargo cicd workspace<CR>', opts)
vim.keymap.set('n', '<leader>ct', ':!cargo cicd test changed<CR>', opts)
vim.keymap.set('n', '<leader>cp', ':!cargo cicd publish<CR>', opts)
vim.keymap.set('n', '<leader>cl', ':!cargo cicd pipeline run<CR>', opts)
vim.keymap.set('n', '<leader>cg', ':!cargo cicd git status<CR>', opts)
```

### Using vim-dispatch

With vim-dispatch plugin for async execution:

```vim
" ~/.config/nvim/init.vim

nnoremap <leader>cs :Dispatch cargo cicd status<CR>
nnoremap <leader>cw :Dispatch cargo cicd workspace<CR>
nnoremap <leader>ct :Dispatch cargo cicd test changed<CR>
nnoremap <leader>cp :Dispatch cargo cicd publish<CR>
nnoremap <leader>cl :Dispatch cargo cicd pipeline run<CR>
```

### Using vim-fugitive (Git)

Extend vim-fugitive for cargo-cicd integration:

```vim
" ~/.config/nvim/init.vim

" After git status, run cargo cicd checks
nnoremap <leader>gs :Git<CR><C-w>j:!cargo cicd git status<CR>
nnoremap <leader>gc :!cargo cicd git close<CR>:Git<CR>
```

### Terminal Buffer

Use Neovim's terminal for interactive output:

```lua
-- ~/.config/nvim/init.lua

local function cargo_cicd(args)
  vim.cmd("belowright split")
  vim.cmd("term cargo cicd " .. args)
  vim.cmd("resize 20")
end

vim.keymap.set('n', '<leader>cs', function() cargo_cicd('status') end)
vim.keymap.set('n', '<leader>cw', function() cargo_cicd('workspace') end)
vim.keymap.set('n', '<leader>ct', function() cargo_cicd('test changed') end)
```

---

## Emacs

### Compilation Mode

Use Emacs compilation-mode for cargo-cicd:

```elisp
;; ~/.emacs.d/init.el

(defun cargo-cicd-status ()
  "Run cargo cicd status"
  (interactive)
  (compile "cargo cicd status"))

(defun cargo-cicd-workspace ()
  "Run cargo cicd workspace"
  (interactive)
  (compile "cargo cicd workspace"))

(defun cargo-cicd-test-changed ()
  "Run cargo cicd test changed"
  (interactive)
  (compile "cargo cicd test changed"))

(defun cargo-cicd-publish ()
  "Run cargo cicd publish"
  (interactive)
  (compile "cargo cicd publish"))

(defun cargo-cicd-pipeline ()
  "Run cargo cicd pipeline run"
  (interactive)
  (compile "cargo cicd pipeline run"))

;; Bind to keys
(with-eval-after-load 'rust-mode
  (define-key rust-mode-map (kbd "C-c s") #'cargo-cicd-status)
  (define-key rust-mode-map (kbd "C-c w") #'cargo-cicd-workspace)
  (define-key rust-mode-map (kbd "C-c t") #'cargo-cicd-test-changed)
  (define-key rust-mode-map (kbd "C-c p") #'cargo-cicd-publish)
  (define-key rust-mode-map (kbd "C-c l") #'cargo-cicd-pipeline))
```

### Ivy/Counsel Integration

Quick launcher for cargo-cicd commands:

```elisp
;; ~/.emacs.d/init.el

(defvar cargo-cicd-commands
  '(("cargo-cicd: status" . "cargo cicd status")
    ("cargo-cicd: workspace" . "cargo cicd workspace")
    ("cargo-cicd: test changed" . "cargo cicd test changed")
    ("cargo-cicd: git status" . "cargo cicd git status")
    ("cargo-cicd: target show" . "cargo cicd target show")
    ("cargo-cicd: publish" . "cargo cicd publish")
    ("cargo-cicd: pipeline" . "cargo cicd pipeline run")))

(defun cargo-cicd-launcher ()
  "Launch a cargo-cicd command via ivy"
  (interactive)
  (ivy-read "cargo-cicd: " cargo-cicd-commands
    :action (lambda (cmd) (compile (cdr cmd)))))

(global-set-key (kbd "C-c c") #'cargo-cicd-launcher)
```

### Org-Mode Integration

Track cargo-cicd output in Org-Mode:

```org
* cargo-cicd Status
** Status Check
#+BEGIN_SRC bash
cargo cicd status
#+END_SRC

** Workspace Health
#+BEGIN_SRC bash
cargo cicd workspace doctor
#+END_SRC

** Test Planning
#+BEGIN_SRC bash
cargo cicd test changed
#+END_SRC
```

Execute with `C-c C-c` in Org-Mode code blocks.

---

## Sublime Text

### Build System

Create a build system for cargo-cicd:

```json
{
  "shell_cmd": "cargo cicd $build_arg",
  "working_dir": "$folder",
  "variants": [
    {
      "name": "Status",
      "shell_cmd": "cargo cicd status"
    },
    {
      "name": "Workspace",
      "shell_cmd": "cargo cicd workspace"
    },
    {
      "name": "Test Changed",
      "shell_cmd": "cargo cicd test changed"
    },
    {
      "name": "Git Status",
      "shell_cmd": "cargo cicd git status"
    },
    {
      "name": "Target Show",
      "shell_cmd": "cargo cicd target show"
    },
    {
      "name": "Publish",
      "shell_cmd": "cargo cicd publish"
    },
    {
      "name": "Pipeline",
      "shell_cmd": "cargo cicd pipeline run"
    }
  ]
}
```

Save as: **Tools** → **Build System** → **New Build System** → `cargo-cicd.sublime-build`

### Keyboard Shortcuts

Add to `Preferences` → `Key Bindings`:

```json
[
  { "keys": ["ctrl+shift+c"], "command": "build", "args": {"variant": "Status"} },
  { "keys": ["ctrl+shift+w"], "command": "build", "args": {"variant": "Workspace"} },
  { "keys": ["ctrl+shift+t"], "command": "build", "args": {"variant": "Test Changed"} },
  { "keys": ["ctrl+shift+p"], "command": "build", "args": {"variant": "Pipeline"} }
]
```

---

## General Editor Integration

### Using Makefile

For any editor, create a `Makefile`:

```makefile
.PHONY: cicd-status cicd-workspace cicd-test cicd-publish cicd-pipeline

cicd-status:
	cargo cicd status

cicd-workspace:
	cargo cicd workspace

cicd-test:
	cargo cicd test changed

cicd-publish:
	cargo cicd publish

cicd-pipeline:
	cargo cicd pipeline run
```

Run with: `make cicd-status`, `make cicd-workspace`, etc.

### Using Shell Aliases

Add to your shell configuration (`.bashrc`, `.zshrc`, etc.):

```bash
alias ccs='cargo cicd status'
alias ccw='cargo cicd workspace'
alias cct='cargo cicd test changed'
alias ccp='cargo cicd publish'
alias ccl='cargo cicd pipeline run'
alias ccg='cargo cicd git status'
alias cctg='cargo cicd target show'
alias cctgp='cargo cicd target prune --apply'
```

Usage:
```bash
ccs      # cargo cicd status
ccw      # cargo cicd workspace
ccl      # cargo cicd pipeline run
```

### Using Shell Functions

Add to `.bashrc` or `.zshrc`:

```bash
# cargo-cicd quick access functions
cc() {
  local cmd=${1:-status}
  case "$cmd" in
    s|status)    cargo cicd status ;;
    w|workspace) cargo cicd workspace ;;
    t|test)      cargo cicd test changed ;;
    p|publish)   cargo cicd publish ;;
    l|pipeline)  cargo cicd pipeline run ;;
    g|git)       cargo cicd git status ;;
    tg|target)   cargo cicd target show ;;
    tp|prune)    cargo cicd target prune --apply ;;
    *)           echo "Unknown command: $cmd"; return 1 ;;
  esac
}
```

Usage:
```bash
cc status          # or cc s
cc workspace       # or cc w
cc pipeline        # or cc l
cc prune           # or cc tp
```

### Using Python/Bash Wrapper Scripts

Create a quick-launcher script:

```bash
#!/bin/bash
# ~/bin/cc (or ~/bin/cc.sh)

# cargo-cicd quick launcher
case "${1:-menu}" in
  status|s)    cargo cicd status ;;
  workspace|w) cargo cicd workspace ;;
  test|t)      cargo cicd test changed ;;
  git|g)       cargo cicd git status ;;
  target|tg)   cargo cicd target show ;;
  prune|tp)    cargo cicd target prune --apply ;;
  publish|p)   cargo cicd publish ;;
  pipeline|l)  cargo cicd pipeline run ;;
  *)
    echo "cargo-cicd launcher"
    echo "Usage: cc [command]"
    echo ""
    echo "Commands:"
    echo "  s, status    - Show status"
    echo "  w, workspace - Workspace doctor"
    echo "  t, test      - Test changed"
    echo "  g, git       - Git status"
    echo "  tg, target   - Target show"
    echo "  tp, prune    - Target prune"
    echo "  p, publish   - Publish state"
    echo "  l, pipeline  - Full pipeline"
    ;;
esac
```

Make executable:
```bash
chmod +x ~/bin/cc
export PATH="$PATH:$HOME/bin"
```

Usage:
```bash
cc s
cc w
cc l
```

---

## Recommended Workflow Combinations

### VS Code + GitHub Copilot

1. Install GitHub Copilot extension
2. Define cargo-cicd tasks in `.vscode/tasks.json`
3. Copilot will understand cargo-cicd and suggest improvements

### IntelliJ + Rust Plugin

1. Use External Tools for cargo-cicd
2. Bind to shortcuts
3. Run before committing

### Neovim + Telescope

With Telescope plugin:

```lua
-- ~/.config/nvim/init.lua

local builtin = require('telescope.builtin')
local actions = require('telescope.actions')

local function cargo_cicd_commands()
  local commands = {
    { name = 'Status', cmd = 'cargo cicd status' },
    { name = 'Workspace', cmd = 'cargo cicd workspace' },
    { name = 'Test Changed', cmd = 'cargo cicd test changed' },
    { name = 'Pipeline', cmd = 'cargo cicd pipeline run' }
  }
  
  builtin.quickfix()  -- or custom picker
end

vim.keymap.set('n', '<leader>cc', cargo_cicd_commands)
```

---

## Troubleshooting IDE Integration

### Command Not Found

Ensure cargo-cicd is in PATH:
```bash
which cargo-cicd
# Should output /home/user/.cargo/bin/cargo-cicd
```

### Working Directory Issues

Make sure external tools run from project root:
- Set working directory to `$ProjectFileDir$` (JetBrains)
- Use absolute paths in scripts
- Test from command line first

### Output Not Appearing

Check if output is being captured correctly:
- Enable "Show output" in task configuration
- Check stderr separately from stdout
- Run command manually to verify output

### Slow Execution

cargo-cicd should be fast, but:
- First run takes longer (compilation)
- Subsequent runs are cached
- Pipeline run with wasm4pm may take extra time

---

## Further Reading

- [Quick Start Guide](../reference/CLI_QUICK_START.md)
- [Cheat Sheet](../reference/CLI_CHEAT_SHEET.md)
- [CI/CD Pipelines](./CI_CD_PIPELINES.md)
