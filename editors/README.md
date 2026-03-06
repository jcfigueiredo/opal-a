# Opal Editor Support

## Cursor / VS Code (recommended)

One-step setup:

```bash
./scripts/setup-cursor-extension.sh
```

Then restart Cursor. Open any `.opl` file — you should see:

- **"Opal" in the status bar** (language detected)
- **Syntax highlighting** (keywords, strings, types, functions, comments, symbols, f-strings)
- **Parse error diagnostics** (red squiggles on syntax errors)
- **Go-to-definition** (Cmd+click or F12 on identifiers)
- **Document outline** (Cmd+Shift+O for functions, classes, modules, etc.)

### What the script does

1. Builds `opal-lsp` (release) if not already built
2. Bundles the LSP client with esbuild (inlines `vscode-languageclient` — no `node_modules` at runtime)
3. Copies the extension to `~/.cursor/extensions/opal-language-0.1.0/`

### Troubleshooting

If LSP features don't work (no diagnostics, no go-to-definition):

1. Open the Output panel: **Cmd+Shift+U**
2. Select **"Opal Language Server"** from the dropdown
3. Check for errors — the extension logs its binary search path

Common issues:
- **"binary not found"** — run `cargo build --release -p opal-lsp` from the project root
- **No output channel** — the extension didn't activate. Restart Cursor
- **Extension not listed** — re-run `./scripts/setup-cursor-extension.sh` and restart

### Updating after changes

Re-run the setup script after changing the grammar or LSP:

```bash
# After editing syntaxes/opal.tmLanguage.json or LSP code:
./scripts/setup-cursor-extension.sh
# Then restart Cursor
```

## Neovim

### Tree-sitter (syntax highlighting)

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.opal = {
  install_info = {
    url = "/path/to/opal/tree-sitter-opal",
    files = { "src/parser.c", "src/scanner.c" },
    branch = "main",
  },
  filetype = "opal",
}

vim.filetype.add({
  extension = {
    opl = "opal",
  },
})
```

Then run `:TSInstall opal`.

Copy the query files:
```bash
mkdir -p ~/.config/nvim/queries/opal/
cp tree-sitter-opal/queries/*.scm ~/.config/nvim/queries/opal/
```

### LSP

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "opal",
  callback = function()
    vim.lsp.start({
      name = "opal-lsp",
      cmd = { "/path/to/opal/target/release/opal-lsp" },
      root_dir = vim.fn.getcwd(),
    })
  end,
})
```

## Features

| Feature | Cursor/VS Code | Neovim |
|---------|---------------|--------|
| Syntax highlighting | TextMate grammar | Tree-sitter grammar (29 corpus tests) |
| Parse error diagnostics | LSP (on file open/change) | LSP |
| Document symbols (Cmd+Shift+O) | LSP | LSP |
| Go-to-definition (Cmd+click) | LSP | LSP |
| Auto-indent | language-configuration.json | indents.scm |
| Bracket matching | language-configuration.json | Tree-sitter |
| F-string interpolation | TextMate patterns | External scanner |

## Architecture

```
editors/vscode-opal/          # Cursor/VS Code extension source
├── package.json               # Extension manifest
├── language-configuration.json # Brackets, comments, indentation
├── syntaxes/opal.tmLanguage.json # TextMate grammar for highlighting
└── src/extension.js           # LSP client (bundled with esbuild)

tree-sitter-opal/             # Tree-sitter grammar (for Neovim/Zed)
├── grammar.js                 # Grammar definition
├── src/scanner.c              # External scanner for f-strings
├── queries/                   # highlights.scm, indents.scm, locals.scm
└── test/corpus/               # 29 corpus tests

crates/opal-lsp/              # LSP server (shared by all editors)
├── src/main.rs                # tower-lsp server
├── src/diagnostics.rs         # Parse error → LSP diagnostic
├── src/symbols.rs             # AST → document symbols
└── src/goto_def.rs            # Symbol table → go-to-definition
```
