# Opal Editor Support

## Prerequisites

Build the LSP server:

```bash
cargo build --release -p opal-lsp
```

The binary is at `target/release/opal-lsp`.

## Neovim

### Tree-sitter (syntax highlighting)

Add to your `init.lua` or tree-sitter config:

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

## Zed

Create an extension directory:

```
~/.config/zed/extensions/opal/
├── extension.toml
├── grammars/opal/
│   └── (symlink to tree-sitter-opal/)
└── languages/opal/
    ├── config.toml
    ├── highlights.scm
    ├── indents.scm
    └── locals.scm
```

`extension.toml`:
```toml
id = "opal"
name = "Opal"
version = "0.1.0"

[grammars.opal]
repository = "file:///path/to/opal/tree-sitter-opal"

[language_servers.opal-lsp]
language = "Opal"
```

`languages/opal/config.toml`:
```toml
name = "Opal"
grammar = "opal"
path_suffixes = ["opl"]
line_comments = ["# "]
```

## Cursor / VS Code

For LSP support, a VS Code extension with `package.json` and `extension.js` is recommended. As a quick start, use a generic LSP client extension and configure:

```json
{
  "opal.lsp.path": "/path/to/opal/target/release/opal-lsp"
}
```

## Features

| Feature | Status |
|---------|--------|
| Syntax highlighting (tree-sitter) | 29 corpus tests, 85% spec coverage |
| Parse error diagnostics | On file open/change |
| Document symbols (outline) | Functions, classes, modules, enums, actors, models, events |
| Go-to-definition | Variables, functions, classes, modules, enums |
| F-string interpolation | External scanner with brace tracking |
