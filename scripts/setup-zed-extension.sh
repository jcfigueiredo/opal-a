#!/usr/bin/env bash
set -euo pipefail

# Setup or update the Opal extension for Zed.
#
# Compiles the tree-sitter grammar to WASM using Zed's WASI SDK,
# then installs it directly into Zed's extension directory.
#
# Re-run this script after changing grammar.js, scanner.c, or query files.
# Restart Zed (or reload) to pick up changes.

OPAL_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TS_DIR="$OPAL_ROOT/tree-sitter-opal"
ZED_EXT_DIR="$HOME/Library/Application Support/Zed/extensions"
WASI_SDK="$ZED_EXT_DIR/build/wasi-sdk"
EXT_INSTALLED="$ZED_EXT_DIR/installed/opal"

echo "Opal root: $OPAL_ROOT"

# Verify tree-sitter-opal exists
if [ ! -f "$TS_DIR/grammar.js" ]; then
    echo "Error: tree-sitter-opal/grammar.js not found"
    exit 1
fi

# Verify Zed's WASI SDK exists
if [ ! -f "$WASI_SDK/bin/clang" ]; then
    echo "Error: Zed's WASI SDK not found at $WASI_SDK"
    echo "Open Zed and install any extension first (this downloads the SDK)."
    exit 1
fi

# Step 1: Regenerate parser.c from grammar.js
echo "Regenerating parser..."
(cd "$TS_DIR" && pnpm install --silent 2>/dev/null && pnpm run generate 2>&1) || {
    echo "Error: Could not regenerate parser."
    echo "  cd $TS_DIR && pnpm install && pnpm run generate"
    exit 1
}

if [ ! -f "$TS_DIR/src/parser.c" ]; then
    echo "Error: src/parser.c not found after generation"
    exit 1
fi

# Step 2: Compile to WASM using Zed's WASI SDK
echo "Compiling grammar to WASM..."
CC="$WASI_SDK/bin/clang"
SYSROOT="$WASI_SDK/share/wasi-sysroot"
TMPDIR=$(mktemp -d)

"$CC" --sysroot="$SYSROOT" --target=wasm32-wasip1 -O2 -c -I "$TS_DIR/src" \
    "$TS_DIR/src/parser.c" -o "$TMPDIR/parser.o"

"$CC" --sysroot="$SYSROOT" --target=wasm32-wasip1 -O2 -c -I "$TS_DIR/src" \
    "$TS_DIR/src/scanner.c" -o "$TMPDIR/scanner.o"

"$WASI_SDK/bin/wasm-ld" --no-entry --export-dynamic \
    -L "$SYSROOT/lib/wasm32-wasip1" -lc \
    -o "$TMPDIR/opal.wasm" "$TMPDIR/parser.o" "$TMPDIR/scanner.o"

echo "  WASM size: $(du -h "$TMPDIR/opal.wasm" | cut -f1)"

# Step 3: Install into Zed's extensions directory
echo "Installing extension..."
mkdir -p "$EXT_INSTALLED/grammars" "$EXT_INSTALLED/languages/opal"

cp "$TMPDIR/opal.wasm" "$EXT_INSTALLED/grammars/opal.wasm"
rm -rf "$TMPDIR"

# extension.toml
cat > "$EXT_INSTALLED/extension.toml" << 'TOML'
id = "opal"
name = "Opal"
version = "0.1.0"
schema_version = 1
description = "Opal language support: syntax highlighting."
repository = "https://github.com/jcfigueiredo/opal-a"
authors = ["Claudio"]

[grammars.opal]
repository = "https://github.com/jcfigueiredo/opal-a"
rev = "main"
TOML

# Language config
cat > "$EXT_INSTALLED/languages/opal/config.toml" << 'TOML'
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
cp "$TS_DIR/queries/highlights.scm" "$EXT_INSTALLED/languages/opal/highlights.scm"
cp "$TS_DIR/queries/indents.scm" "$EXT_INSTALLED/languages/opal/indents.scm"

# Outline query for Zed's symbol outline
cat > "$EXT_INSTALLED/languages/opal/outline.scm" << 'SCM'
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

# Step 4: Update Zed's extension index
echo "Updating extension index..."
python3 << 'PY'
import json, os

index_path = os.path.expanduser("~/Library/Application Support/Zed/extensions/index.json")
with open(index_path) as f:
    index = json.load(f)

index["extensions"]["opal"] = {
    "manifest": {
        "id": "opal",
        "name": "Opal",
        "version": "0.1.0",
        "schema_version": 1,
        "description": "Opal language support: syntax highlighting.",
        "repository": "https://github.com/jcfigueiredo/opal-a",
        "authors": ["Claudio"],
        "lib": {"kind": None, "version": None},
        "themes": [],
        "icon_themes": [],
        "languages": ["languages/opal"],
        "grammars": {
            "opal": {
                "repository": "https://github.com/jcfigueiredo/opal-a",
                "rev": "main",
                "path": None
            }
        },
        "language_servers": {},
        "context_servers": {},
        "agent_servers": {},
        "slash_commands": {},
        "snippets": None,
        "capabilities": []
    },
    "dev": False
}

index["languages"]["Opal"] = {
    "extension": "opal",
    "path": "languages/opal",
    "matcher": {
        "path_suffixes": ["opl"],
        "first_line_pattern": None
    },
    "hidden": False,
    "grammar": "opal"
}

with open(index_path, "w") as f:
    json.dump(index, f, indent=2)
PY

echo ""
echo "Done! Restart Zed to pick up the Opal extension."
echo "Open any .opl file — it should show 'Opal' in the status bar."
