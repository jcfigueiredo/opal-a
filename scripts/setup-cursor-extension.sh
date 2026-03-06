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

echo "Installing Opal extension to: $EXT_DIR"

# Remove old version if exists
rm -rf "$EXT_DIR"

# Copy extension (can't symlink — Cursor needs real files)
cp -r "$EXT_SRC" "$EXT_DIR"
# Remove node_modules if accidentally copied
rm -rf "$EXT_DIR/node_modules"

echo ""
echo "Done! Restart Cursor to pick up the Opal extension."
echo "Open any .opl file — it should show 'Opal' with syntax highlighting."
