#!/usr/bin/env bash
set -euo pipefail

# Setup or update the Opal dev extension for Zed.
#
# This creates an extension directory that Zed can load via:
#   Cmd+Shift+P → "zed: install dev extension" → select the extension dir
#
# Re-run this script after changing the grammar or queries to update.

OPAL_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXT_DIR="$OPAL_ROOT/zed-opal-extension"
TS_DIR="$OPAL_ROOT/tree-sitter-opal"

echo "Opal root: $OPAL_ROOT"
echo "Extension dir: $EXT_DIR"

# Verify tree-sitter-opal exists
if [ ! -f "$TS_DIR/grammar.js" ]; then
    echo "Error: tree-sitter-opal/grammar.js not found"
    exit 1
fi

# Create extension structure
mkdir -p "$EXT_DIR/grammars/opal"
mkdir -p "$EXT_DIR/languages/opal"

# extension.toml
cat > "$EXT_DIR/extension.toml" << 'TOML'
id = "opal"
name = "Opal"
version = "0.1.0"
schema_version = 1
description = "Opal language support: syntax highlighting and LSP."
authors = ["Claudio"]

[grammars.opal]
TOML

# Language config
cat > "$EXT_DIR/languages/opal/config.toml" << 'TOML'
name = "Opal"
grammar = "opal"
path_suffixes = ["opl"]
line_comments = ["# "]
block_comment = ["###", "###"]
autoclose_before = ",]})'"
brackets = [
    { start = "(", end = ")", close = true, newline = true },
    { start = "[", end = "]", close = true, newline = true },
    { start = "{", end = "}", close = true, newline = true },
    { start = "\"", end = "\"", close = true, newline = false, not_in = ["comment", "string"] },
    { start = "'", end = "'", close = true, newline = false, not_in = ["comment", "string"] },
]
word_characters = ["!"]
TOML

# Copy query files
echo "Copying query files..."
cp "$TS_DIR/queries/highlights.scm" "$EXT_DIR/languages/opal/highlights.scm"
cp "$TS_DIR/queries/indents.scm" "$EXT_DIR/languages/opal/indents.scm"

# Create outline query from locals (Zed uses outline.scm for symbol outline)
cat > "$EXT_DIR/languages/opal/outline.scm" << 'SCM'
(function_definition name: (identifier) @name) @item
(class_definition name: (identifier) @name) @item
(module_definition name: (identifier) @name) @item
(protocol_definition name: (identifier) @name) @item
(enum_definition name: (identifier) @name) @item
(model_definition name: (identifier) @name) @item
(actor_definition name: (identifier) @name) @item
(event_definition name: (identifier) @name) @item
(type_alias name: (identifier) @name) @item
SCM

# Symlink grammar source files into the extension's grammar dir
# Zed dev extensions compile from source — it needs the tree-sitter files
echo "Linking grammar source..."
for f in grammar.js tree-sitter.json package.json; do
    ln -sf "$TS_DIR/$f" "$EXT_DIR/grammars/opal/$f"
done

# Link src/ directory (parser.c, scanner.c, etc.)
if [ -d "$TS_DIR/src" ]; then
    ln -sfn "$TS_DIR/src" "$EXT_DIR/grammars/opal/src"
fi

# Build LSP if not already built
LSP_BIN="$OPAL_ROOT/target/release/opal-lsp"
if [ ! -f "$LSP_BIN" ]; then
    echo "Building opal-lsp (release)..."
    (cd "$OPAL_ROOT" && cargo build --release -p opal-lsp)
fi

echo ""
echo "Done! Extension ready at: $EXT_DIR"
echo ""
echo "To install in Zed:"
echo "  1. Open Zed"
echo "  2. Cmd+Shift+P → 'zed: install dev extension'"
echo "  3. Select: $EXT_DIR"
echo ""
echo "To configure the LSP, add to ~/.config/zed/settings.json:"
echo ""
echo '  "lsp": {'
echo '    "opal-lsp": {'
echo "      \"binary\": { \"path\": \"$LSP_BIN\" }"
echo '    }'
echo '  },'
echo '  "languages": {'
echo '    "Opal": {'
echo '      "language_servers": ["opal-lsp"]'
echo '    }'
echo '  }'
echo ""
echo "After grammar changes, re-run this script then"
echo "Cmd+Shift+P → 'zed: rebuild extension' in Zed."
