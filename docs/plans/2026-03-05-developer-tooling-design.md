# Developer Tooling Design

## Goal

Add editor support for Opal: syntax highlighting via tree-sitter, and a language server for diagnostics, go-to-definition, and document symbols.

## Chosen Approach: A — Tree-sitter first, LSP second

**Order:** Tree-sitter grammar → LSP server → (Formatter deferred)

Tree-sitter gives immediate visual payoff in all three target editors (Cursor, Neovim, Zed). It's self-contained and required by Zed. The LSP builds on the existing Rust parser and gives diagnostics + go-to-definition. The formatter is deferred until formatting becomes a pain point.

## Target Editors

| Editor | Syntax Highlighting | Intelligence |
|--------|-------------------|-------------|
| Cursor (VS Code fork) | TextMate grammar or tree-sitter via extension | LSP |
| Neovim | Tree-sitter (native) | LSP (nvim-lspconfig) |
| Zed | Tree-sitter (native, required) | LSP |

Tree-sitter + LSP covers all three editors with the same artifacts.

---

## Part 1: Tree-sitter Grammar

### Deliverable

`tree-sitter-opal` — a standalone tree-sitter grammar package.

### Structure

```
tree-sitter-opal/
├── grammar.js          # Grammar definition
├── src/
│   └── scanner.c       # External scanner (f-string interpolation)
├── queries/
│   ├── highlights.scm  # Syntax highlighting queries
│   ├── locals.scm      # Scope/local variable tracking
│   └── indents.scm     # Auto-indentation rules (Neovim/Zed)
├── test/corpus/        # Tree-sitter test cases
├── package.json        # npm package for tree-sitter CLI
└── bindings/           # Auto-generated (C, Rust, Node)
```

### Grammar Approach

Translate the 83 `parse_*` methods in `crates/opal-parser/src/parser.rs` into tree-sitter's `grammar.js` DSL. The Rust parser is the canonical reference.

**Language complexity:** 109 tokens (55 keywords), 27 statement kinds, 27 expression kinds.

### Key Challenges

- **F-string interpolation** — Requires an external scanner (C code) to handle `f"hello {expr}"` with brace-counting inside strings. This is the hardest part. Reference: tree-sitter-python's f-string scanner.
- **`end`-terminated blocks** — Ruby-like syntax, well-supported in tree-sitter. Reference: tree-sitter-ruby.
- **Operator precedence** — 14 binary operators mapped to `prec.left()` / `prec.right()` levels.
- **Multiline comments** — `###...###` blocks need external scanner support.

### Testing

- Tree-sitter corpus tests: one test per major language construct
- Validate against all 42 existing `.opl` spec test files
- `tree-sitter parse` and `tree-sitter highlight` smoke tests

### Editor Integration

- **Neovim:** Add to `nvim-treesitter` parsers config (local initially)
- **Zed:** `extensions/opal/` with `extension.toml` pointing to grammar
- **Cursor/VS Code:** Extension with TextMate grammar as primary (VS Code's tree-sitter support is limited), tree-sitter via vscode-anycode as alternative

---

## Part 2: LSP Server

### Deliverable

`opal-lsp` — a new crate in the Cargo workspace, producing the `opal-lsp` binary.

### Tech Stack

- `tower-lsp-server` (community fork) + `tokio` for async
- Reuses `opal-lexer` and `opal-parser` directly — no duplicate parsing

### Scope (v1 — minimal)

| Feature | LSP Method | Implementation |
|---------|-----------|----------------|
| Diagnostics | `publishDiagnostics` | Parse on save/change, convert `ParseError` spans to LSP diagnostics |
| Go-to-definition | `textDocument/definition` | Walk AST, build single-file symbol table, resolve identifier at cursor |
| Document symbols | `textDocument/documentSymbol` | Walk AST, emit nested symbols (class → methods) for outline view |

**Not in v1:** Completions, hover, find-references, rename, workspace indexing, type inference.

### Architecture

```
crates/opal-lsp/
├── src/
│   ├── main.rs          # tokio + tower-lsp Server setup over stdio
│   ├── backend.rs       # LanguageServer trait impl
│   ├── diagnostics.rs   # Parse source → Vec<Diagnostic>
│   ├── symbols.rs       # AST → Vec<DocumentSymbol>
│   └── goto_def.rs      # Position → symbol definition location
├── Cargo.toml
```

### Document Management

Full text sync (`TextDocumentSyncKind::FULL`) — re-parse the whole file on each change. Opal files are small; the parser is fast. Incremental sync is a future optimization.

### Editor Integration

All three editors support LSP natively. Configuration is just pointing to the `opal-lsp` binary path.

---

## Future Improvements (Deferred)

### Formatter (`opal fmt`)

Deferred until formatting becomes a pain point. When built:

- **Lightweight approach:** Line-based text formatter in `opal-cli`, tracking keyword/end nesting for indentation. No AST round-tripping needed for basic formatting.
- **Full approach:** AST-based pretty-printer for operator spacing, argument alignment, line wrapping.
- **LSP integration:** Expose via `textDocument/formatting` for format-on-save.

### Alternative Approaches Considered

**Approach B — LSP first, tree-sitter second:** LSP can provide semantic highlighting via `textDocument/semanticTokens`, giving highlighting + intelligence in one tool. Rejected because: semantic tokens are slower, don't work in Zed, and provide worse highlighting for incomplete code.

**Approach C — All-in-one:** Build tree-sitter and LSP together as a single package. Rejected because: larger single deliverable, harder to test incrementally, blocks shipping if either part has issues.

### LSP v2 Features

- Completions (known symbols in scope)
- Hover info (type annotations, function signatures)
- Find references
- Workspace-wide indexing (multi-file)
- Rename symbol
- Signature help
