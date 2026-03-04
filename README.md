# Opal

An opinionated programming language combining Ruby's expressiveness with Erlang's concurrency model.

**Status:** Phase 1 complete — Rust tree-walk interpreter with functions, closures, classes, modules, actors, macros, pattern matching, error handling, FFI, and an HTTP web server.

## Quick Start

```bash
# Build
cargo build

# Start the REPL
cargo run -- repl

# Run a program
cargo run -- run tests/spec/02-functions/factorial.opl

# Run all tests
cargo test --all

# Run spec tests (end-to-end)
bash tests/run_spec.sh
```

## Examples

```opal
# Functions and recursion
def factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1) end
end
print(factorial(10))  # => 3628800

# Lists, closures, and method chaining
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
evens = numbers.filter(|n| n % 2 == 0)
squares = evens.map(|n| n ** 2)
total = squares.reduce(0) do |acc, n|
  acc + n
end
print(f"Sum of even squares: {total}")  # => 220

# Classes and modules
module Shapes
  class Circle
    needs radius: Float
    def area() -> Float
      Math.pi() * .radius ** 2
    end
  end
end
from Shapes import Circle
c = Circle.new(radius: 5.0)
print(f"Area: {c.area()}")

# Pattern matching with Result types
def divide(a, b)
  requires b != 0, "division by zero"
  Ok(a / b)
end
match divide(10.0, 3.0)
  case Ok(result)
    print(f"Result: {result}")
  case Error(msg)
    print(f"Error: {msg}")
end

# Actors
actor Counter
  def init()
    .count = 0
  end
  receive
    case :increment
      .count = .count + 1
    case :get
      reply .count
  end
end
counter = Counter.new()
counter.send(:increment)
print(await counter.send(:get))  # => 1

# Macros
macro unless(condition, body)
  ast
    if not ($condition)
      $body
    end
  end
end
@unless false
  print("This prints!")
end
```

## Project Structure

```
crates/
  opal-lexer/     # logos-based tokenizer
  opal-parser/    # recursive descent parser with Pratt expressions
  opal-interp/    # tree-walk interpreter
  opal-runtime/   # values, environment, scoping
  opal-stdlib/    # builtins, FFI plugin registry, HTTP server
  opal-cli/       # CLI (run, repl, test, bench)
tests/
  spec/           # end-to-end spec tests (.opl files with expected output)
  bench/          # benchmark programs
Opal.md           # language specification
docs/             # design documents and plans
```

## Requirements

- Rust 1.85+ (edition 2024)
