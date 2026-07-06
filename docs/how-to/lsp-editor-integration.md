# Editor Integration

cargo-cicd-lsp speaks the Language Server Protocol over stdio. Any LSP-compliant editor
can connect to it by configuring the server binary to `cargo-cicd-lsp`.

---

## VS Code

VS Code support is coming soon. A dedicated extension is planned.

In the meantime, the generic
[vscode-glspc](https://marketplace.visualstudio.com/items?itemName=rjm.glspc) extension
accepts a custom LSP server command:

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

---

## Neovim

Requires [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) 0.2.0 or later.

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

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

lspconfig.cargo_cicd_lsp.setup({})
```

Diagnostics appear in the sign column. View them with `:lua vim.diagnostic.open_float()`.

---

## Helix

Add to `~/.config/helix/languages.toml`:

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

Diagnostics appear in the status line and the diagnostics picker (`<space>d`).

---

## Zed

Zed support is coming later.

---

## General

Any LSP client can connect by setting the server binary to `cargo-cicd-lsp`.

The server communicates over stdin/stdout using standard LSP JSON-RPC framing
(Content-Length headers). It handles:

- `initialize` / `initialized`
- `textDocument/didOpen`, `didChange`, `didSave`, `didClose`
- `workspace/didChangeWatchedFiles`
- `textDocument/publishDiagnostics` (server-push)
- `shutdown` / `exit`

Completion, hover, and code actions are not implemented in v1. These capabilities are
declared as unsupported in the `initialize` response.

---

## Troubleshooting

**Server does not start**

- Confirm the binary is on `$PATH`: `which cargo-cicd-lsp`
- Run `cargo cicd lsp doctor` in the workspace root to verify standalone operation

**No diagnostics appear**

- Confirm `cicd.toml` exists at the workspace root
- Check the editor LSP log for the `initialize` exchange
- Run `cargo cicd lsp doctor` to confirm diagnostics are produced outside the editor

**Diagnostics do not clear after fixing a problem**

- Save the file that was fixed; the server evaluates on `didSave`
- If `cicd.toml` was changed externally, confirm your editor sends `didChangeWatchedFiles`

---

## See Also

- [README.md](README.md) — Installation and commands
- [DIAGNOSTICS.md](DIAGNOSTICS.md) — Full code catalog by family
- [LIFECYCLE.md](LIFECYCLE.md) — How diagnostics are raised and cleared
