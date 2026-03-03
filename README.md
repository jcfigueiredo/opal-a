# Opal

An opinionated programming language combining Ruby's expressiveness with Erlang's concurrency model.

**Status:** Phase 1 — Rust tree-walk interpreter. Currently supports strings, numbers, functions, recursion, and `print()`.

## Quick Start

```bash
# Build
cargo build

# Run a program
cargo run -- run tests/spec/02-functions/factorial.opl

# Run all tests
cargo test --all

# Run spec tests (end-to-end)
bash tests/run_spec.sh

# Run benchmarks
cargo bench --package opal-lexer
```

## Example

```opal
def factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1) end
end

print(factorial(10))  # => 3628800
```

## Project Structure

```
crates/
  opal-lexer/     # logos-based tokenizer
  opal-parser/    # recursive descent parser
  opal-interp/    # tree-walk interpreter
  opal-runtime/   # values, environment, scoping
  opal-stdlib/    # builtin functions (print, println)
  opal-cli/       # CLI entry point (opal run/repl/test/bench)
tests/
  spec/           # end-to-end spec tests (.opl files)
  bench/          # benchmark programs
Opal.md           # language specification
docs/             # design documents
```

## Requirements

- Rust 1.85+ (edition 2024)
