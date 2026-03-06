#!/usr/bin/env bash
set -euo pipefail

# Setup or update the Opal extension for Cursor/VS Code.
#
# Symlinks the extension into Cursor's extensions directory.
# Restart Cursor after running to pick up changes.

OPAL_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXT_SRC="$OPAL_ROOT/editors/vscode-opal"

# Find the extensions directory
if [ -d "$HOME/.cursor/extensions" ]; then
    EXT_DIR="$HOME/.cursor/extensions/opal-language-0.1.0"
elif [ -d "$HOME/.vscode/extensions" ]; then
    EXT_DIR="$HOME/.vscode/extensions/opal-language-0.1.0"
else
    echo "Error: Neither ~/.cursor/extensions nor ~/.vscode/extensions found"
    exit 1
fi

# Build LSP if not already built
LSP_BIN="$OPAL_ROOT/target/release/opal-lsp"
if [ ! -f "$LSP_BIN" ]; then
    echo "Building opal-lsp (release)..."
    (cd "$OPAL_ROOT" && cargo build --release -p opal-lsp)
fi

# Bundle the extension (inlines vscode-languageclient into a single file)
echo "Bundling extension..."
(cd "$EXT_SRC" && pnpm install --silent 2>/dev/null && pnpm exec esbuild src/extension.js --bundle --outfile=dist/extension.js --external:vscode --format=cjs --platform=node --minify 2>&1)

echo "Installing Opal extension to: $EXT_DIR"

# Remove old version if exists
rm -rf "$EXT_DIR"

# Copy only what's needed (no node_modules, no src)
mkdir -p "$EXT_DIR/dist" "$EXT_DIR/syntaxes"
cp "$EXT_SRC/package.json" "$EXT_DIR/"
cp "$EXT_SRC/language-configuration.json" "$EXT_DIR/"
cp "$EXT_SRC/syntaxes/opal.tmLanguage.json" "$EXT_DIR/syntaxes/"
cp "$EXT_SRC/dist/extension.js" "$EXT_DIR/dist/"

echo ""
echo "Done! Restart Cursor to pick up the Opal extension."
echo "Features: syntax highlighting, diagnostics, go-to-definition, document outline."
