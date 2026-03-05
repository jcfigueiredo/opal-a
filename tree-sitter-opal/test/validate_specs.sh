#!/usr/bin/env bash
set -euo pipefail

PASS=0
ERRORS=0

for f in ../tests/spec/**/*.opl; do
    output=$(pnpm exec tree-sitter parse "$f" 2>&1)
    if echo "$output" | grep -q "ERROR"; then
        echo "HAS ERRORS: $f"
        echo "$output" | grep "ERROR" | head -3
        echo ""
        ERRORS=$((ERRORS + 1))
    else
        PASS=$((PASS + 1))
    fi
done

echo "Results: $PASS clean, $ERRORS with errors (out of $((PASS + ERRORS)) files)"
