# Editor Integration

cargo-cicd-lsp speaks the Language Server Protocol over stdio. Any LSP-compliant editor can connect to it by configuring the server command.

---

## VS Code

Install the `cargo-cicd-lsp` binary, then add the following to your workspace or user `settings.json`.

This configuration uses the generic [vscode-glspc](https://marketplace.visualstudio.com/items?itemName=rjm.glspc) extension or any extension that accepts a custom LSP server command. A dedicated VS Code extension is planned.

```jsonc
// .vscode/settings.json
{
  "glspc.serverPath": "cargo-cicd-lsp",
  "glspc.serverArgs": ["serve"],
  "glspc.documentSelector": [
    { "language": "toml", "scheme": "file" },
    { "language": "rust", "scheme": "file" }
  ],
  "glspc.rootPatterns": ["cicd.toml", "Cargo.toml"]
}
```

Diagnostics will appear in the Problems panel and as inline squiggles in `cicd.toml` and `Cargo.toml` files.

### Recommended Extensions

- `tamasfe.even-better-toml` — syntax highlighting and schema validation for TOML files
- `rust-lang.rust-analyzer` — Rust language support (runs independently of cargo-cicd-lsp)

### Tasks Integration

Add a task to run `doctor` without opening the editor:

```jsonc
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo-cicd: doctor",
      "type": "shell",
      "command": "cargo-cicd-lsp doctor",
      "group": "build",
      "presentation": {
        "reveal": "always",
        "panel": "shared"
      },
      "problemMatcher": []
    }
  ]
}
```

---

## Neovim

Requires [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) (0.2.0 or later).

Add the following to your Neovim configuration:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Register cargo-cicd-lsp if not already known
if not configs.cargo_cicd_lsp then
  configs.cargo_cicd_lsp = {
    default_config = {
      cmd = { 'cargo-cicd-lsp', 'serve' },
      filetypes = { 'toml', 'rust' },
      root_dir = lspconfig.util.root_pattern('cicd.toml', 'Cargo.toml'),
      settings = {},
    },
  }
end

lspconfig.cargo_cicd_lsp.setup({
  on_attach = function(client, bufnr)
    -- Optional: bind explain to a keymap
    vim.keymap.set('n', '<leader>ce', function()
      local diagnostics = vim.diagnostic.get(bufnr, { lnum = vim.fn.line('.') - 1 })
      if #diagnostics > 0 then
        local code = diagnostics[1].code
        if code then
          vim.fn.system('cargo-cicd-lsp explain ' .. code)
        end
      end
    end, { buffer = bufnr, desc = 'cargo-cicd: explain diagnostic' })
  end,
})
```

Diagnostics appear in the sign column and can be viewed with `:lua vim.diagnostic.open_float()`.

### Telescope Integration (optional)

```lua
-- List all cargo-cicd diagnostics in Telescope
vim.keymap.set('n', '<leader>cd', function()
  require('telescope.builtin').diagnostics({
    namespace = vim.lsp.diagnostic.get_namespace(
      vim.lsp.get_clients({ name = 'cargo_cicd_lsp' })[1].id
    )
  })
end, { desc = 'cargo-cicd: diagnostics' })
```

---

## Helix

Add the following to your `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "toml"
language-servers = ["rust-analyzer", "cargo-cicd-lsp"]

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "cargo-cicd-lsp"]

[language-server.cargo-cicd-lsp]
command = "cargo-cicd-lsp"
args = ["serve"]
```

Helix will spawn `cargo-cicd-lsp serve` when opening any `.toml` or `.rs` file inside a workspace that contains a `cicd.toml`.

Diagnostics appear in the status line and the diagnostics picker (`<space>d`).

---

## Other Editors

Any editor that supports LSP can connect to cargo-cicd-lsp by spawning:

```sh
cargo-cicd-lsp serve
```

The server communicates over stdin/stdout using the standard LSP JSON-RPC framing (Content-Length headers). It handles:

- `initialize` / `initialized`
- `textDocument/didOpen`, `didChange`, `didSave`, `didClose`
- `workspace/didChangeWatchedFiles`
- `textDocument/publishDiagnostics` (server-push)
- `shutdown` / `exit`

It does not implement completion, hover, or code actions in v1. Those capabilities are declared as unsupported in the `initialize` response.

---

## Troubleshooting

**Server does not start**
- Confirm `cargo-cicd-lsp` is on `$PATH`: `which cargo-cicd-lsp`
- Run `cargo-cicd-lsp doctor` in the workspace root to verify standalone operation

**No diagnostics appear**
- Confirm a `cicd.toml` exists at the workspace root
- Check the editor's LSP log for the `initialize` exchange
- Run `cargo-cicd-lsp doctor` to confirm diagnostics are produced outside the editor

**Diagnostics do not clear after fixing a problem**
- Save the file that was fixed; the server evaluates on `didSave`
- If `cicd.toml` was changed externally, the `didChangeWatchedFiles` notification triggers re-evaluation — confirm your editor sends this notification

---

## See Also

- [README.md](README.md) — Installation and commands
- [DIAGNOSTICS.md](DIAGNOSTICS.md) — Full code table
- [LIFECYCLE.md](LIFECYCLE.md) — How diagnostics are raised and cleared
