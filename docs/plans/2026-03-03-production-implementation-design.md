# Opal Production Implementation Design

## Goal

Bring Opal from specification to a working, deployable language. Deliver value iteratively through vertical slices — each slice adds features needed for a concrete program, culminating in a working web server that proves the language works end-to-end.

## Strategy: Phased, XP-Style

Two phases, one codebase. Phase 1 validates language design cheaply with a Rust interpreter. Phase 2 graduates to production quality by compiling to the BEAM (Erlang VM).

**Why phased:** Debugging language semantics and compiler correctness simultaneously is expensive. Gleam followed this pattern (Erlang prototype → Rust rewrite). The interpreter phase lets us discover design problems (ambiguous dispatch, macro edge cases, closure semantics) before committing to a production backend.

**Why BEAM for production:** Opal's actor model, supervisors, fault-tolerance, and "let it crash" error handling are a near-perfect semantic match for the BEAM. Actors become BEAM processes, supervisors become OTP supervisors, message passing is native. The alternative (building an actor runtime from scratch on LLVM or a custom VM) would cost months of engineering for an inferior result.

**Why Rust for the compiler:** Industry standard for language tooling (Ruff, Biome, SWC, Gleam). Memory-safe, fast compilation, excellent parser libraries (logos for lexing), strong BEAM interop via rustler if needed.

---

## Architecture

### Phase 1: Rust Interpreter (Slices 0-8)

```
opal CLI
├── opal run <file.opl>
├── opal repl
├── opal test
└── opal bench
─────────────────────────
Interpreter
├── Lexer (logos — DFA from Rust enums, zero-copy)
├── Parser (hand-written recursive descent)
├── Macro Expander (AST → AST)
├── Tree-Walk Evaluator (AST → values)
└── Runtime (values, env, actors on Tokio)
─────────────────────────
Stdlib (Opal + Rust native modules)
├── Core (String, Int, Float, Bool, List, Dict, ...)
├── IO, File, Net
└── Test, Bench
```

### Phase 2: BEAM Compiler (after Slice 8)

```
opal CLI (same interface)
├── opal build → .beam files
├── opal run (via BEAM)
└── opal release
─────────────────────────
Compiler (reuses Phase 1 frontend)
├── Lexer (same)
├── Parser (same)
├── Macro Expander (same)
├── Type Checker (new — gradual typing)
├── Core Erlang Codegen (new)
└── BEAM integration (OTP, mix/hex interop)
```

**Key:** Lexer, parser, and macro expander are shared. Phase 2 replaces only the backend.

### Rust Workspace

```
opal/
├── crates/
│   ├── opal-lexer/        # Token definitions + logos-based lexer
│   ├── opal-parser/       # AST types + recursive descent parser
│   ├── opal-macros/       # Macro expander (AST → AST)
│   ├── opal-interp/       # Tree-walk interpreter (Phase 1)
│   ├── opal-codegen/      # Core Erlang codegen (Phase 2, stub)
│   ├── opal-runtime/      # Values, environment, actors
│   ├── opal-stdlib/       # Standard library implementations
│   └── opal-cli/          # CLI entry point
├── tests/
│   ├── spec/              # Reference test suite (.opl files)
│   ├── fuzz/              # Fuzzing targets
│   └── bench/             # Benchmark programs
├── docs/                  # Existing spec docs
└── Cargo.toml             # Workspace manifest
```

---

## Vertical Slices

Each slice delivers a working program. Features are added only when a concrete program needs them.

### Slice 0: Project Scaffold & CI Pipeline

**Delivers:** Empty Rust workspace, CLI skeleton, CI pipeline, benchmark harness, reference test runner.

**What gets built:**
- Rust workspace with crate stubs
- `opal` CLI accepting `run`, `repl`, `test`, `bench` subcommands (all print "not yet implemented")
- GitHub Actions CI: build, test, clippy, format check
- Benchmark harness using `criterion` — runs on every push, stores results
- `tests/spec/` directory with test runner that executes `.opl` files and compares output
- Fuzzing setup with `cargo-fuzz` targeting lexer/parser

### Slice 1: Hello World — Strings & IO

**Target program:**
```opal
name = "Opal"
print(f"Hello, {name}!")
```

**Features:** Lexer (full), parser (expressions + assignment + calls), string literals (`"..."`, `'...'`, f-strings), `print()`, variables, basic evaluation.

**Why first:** Forces the full pipeline end-to-end: source → tokens → AST → eval → output.

### Slice 2: Calculator — Numbers & Functions

**Target program:**
```opal
def factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1) end
end

print(factorial(10))  # => 3628800
```

**Features:** Numbers (Int, Float), arithmetic, comparisons, `if`/`else`, function definitions with type annotations, recursion, `let`.

**First benchmarks:** Fibonacci and factorial.

### Slice 3: Data Cruncher — Collections & Closures

**Target program:**
```opal
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

evens = numbers.filter(|n| n % 2 == 0)
squares = evens.map(|n| n ** 2)
total = squares.reduce(0) do |acc, n|
  acc + n
end

print(f"Sum of even squares: {total}")  # => 220
```

**Features:** Lists, tuples, dicts, ranges, closures (both forms), `for`/`while`, iterators (`.map`, `.filter`, `.reduce`), pipe operator, pattern matching, destructuring.

**Hardest slice.** Collections + closures + pattern matching are the language's core.

### Slice 4: OOP Program — Classes & Modules

**Target program:**
```opal
module Shapes
  class Circle
    needs radius: Float

    def area() -> Float
      Math.pi() * .radius ** 2
    end
  end

  class Rectangle
    needs width: Float
    needs height: Float

    def area() -> Float
      .width * .height
    end
  end
end

from Shapes import Circle, Rectangle

shapes = [Circle.new(radius: 5.0), Rectangle.new(width: 3.0, height: 4.0)]
for shape in shapes
  print(f"Area: {shape.area()}")
end
```

**Features:** Classes, `needs` (DI), instance variables, modules, `from X import Y`, protocols, visibility, `self`, inheritance.

### Slice 5: Error Handling

**Target program:**
```opal
def divide(a: Float, b: Float) -> Result[Float, String]
  requires b != 0.0, "division by zero"
  Ok(a / b)
end

match divide(10.0, 3.0)
  case Ok(result)
    print(f"Result: {result}")
  case Error(msg)
    print(f"Error: {msg}")
end
```

**Features:** `try`/`catch`/`ensure`, `Result[T, E]`, `Option[T]`, `requires`, `raise`, custom exceptions, data-carrying enums.

### Slice 6: Concurrent System — Actors

**Target program:**
```opal
actor Counter
  def init()
    .count = 0
  end

  receive
    case :increment
      .count += 1
    case :get
      reply .count
  end
end

counter = Counter.new()
counter.send(:increment)
counter.send(:increment)
print(await counter.send(:get))  # => 2
```

**Features:** `actor`, `receive`/`case`, `send`, `reply`, `async`/`await`, supervisors, `parallel`. Actors run on Tokio tasks in Phase 1.

### Slice 7: Macro-Powered App

**Target program:**
```opal
macro unless(condition, body)
  ast
    if !($condition)
      $body
    end
  end
end

@unless false
  print("This prints!")
end

macroexpand do
  @unless false
    print("test")
  end
end
```

**Features:** `ast()` / `ast...end`, `$` interpolation, `$...` splats, `macro` definitions, `@name` invocation, `@[annotations]`, `esc()`, `macroexpand()`, `Expr` type. Self-hosting macros work.

### Slice 8: Web Server — The Capstone

**Target program:**
```opal
import OpalWeb

app = OpalWeb.App.new("my app")

@get "/" do
  "Hello, world!"
end

@get "/users/:id" do |params|
  user = Database.find(User, params.id)
  match user
    case Some(u) then u.to_json()
    case None then {status: 404, body: "Not found"}
  end
end

app.run!()
```

**Features:** Web framework as macro DSL, HTTP server (wrapping Rust HTTP library via FFI), JSON, routing. The pretotype works.

---

## Parser Strategy

**Hand-written recursive descent** from day one:

1. The BNF (95 rules in `Opal.md`) maps directly to parser functions
2. Opal needs excellent error messages — critical for a new language
3. The macro system needs tight parser integration
4. Generated parsers would be replaced eventually anyway

**Lexer:** `logos` crate — generates a DFA from Rust enum variants. Zero-copy, very fast.

**Parser:** One function per BNF rule. Returns `Result<AstNode, ParseError>` with span information. Panic-mode error recovery (skip to next statement boundary).

**AST:** Arena-allocated for performance. Each node carries a `Span` for error reporting and source mapping.

---

## Validation Pipeline

### Reference Test Suite

```
tests/spec/
├── 01-basics/
│   ├── hello_world.opl          # expect: Hello, world!
│   ├── string_interpolation.opl
│   └── ...
├── 02-control-flow/
├── 03-functions/
├── errors/                       # Programs that SHOULD fail
│   ├── type_error.opl           # expect-error: TypeError
│   └── ...
└── programs/                     # Full integration tests
    ├── calculator.opl
    └── web_server.opl
```

Each `.opl` file has a header comment:
```opal
# expect: Hello, Opal!
name = "Opal"
print(f"Hello, {name}!")
```

Test runner parses headers, runs files, compares output.

### Fuzzing

- `cargo-fuzz` targets for lexer and parser from Slice 0
- Property-based tests via `proptest`: valid token sequences never crash the parser
- Grammar-aware fuzzing after Slice 3 (generate programs from BNF)

### Performance Benchmarks

```
tests/bench/
├── micro/
│   ├── fibonacci.opl        # Recursive fib(35)
│   ├── sorting.opl          # Sort 10K elements
│   ├── pattern_match.opl    # 1M pattern matches
│   └── actor_pingpong.opl   # 100K actor messages
└── macro/
    ├── json_parse.opl       # Parse 1MB JSON
    └── web_requests.opl     # Handle 1K HTTP requests
```

Run on every push via CI. Results stored in repo. Regressions >10% fail the build.

---

## Technology Choices

| Component | Choice | Why |
|-----------|--------|-----|
| Implementation language | Rust | Industry standard for lang tooling. Memory-safe, fast, good ecosystem |
| Lexer | `logos` | DFA from Rust enums. Zero-copy, very fast, well-maintained |
| Parser | Hand-written recursive descent | Best error messages, full control, BNF maps directly to code |
| Async runtime | `tokio` | For actor scheduling in Phase 1 interpreter |
| CLI framework | `clap` | Standard Rust CLI library |
| Benchmarks | `criterion` | Statistical benchmarking for Rust |
| Fuzzing | `cargo-fuzz` + `libfuzzer` | Coverage-guided fuzzing |
| Property testing | `proptest` | Property-based testing |
| CI | GitHub Actions | Standard, free for open source |
| Phase 2 target | BEAM (Core Erlang) | Actors, supervisors, fault-tolerance built-in |

---

## BEAM Transition (Phase 2 Preview)

After Slice 8 validates the language, Phase 2 adds:

1. **`opal-codegen`** — Opal AST → Core Erlang AST → Core Erlang text
2. **Erlang compilation** — `erlc` compiles Core Erlang → BEAM bytecode
3. **OTP integration** — actors → Erlang processes, supervisors → OTP supervisors
4. **Hex interop** — Opal packages can depend on Erlang/Elixir libraries
5. **Performance jump** — from Rust threads to millions of BEAM processes

Phase 2 is a separate design doc when we get there. The architecture ensures frontend reuse.

---

## What This Does NOT Include (YAGNI)

- Tree-sitter grammar (add after grammar stabilizes, ~Slice 4)
- Language server / LSP (add after Phase 1)
- Formatter / linter (add after Phase 1)
- Package manager (add in Phase 2)
- LLVM backend (Phase 3 if ever needed)
- WASM target (future consideration)
- Self-hosting compiler (far future)

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Grammar ambiguity discovered during parsing | Reference test suite catches issues early. BNF is already written and reviewed |
| Macro system too complex for tree-walk interpreter | Slice 7 specifically stress-tests macros. If needed, simplify macro semantics |
| Actor concurrency bugs in interpreter | Tokio provides solid foundations. Actors are isolated by design |
| BEAM transition is harder than expected | Frontend is clean and separable. Core Erlang is well-documented. Gleam's source code is a reference |
| Performance regressions | CI benchmark suite catches regressions on every commit |
| Scope creep | Vertical slices enforce "only build what the current program needs" |
