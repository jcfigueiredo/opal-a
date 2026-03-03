#!/usr/bin/env bash
set -euo pipefail

OPAL_BIN="${OPAL_BIN:-cargo run --quiet --}"
SPEC_DIR="${1:-tests/spec}"
PASS=0
FAIL=0
SKIP=0

for test_file in $(find "$SPEC_DIR" -name '*.opl' -type f | sort); do
    # Extract expected output from header comment
    expect_line=$(head -1 "$test_file")

    if [[ "$expect_line" =~ ^#\ expect:\ (.+)$ ]]; then
        expected="${BASH_REMATCH[1]}"
        actual=$($OPAL_BIN run "$test_file" 2>&1) || true

        if [[ "$actual" == "$expected" ]]; then
            echo "PASS: $test_file"
            PASS=$((PASS + 1))
        else
            echo "FAIL: $test_file"
            echo "  expected: $expected"
            echo "  actual:   $actual"
            FAIL=$((FAIL + 1))
        fi

    elif [[ "$expect_line" =~ ^#\ expect-error:\ (.+)$ ]]; then
        expected_error="${BASH_REMATCH[1]}"
        actual=$($OPAL_BIN run "$test_file" 2>&1) || true

        if [[ "$actual" == *"$expected_error"* ]]; then
            echo "PASS: $test_file"
            PASS=$((PASS + 1))
        else
            echo "FAIL: $test_file"
            echo "  expected error containing: $expected_error"
            echo "  actual: $actual"
            FAIL=$((FAIL + 1))
        fi

    else
        echo "SKIP: $test_file (no expect header)"
        SKIP=$((SKIP + 1))
    fi
done

echo ""
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"

if [[ $FAIL -gt 0 ]]; then
    exit 1
fi
