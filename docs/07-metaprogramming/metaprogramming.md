# Metaprogramming

Opal's metaprogramming system is Julia-inspired, adapted to Opal's `end`-block syntax and `:symbol` conventions. It provides quoting, interpolation, macros, annotations, and AST manipulation as first-class features.

**Core principles:**

- **Hygienic by default.** Macro-introduced variables don't leak into the caller's scope. Explicit `esc()` to opt out.
- **Valid AST only.** Macros produce Opal AST nodes, not arbitrary text. No C-preprocessor-style pitfalls.
- **No generated functions.** Opal's multiple dispatch + macros covers the same ground -- YAGNI.
- **Subdomains as macro packages.** Users and Opal itself can define domain-specific extensions as packages of macros.

---

## Quoting -- Code as Data

Code is captured as `Expr` (AST node) using `quote ... end`. Inside a quote, `$` interpolates values.

### Basic Quoting

```opal
# Capture code as data
ast = quote x + y * 2 end
typeof(ast)   # => Expr
ast.head      # => :call
ast.args      # => [:+, :x, Expr(:call, :*, :y, 2)]

# Multi-line quoting
ast = quote
  x = 1
  y = 2
  x + y
end
```

### Interpolation

```opal
# Splice runtime values into the AST
name = :greet
message = "hello"
ast = quote
  def $name()
    print($message)
  end
end
# ast represents: def greet() print("hello") end

# Splat interpolation for lists
params = [:a, :b, :c]
ast = quote f($params...) end
# ast represents: f(a, b, c)
```

### Programmatic AST Construction

```opal
# Build AST without quoting
ast = Expr.new(:call, :+, 1, 2)
eval(ast)  # => 3

# Equivalent to:
ast = quote 1 + 2 end
eval(ast)  # => 3
```

Note: `eval()` is a metaprogramming primitive for evaluating AST at runtime. It operates on Opal's own `Expr` type, not arbitrary strings. It is intended for macro expansion and code generation, not for evaluating untrusted input.

### Rules

- `quote ... end` returns an `Expr` -- code as a manipulable data structure.
- `$expr` inside a quote splices the value of `expr` into the AST at construction time.
- `$list...` splats a list of expressions into argument position.
- `Expr.new(head, args...)` constructs AST nodes programmatically.
- `eval(expr)` evaluates an `Expr` at runtime (metaprogramming use only).

---

## Macros

Macros receive AST at parse time and return transformed AST. They are hygienic by default.

### Basic Macros

```opal
macro say_hello()
  quote
    print("Hello, world!")
  end
end

@say_hello  # => "Hello, world!"
```

### Macros with Arguments

```opal
macro say_hello(name)
  quote
    print(f"Hello, {$name}")
  end
end

@say_hello "claudio"  # => "Hello, claudio"
```

### Hygiene

Variables introduced inside a macro's `quote` are scoped to the macro -- they don't shadow or leak into the caller's scope.

```opal
macro measure(body)
  quote
    start = Time.now()
    result = $body
    elapsed = Time.since(start)
    print(f"Took {elapsed}")
    result
  end
end

# Safe — caller's 'start' is NOT shadowed
start = "hello"
@measure do
  expensive_operation()
end
print(start)  # still "hello"
```

### Escaping Hygiene

Use `esc(expr)` to explicitly inject an expression into the caller's scope:

```opal
macro define_var(name, value)
  quote
    $(esc(name)) = $value
  end
end

@define_var x, 42
print(x)  # => 42 (x exists in caller's scope because of esc)
```

### Debugging Macros

```opal
# See what a macro expands to without executing it
macroexpand(@measure do 1 + 1 end)
# => Expr representing the expanded code
```

### Rules

- `macro name(params) ... end` defines a macro. The body must return an `Expr`.
- `@name args` invokes a macro at parse time.
- `@[key: val]` attaches annotation metadata (see Annotations section below).
- `@` followed by an identifier is a macro; `@` followed by `[` is an annotation.
- Macros receive arguments as `Expr` (AST), not evaluated values.
- **Hygienic by default:** variables in macro quotes don't leak.
- `esc(expr)` escapes into the caller's scope (opt-in).
- `macroexpand(@name args)` shows expansion without executing.

---

## Annotations -- Declarative Metadata

Annotations attach metadata to declarations. They are distinct from macros: macros transform code, annotations describe it.

| Syntax | Purpose | When | What it does |
|---|---|---|---|
| `@name args` | Macro invocation | Parse time | Transforms code (AST to AST) |
| `@[key: val, ...]` | Annotation | Never "runs" | Attaches metadata, queryable at runtime |

### Annotation Syntax

```opal
# Simple tag (presence means true)
@[deprecated]
def old_api()
  # ...
end

# Tag with values
@[deprecated, since: "2.0", replacement: "new_api"]
def old_api()
  # ...
end

# On classes
@[serializable, version: 3]
class User
  needs name: String
  needs email: String
end

# On fields
class Config
  @[env: "DATABASE_URL"]
  needs db_url: String

  @[env: "PORT", default: 8080]
  needs port: Int32
end

# Multiple annotations stack
@[deprecated, since: "1.5"]
@[experimental]
def risky_method()
  # ...
end
```

### Querying Annotations

```opal
# On functions
annotations(old_api)
# => [{deprecated: true, since: "2.0", replacement: "new_api"}]

# On classes
User.annotations()
# => [{serializable: true, version: 3}]

# On fields
Config.field_annotations(:db_url)
# => [{env: "DATABASE_URL"}]

# Check for specific annotation
if :deprecated in annotations(old_api)
  print("This function is deprecated")
end
```

### Macros Reading Annotations

Macros can read annotations at parse time -- annotations provide data, macros provide transformation:

```opal
@[json_field, name: "user_name"]
needs name: String

# Macro reads field annotations during code generation
macro json_serializable(class_def)
  fields = class_def.needs_fields()
  for field in fields
    annots = field.annotations()
    json_name = if :json_field in annots
      annots[:json_field][:name]
    else
      field.name.to_string()
    end
    # ... use json_name in generated code
  end
end
```

### Built-in Annotations

| Annotation | Purpose |
|---|---|
| `@[deprecated]` | Mark as deprecated (compiler warning on use) |
| `@[deprecated, since: "X", use: "Y"]` | With migration info |
| `@[experimental]` | Mark as unstable API |
| `@[inline]` | Hint to inline this function |
| `@[todo, note: "..."]` | In-code TODO that tooling can collect |
| `@[test_only]` | Only available in test files (`.topl`) |

### Rules

- `@[...]` attaches metadata as a dict of symbols to values.
- `@name` remains exclusively macro invocation -- unchanged.
- Annotations are inert -- no code transformation.
- Annotations are queryable via `annotations()` at runtime and by macros at parse time.
- Annotations stack (multiple `@[...]` on the same target).
- Annotations apply to the immediately following declaration (`def`, `class`, `needs`, etc.).

---

## AST Reflection & Introspection

### Inspecting Expressions

```opal
ast = quote x + y * 2 end
ast.dump()
# Expr(:call, :+,
#   :x,
#   Expr(:call, :*, :y, 2))

ast.head       # => :call
ast.args       # => [:+, :x, Expr(:call, :*, :y, 2)]
ast.args[0]    # => :+ (the operator)
ast.args[1]    # => :x
```

### Transforming AST

```opal
def double_literals(expr: Expr)
  match expr
    case n: Int32
      n * 2
    case Expr(head, args)
      Expr.new(head, args.map(|a| double_literals(a))...)
    case other
      other
  end
end

ast = quote 1 + 2 * 3 end
doubled = double_literals(ast)
eval(doubled)  # => eval(2 + 4 * 6) => 26
```

### Runtime Introspection

```opal
# Introspect functions
methods(greet)         # => list of dispatch variants
typeof(greet)          # => Function
code_ast(greet)        # => the Expr representing the function body

# Introspect classes
User.fields()          # => [(:name, String), (:email, String), (:age, Int32)]
User.methods()         # => [:to_json, :from_json, :new, ...]
User.needs()           # => [(:db, Database), (:mailer, Mailer)]
User.implements()      # => [Printable, Comparable]
```

---

## Practical Macro Examples

### Code Generation -- JSON Serialization

```opal
macro json_serializable(class_def)
  fields = class_def.needs_fields()

  to_json = quote
    def to_json()
      JSON.object($(generate_field_pairs(fields)...))
    end
  end

  from_json = quote
    def self.from_json(data: String)
      parsed = JSON.parse(data)
      self.new($(generate_from_json(fields)...))
    end
  end

  class_def.add_methods(to_json, from_json)
end

@json_serializable
class User
  needs name: String
  needs email: String
  needs age: Int32
end

user = User.new(name: "claudio", email: "c@opal.dev", age: 15)
user.to_json()   # => {"name":"claudio","email":"c@opal.dev","age":15}
User.from_json("""{"name":"claudio","email":"c@opal.dev","age":15}""")
```

### DSL Creation -- Test Framework

```opal
macro test(name, body)
  quote
    try
      $body
      Test.pass($name)
    catch as e
      Test.fail($name, e.message)
    end
  end
end

macro describe(name, body)
  quote
    Test.group($name)
    $body
    Test.end_group()
  end
end

@describe "Math" do
  @test "addition" do
    assert_eq(2 + 2, 4)
  end

  @test "negative numbers" do
    assert_eq(-1 + 1, 0)
  end
end
```

This is how Opal's built-in test framework (`OpalTest` subdomain) is implemented under the hood.

### Debugging -- @debug Macro

```opal
macro debug(expr)
  name = string(expr)
  quote
    value = $expr
    print(f"Debug: {$name} = {value}")
    value
  end
end

x = 42
@debug x * 2 + 1  # => "Debug: x * 2 + 1 = 85"
```

### Memoization

```opal
macro memoize(fn_def)
  fn_name = fn_def.name
  quote
    _cache = {:}

    def $fn_name($(fn_def.params...))
      key = ($(fn_def.params...),)
      if _cache.has?(key)
        return _cache[key]
      end
      result = $(fn_def.body)
      _cache[key] = result
      result
    end
  end
end

@memoize
def fibonacci(n: Int32) -> Int32
  if n <= 1 then n else fibonacci(n - 1) + fibonacci(n - 2) end
end
```

---

## Self-Hosting Potential

With quoting + macros, some of Opal's own features could be defined in Opal itself. This doesn't mean they *must* be -- core keywords can stay in the parser for performance and clarity. But the macro system is powerful enough that users could build equivalent constructs.

### What Stays in the Parser (Core Syntax)

These are fundamental to the language and must be parsed natively:

- `def`, `class`, `module`, `actor`, `if`, `for`, `while`, `match`, `try`
- `quote`, `macro`, `$` (metaprogramming primitives)
- `=`, `.`, `:`, operators

### What Could Be Macros

These are essentially code transformations and could theoretically be implemented as macros:

- `needs` -- generates constructor injection
- `event` -- generates an immutable data class
- `emit` -- generates actor-based event dispatch
- `on` -- generates event handler registration
- `requires` -- generates pre-condition checks
- `supervisor` -- generates actor supervision setup

Whether they stay as keywords or become macros is an implementation decision. The key insight is that the macro system is *expressive enough* to define them.

---

## Domain Extension Guidelines

Opal's macro system enables **subdomains** -- packages of macros that extend the language for a specific problem domain. This is how Opal and its ecosystem grow without bloating the core language.

### What is a Subdomain?

A subdomain is a module that exports macros, providing domain-specific syntax and abstractions. It is a mini-language within Opal, tailored to a particular problem.

### Creating a Subdomain

A subdomain is a standard Opal module that exports macros:

```opal
# File: opal_web/macros.opl
module OpalWeb
  # Route definition DSL
  macro get(path, body)
    quote
      app.route("GET", $path, |req, res|
        $body
      end)
    end
  end

  macro post(path, body)
    quote
      app.route("POST", $path, |req, res|
        $body
      end)
    end
  end

  # Middleware DSL
  macro middleware(name, body)
    quote
      app.use($name, |req, res, next|
        $body
        next()
      end)
    end
  end
end
```

```opal
# Usage — the subdomain provides web-specific syntax
import OpalWeb

@middleware :logging do
  print(f"[{Time.now()}] {req.method} {req.path}")
end

@get "/" do
  res.send("Hello, world!")
end

@post "/users" do
  user = User.from_json(req.body)
  user.save()
  res.json(user.to_json())
end
```

### Subdomain Guidelines

**1. Name macros as verbs or nouns that read naturally at the call site.**

```opal
# Good — reads like a sentence
@get "/users" do ... end
@test "addition" do ... end
@memoize def fib(n) ... end

# Bad — unclear at the call site
@r "/users" do ... end
@m def fib(n) ... end
```

**2. One macro per concept. Don't overload a macro to do multiple things.**

```opal
# Good — separate macros for separate concepts
@get "/users" do ... end
@post "/users" do ... end

# Bad — one macro with a mode parameter
@route "GET", "/users" do ... end
```

**3. Macros should produce valid, inspectable code.**

```opal
# Always test with macroexpand
macroexpand(@get "/" do res.send("hello") end)
# Should produce clean, readable Opal
```

**4. Document what the macro expands to.**

Every macro should include a comment or doc showing the equivalent non-macro code:

```opal
# @get "/" do ... end
# expands to:
# app.route("GET", "/", |req, res| ... end)
```

**5. Prefer macros that compose with existing features.**

Macros should work with preconditions, pattern matching, DI, and events -- not bypass them:

```opal
# Good — composes with preconditions
@memoize
def sqrt(x: Float64) -> Float64
  requires x >= 0
  x ** 0.5
end

# Good — composes with needs
@json_serializable
class User
  needs name: String  # needs still works inside macro-processed class
end
```

**6. Subdomains should be importable and scoped.**

```opal
# Import a subdomain
import OpalWeb          # all macros available
import OpalWeb.{get, post}  # selective import

# Macros from different subdomains don't conflict
import OpalWeb
import OpalTest
# @get is from OpalWeb, @test is from OpalTest
```

### Opal's Own Subdomains

Opal's standard library uses this same model. Rather than hardcoding every feature, the stdlib provides subdomains:

| Subdomain | Provides | Macros |
|---|---|---|
| `Opal.Core` | Core language (parser-level) | None -- native syntax |
| `Opal.Test` | Testing framework | `@test`, `@describe`, `@assert` |
| `Opal.Web` | Web framework | `@get`, `@post`, `@middleware` |
| `Opal.Data` | Database/ORM | `@schema`, `@migration`, `@query` |
| `Opal.Bench` | Benchmarking | `@benchmark`, `@profile` |
| `Opal.Debug` | Debugging tools | `@debug`, `@trace`, `@breakpoint` |
| `Opal.Serial` | Serialization | `@json_serializable`, `@msgpack` |

Each subdomain is an independent package -- you only import what you use.

---

## Design Rationale

### What Opal Adapts from Julia

Opal's metaprogramming is directly inspired by Julia. The core model -- quoting, interpolation, `Expr` type, hygienic macros, `esc`, `macroexpand` -- maps closely. The adaptations are syntactic: Opal uses `quote ... end` instead of `:(...)` (which would conflict with `:symbol`), and Opal adds the `@[key: val]` annotation syntax which Julia does not have.

### What Opal Skips

- **`:(expr)` single-expression quoting** -- conflicts with `:symbol` syntax.
- **`@generated function`** -- YAGNI with multiple dispatch + macros covering the same ground.

### Julia Comparison Table

| Julia Feature | Opal Adaptation |
|---|---|
| `:(expr)` quoting | `quote expr end` / `quote ... end` |
| `$var` interpolation | `$var` (identical) |
| `Expr` type | `Expr` type with `.head`, `.args`, `.dump()` |
| `macro ... end` | `macro ... end` (identical structure) |
| `@name` invocation | `@name` (identical) |
| `eval()` | `eval()` (identical) |
| `esc()` | `esc()` (identical) |
| `macroexpand()` | `macroexpand()` (identical) |
| `@generated function` | Skipped -- multiple dispatch + macros covers it |
| Non-standard string literals | Already in Opal (`f"..."`, `r"..."`, `t"..."`) |
| (no equivalent) | `@[key: val]` annotation (metadata, not transformation) |

---

## Summary

### New Keywords

| Keyword | Purpose |
|---|---|
| `quote ... end` | Capture code as AST |
| `$` (inside quote) | Interpolate into AST |
| `macro ... end` | Define a macro |
| `@name` | Invoke a macro |
| `@[key: val]` | Attach metadata annotation |

Opal's metaprogramming gives users the same tools the language uses internally: quoting captures code as data, macros transform it at parse time, annotations attach metadata without transformation, and subdomains package macros into domain-specific extensions. The system is hygienic by default, produces valid AST only, and composes with all other language features.
