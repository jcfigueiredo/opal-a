# Opal — Opinionated Programming Algorithmic Language

[...towards a better programming...](http://www.chris-granger.com/2014/03/27/toward-a-better-programming/)

Opal is a dynamic, interpreted, object-oriented language with first-class functions, multiple dispatch, an actor-based concurrency model, and a gradual type system. It prioritizes readability, explicitness, and demonstrating sound software engineering concepts.

---

## 1. Design Philosophy

- **Readability is paramount.** Code is read far more than it is written.
- **One explicit way.** There should be one obvious way to do something — no alternative syntax for the same operation.
- **Software engineering concepts are first-class.** Dependency injection, domain events, specifications, preconditions, null objects, validated models, settings, the actor model, metaprogramming, annotations, and pipe-based composition are built into the language, not bolted on.
- **Batteries included.** Built-in testing, mocking, fixtures, documentation generation, project scaffolding, and package management.
- **Immutable by intent.** `let` bindings and immutable-by-default parameters support correctness.
- **Gradual typing.** Write quick scripts with no annotations, then add types at module boundaries for safety.

---

## 2. Facts & Semantics

| Question | Answer |
|---|---|
| Direct pointer access? | No. |
| Data types? | Rich: integers, floats, strings, booleans, null, symbols, lists, tuples, dicts, ranges, regex. |
| Static or dynamic? | Dynamic, interpreted. |
| Memory model? | Garbage collected (inherited from host runtime). |
| Concurrency model? | Actor model. |
| Primitives? | The least number of primitives as possible; most functionality comes from the standard library. |
| Paradigm? | Multi-paradigm: object-oriented with functional features, multiple dispatch, and actor concurrency. |
| FFI? | Placeholder `extern` syntax — runtime-dependent. See [FFI](#ffi). |
| Unicode support? | Full. Variable names, string literals, and symbols may contain Unicode characters. |
| File extensions? | `.opl` for source, `.topl` for tests. |

---

## 3. Formal Grammar (BNF Excerpt)

```bnf
<program>       ::= <statement>*

<statement>     ::= <expression> NEWLINE
                   | <assignment>
                   | <conditional>
                   | <loop>
                   | <function_def>
                   | <class_def>
                   | <module_def>
                   | <match_expr>
                   | <try_expr>
                   | <actor_def>
                   | <supervisor_def>
                   | <parallel_expr>
                   | <async_expr>
                   | <event_def>
                   | <emit_expr>
                   | <on_handler>
                   | <macro_def>
                   | <quote_expr>
                   | <type_alias>
                   | <implements_for>
                   | <enum_def>
                   | <model_def>
                   | <settings_def>
                   | <extern_def>
                   | <import_stmt>
                   | <export_stmt>

<assignment>    ::= IDENTIFIER "=" <expression>
                   | "let" IDENTIFIER "=" <expression>
                   | "let" <destructure>

<expression>    ::= <literal>
                   | IDENTIFIER
                   | <expression> <binary_op> <expression>
                   | <unary_op> <expression>
                   | <expression> "." IDENTIFIER
                   | <expression> "." IDENTIFIER "(" <args> ")"
                   | <expression> "?." IDENTIFIER
                   | <expression> "?." IDENTIFIER "(" <args> ")"
                   | <expression> "??" <expression>
                   | <expression> "[" <expression> "]"
                   | <expression> "in" <expression>
                   | <expression> "not" "in" <expression>
                   | <function_call>
                   | <lambda>
                   | <with_expr>
                   | <list_comp>
                   | <dict_comp>
                   | "(" <expression> ")"

<literal>       ::= INTEGER | FLOAT | STRING | BOOL | NULL
                   | SYMBOL | <list> | <tuple> | <dict> | <range> | <regex>
                   | <f_string> | <r_string> | <t_string>

<string>        ::= '"' ( STRING_CONTENT | ESCAPE_SEQ )* '"'
                   | "'" ( STRING_CONTENT | ESCAPE_SEQ )* "'"
                   | '"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE )* '"""'
<f_string>      ::= 'f"' ( STRING_CONTENT | ESCAPE_SEQ | "{" <expression> FORMAT_SPEC? "}" )* '"'
                   | 'f"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE | "{" <expression> FORMAT_SPEC? "}" )* '"""'
<r_string>      ::= 'r"' RAW_CONTENT* '"'
                   | 'r"""' RAW_CONTENT* '"""'
<t_string>      ::= 't"' ( STRING_CONTENT | ESCAPE_SEQ | "{" <expression> "}" )* '"'
                   | 't"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE | "{" <expression> "}" )* '"""'
<format_spec>   ::= "=" | ":" FORMAT_CONTENT

<list>          ::= "[" <expression> ("," <expression>)* "]"
                   | "[" "]"
<tuple>         ::= "(" <expression> ("," <expression>)* ")"
                   | "()"
<dict>          ::= "{" <dict_entry> ("," <dict_entry>)* "}"
                   | "{:}"
<dict_entry>    ::= <expression> ":" <expression>
<list_comp>     ::= "[" <expression> "for" IDENTIFIER "in" <expression>
                     ("for" IDENTIFIER "in" <expression>)*
                     ("if" <expression>)? "]"
<dict_comp>     ::= "{" <expression> ":" <expression> "for" IDENTIFIER "in" <expression>
                     ("if" <expression>)? "}"
<range>         ::= <expression> ".." <expression>
                   | <expression> "..." <expression>
<regex>         ::= "/" REGEX_CONTENT "/" REGEX_FLAGS?

<symbol>        ::= ":" IDENTIFIER
                   | ":" '"' STRING_CONTENT '"'

<function_call> ::= IDENTIFIER "(" <args> ")" <trailing_block>?
                   | <expression> "." IDENTIFIER "(" <args> ")" <trailing_block>?
                   | IDENTIFIER <trailing_block>
<trailing_block>::= "do" ("|" <params> "|")? NEWLINE <block> "end"
<args>          ::= <arg> ("," <arg>)*
<arg>           ::= <expression>
                   | IDENTIFIER ":" <expression>

<lambda>        ::= "|" <params> "|" <expression>
                   | "|" <params> "|" NEWLINE <block> "end"
                   | "do" <expression> "end"
                   | "do" NEWLINE <block> "end"
                   | "do" "|" <params> "|" NEWLINE <block> "end"
                   | "fn" "(" <params> ")" <expression> "end"
                   | "fn" "(" <params> ")" NEWLINE <block> "end"

<with_expr>     ::= <expression> "with" <dict>

<function_def>  ::= <annotation>* "def" IDENTIFIER "(" <params> ")" ("->" <type_expr>)? NEWLINE <block> "end"
<params>        ::= <param> ("," <param>)*
<param>         ::= IDENTIFIER
                   | IDENTIFIER "::" TYPE
                   | IDENTIFIER "=" <expression>

<conditional>   ::= "if" <expression> NEWLINE <block>
                     ("elsif" <expression> NEWLINE <block>)*
                     ("else" NEWLINE <block>)? "end"

<loop>          ::= "while" <expression> NEWLINE <block> "end"
                   | "for" IDENTIFIER "in" <expression> NEWLINE <block> "end"

<destructure>   ::= "(" <destruct_target> ("," <destruct_target>)* ")" "=" <expression>
                   | "{" <dict_destruct> ("," <dict_destruct>)* "}" "=" <expression>
                   | "[" <destruct_target> ("|" IDENTIFIER)? "]" "=" <expression>
<destruct_target>::= IDENTIFIER | "_" | "(" <destruct_target> ("," <destruct_target>)* ")"
<dict_destruct> ::= IDENTIFIER ":" IDENTIFIER | IDENTIFIER "?:" IDENTIFIER

<operator_def>  ::= "def" OPERATOR "(" <params> ")" NEWLINE <block> "end"

<class_body>    ::= (<needs_decl> | <function_def> | <operator_def> | <assignment>)*

<module_def>    ::= "module" IDENTIFIER NEWLINE <module_body> "end"
<module_body>   ::= (<needs_decl> | <function_def> | <class_def> | <assignment> | <on_handler>)*

<match_expr>    ::= "match" <expression> NEWLINE <case_clause>+ "end"
<case_clause>   ::= "case" <pattern> NEWLINE <block>
<pattern>       ::= <literal>
                   | IDENTIFIER
                   | "_"
                   | IDENTIFIER "::" TYPE
                   | <tuple_pattern>
                   | <list_pattern>
                   | <dict_pattern>
                   | <enum_pattern>
                   | <pattern> "|" <pattern>
                   | <pattern> "as" IDENTIFIER
                   | <pattern> "if" <expression>
<tuple_pattern>  ::= "(" <pattern> ("," <pattern>)* ")"
<list_pattern>   ::= "[" "]"
                    | "[" <pattern> ("," <pattern>)* "]"
                    | "[" <pattern> "|" IDENTIFIER "]"
<dict_pattern>   ::= "{" <dict_pat_entry> ("," <dict_pat_entry>)* "}"
<dict_pat_entry> ::= IDENTIFIER ":" <pattern>
<enum_pattern>   ::= <module_path> "." IDENTIFIER ("(" <pattern> ("," <pattern>)* ")")?

<try_expr>      ::= "try" NEWLINE <block>
                     ("catch" TYPE ("as" IDENTIFIER)? NEWLINE <block>)*
                     ("catch" ("as" IDENTIFIER)? NEWLINE <block>)?
                     ("ensure" NEWLINE <block>)?
                     "end"

<actor_def>     ::= "actor" IDENTIFIER NEWLINE <actor_body> "end"
<actor_body>    ::= ("receives" <symbol_list> NEWLINE)?
                     (<needs_decl> | <function_def> | <receive_clause>)*
<symbol_list>   ::= SYMBOL ("," SYMBOL)*
                   | IDENTIFIER
<receive_clause>::= "receive" NEWLINE <case_clause>+ "end"

<supervisor_def>::= "supervisor" IDENTIFIER NEWLINE <supervisor_body> "end"
<supervisor_body>::= ("strategy" SYMBOL NEWLINE)?
                     ("max_restarts" INTEGER "," INTEGER NEWLINE)?
                     ("supervise" <expression> NEWLINE)*

<parallel_expr> ::= "parallel" NEWLINE <block> "end"
                   | "parallel" ("max:" INTEGER)? "for" IDENTIFIER "in" <expression> NEWLINE <block> "end"

<async_expr>    ::= "async" <expression>
<await_expr>    ::= "await" <expression>

<needs_decl>    ::= <annotation>* "needs" IDENTIFIER "::" TYPE ("=" <expression>)?
<event_def>     ::= "event" IDENTIFIER "(" <params> ")"
<emit_expr>     ::= "emit" <expression> ("await")?
<on_handler>    ::= "on" TYPE "do" "|" IDENTIFIER "|" NEWLINE <block> "end"

<macro_def>     ::= "macro" IDENTIFIER "(" <params> ")" NEWLINE <block> "end"
<macro_invoke>  ::= "@" IDENTIFIER <args>?
<annotation>    ::= "@[" <annot_entry> ("," <annot_entry>)* "]"
<annot_entry>   ::= IDENTIFIER
                   | IDENTIFIER ":" <expression>
<quote_expr>    ::= "quote" <expression> "end"
                   | "quote" NEWLINE <block> "end"

<type_alias>    ::= "type" IDENTIFIER ("[" <type_params> "]")? "=" <type_expr>
<type_expr>     ::= TYPE
                   | <type_expr> "|" <type_expr>
                   | TYPE "[" <type_args> "]"
                   | TYPE "?"
                   | "|" <type_list> "|" "->" <type_expr>
<type_params>   ::= <type_param> ("," <type_param>)*
<type_param>    ::= IDENTIFIER ("implements" TYPE ("," TYPE)*)?
<type_args>     ::= <type_expr> ("," <type_expr>)*
<where_clause>  ::= "where" <constraint> ("," <constraint>)*
<constraint>    ::= IDENTIFIER "implements" TYPE ("," TYPE)*

<implements_for>::= "implements" TYPE "for" TYPE NEWLINE <class_body> "end"

<enum_def>      ::= "enum" IDENTIFIER ("[" <type_params> "]")?
                     ("implements" TYPE ("," TYPE)*)? NEWLINE
                     <variant>+ <function_def>* "end"
<variant>       ::= IDENTIFIER ("(" <params> ")")?

<model_def>     ::= "model" IDENTIFIER ("[" <type_params> "]")? NEWLINE
                     (<needs_decl> | <where_field> | <validate_block> | <function_def>)* "end"
<settings_def>  ::= "settings" "model" IDENTIFIER ("[" <type_params> "]")? NEWLINE
                     (<needs_decl> | <where_field> | <validate_block> | <function_def>)* "end"
<where_field>   ::= "needs" IDENTIFIER "::" TYPE ("=" <expression>)?
                     "where" <field_constraint> ("," <field_constraint>)*
<field_constraint> ::= <lambda> | IDENTIFIER ("(" <args> ")")?
<validate_block>::= "validate" "do" NEWLINE <block> "end"

<extern_def>    ::= "extern" STRING NEWLINE <extern_decl>* "end"
<extern_decl>   ::= "def" IDENTIFIER "(" <params> ")" ("->" <type_expr>)? NEWLINE

<import_stmt>   ::= "import" <module_path>
                   | "import" <module_path> "as" IDENTIFIER
                   | "import" <module_path> ".{" <import_list> "}"
<import_list>   ::= <import_item> ("," <import_item>)*
<import_item>   ::= IDENTIFIER
                   | IDENTIFIER "as" IDENTIFIER
<module_path>   ::= IDENTIFIER ("." IDENTIFIER)*
<export_stmt>   ::= "export" <module_path> ".{" <import_list> "}"

<is_expr>       ::= <expression> "is" TYPE
<propagate_expr>::= <expression> "!"
<requires_expr> ::= "requires" <expression> ("," STRING)?

<class_def>     ::= <annotation>* "class" IDENTIFIER ("[" <type_params> "]")? ("<" IDENTIFIER)?
                     (<where_clause>)? NEWLINE <class_body> "end"
<null_object_def> ::= "class" IDENTIFIER "<" IDENTIFIER "defaults" <dict>

<block>         ::= <statement>+

<binary_op>     ::= "+" | "-" | "*" | "/" | "%" | "**"
                   | "==" | "!=" | "<" | ">" | "<=" | ">="
                   | "and" | "or"
                   | "in" | "not" "in"
                   | ".." | "..."
                   | "|>"
<unary_op>      ::= "-" | "not"
```

---

## 4. Basics

### Comments

Single-line comments begin with `#`. Multiline comments are delimited by `###`.

```opal
# single-line comment
###
  multiline comment
###
```

> See [Comments](docs/01-basics/comments.md) for the full specification.

### Variables & Assignment

Variables are dynamically typed. `let` creates immutable bindings. Unicode identifiers supported.

```opal
name = "claudio"        # mutable
let pi = 3.14           # immutable
x, y = 1, 2             # parallel assignment
```

> See [Variables & Assignment](docs/01-basics/variables-and-assignment.md) for destructuring, type annotations, and naming conventions.

### Literals

Rich literal types: integers, floats, strings (single/double quotes, f/r/t prefixes, triple-quoted), booleans, null, and symbols. Symbol sets (`type Status = :ok | :error`) provide typed enumerations.

```opal
total = 42
price = 22.3
greeting = f"Hi {name}"
path = r"C:\raw\path"
status = :ok
```

> See [Literals](docs/01-basics/literals.md) for numeric semantics, string methods, escape sequences, and symbol sets.

### Operators

Arithmetic, comparison, logical, membership (`in`/`not in`), pipe (`|>`), null-safe chaining (`?.`), null coalescing (`??`). Operators are overloadable methods.

```opal
result = data |> parse |> validate |> format
city = user?.address?.city ?? "Unknown"
```

> See [Operators](docs/01-basics/operators.md) for operator overloading, pipe semantics, and the full operator table.

### Collections

Lists (mutable, ordered), tuples (immutable), dicts (mutable key-value), and ranges. Comprehensions provide concise construction with filtering.

```opal
numbers = [1, 2, 3, 4, 5]
point = (10, 20)
ages = {"alice": 30, "bob": 25}
squares = [x ** 2 for x in 1..10 if x % 2 == 0]
```

> See [Collections](docs/01-basics/collections.md) for collection methods, comprehensions, ranges, and regex.

### Destructuring

Pattern-based unpacking for tuples, lists (head/tail), and dicts. Works in assignment, function parameters, for loops, and closures.

```opal
(x, y) = get_point()
[head | tail] = [1, 2, 3, 4]
{name: n, age: a} = person_dict
```

> See [Destructuring](docs/01-basics/destructuring.md) for all destructuring forms and rules.

---

## 5. Control Flow

### Conditionals

`if`/`elsif`/`else`/`end` blocks, suffix form for single expressions, and inline ternary style.

```opal
status = if active then "on" else "off" end
print("even") if n % 2 == 0
```

> See [Conditionals](docs/02-control-flow/conditionals.md) for the full specification.

### Loops & Iteration

`while` and `for...in` loops with `break` and `next`. Supports indexed iteration via `.with_index()`.

```opal
for item, index in names.with_index()
  print(f"{index}: {item}")
end
```

> See [Loops & Iteration](docs/02-control-flow/loops-and-iteration.md) for the full specification.

### Pattern Matching

`match` expressions with literal, type, tuple, list, dict, enum, or-pattern, guard, and as-binding patterns. Exhaustive matching enforced for enums and symbol sets.

```opal
match shape
  case Shape.Circle(r)      then Math.PI * r ** 2
  case Shape.Rectangle(w, h) then w * h
  case _                     then 0.0
end
```

> See [Pattern Matching](docs/02-control-flow/pattern-matching.md) for all pattern forms, nesting, guards, and exhaustiveness rules.

---

## 6. Functions & Types

### Functions & Closures

Functions defined with `def` are first-class values with optional type annotations and default arguments. Closures use `|params| body`, `do...end`, or `fn(params) ... end` syntax. Closures capture by reference.

```opal
def greet(name::String) -> String
  f"Hello, {name}!"
end

double = |x| x * 2
numbers.each do |x| print(x) end
```

> See [Functions & Closures](docs/03-functions-and-types/functions-and-closures.md) for capture semantics, trailing blocks, closure types, and `fn` syntax.

### Type System

Gradual typing: unannotated code is dynamic, `::` annotations are checked at boundaries. Supports generics, constraints, union types, type aliases, and runtime introspection with `is` and `typeof`.

```opal
def add(a::Int32, b::Int32) -> Int32
  a + b
end

type Result[T] = T | Error
```

> See [Type System](docs/03-functions-and-types/type-system.md) for generics, constraints, union types, aliases, and introspection.

### Classes & Inheritance

Classes use `needs` for dependency injection and `def init()` for initialization. Instance variables use `.` prefix. Single inheritance with `<`. Constructor shorthand: `Type(args)` is sugar for `Type.new(args)`.

```opal
class Dog < Animal
  needs breed::String

  def speak()
    print(f"Woof! I'm a {.breed}")
  end
end
```

> See [Classes & Inheritance](docs/03-functions-and-types/classes-and-inheritance.md) for construction order, inherited needs, `super`, and class rules.

### Modules & Imports

Each `.opl` file implicitly defines a module (PascalCase). Directories create hierarchies. Imports are absolute, with selective import and aliasing. Re-exports via `export`.

```opal
import Math.{abs, max}
import Math.Vector as Vec
```

> See [Modules & Imports](docs/03-functions-and-types/modules-and-imports.md) for file-to-module mapping, packages, collision rules, and re-exports.

### Visibility

Default is `public`. Mark methods `private` (same class/module only) or `protected` (class + subclasses). Applies to methods, functions, classes, and constants.

> See [Visibility](docs/03-functions-and-types/visibility.md) for the full visibility rules and summary table.

### Protocols

Protocols define contracts with required methods (no body) and default methods (with body). Nominal typing -- classes must declare `implements`. Retroactive conformance adds protocols to types you don't own.

```opal
protocol Printable
  def to_string() -> String
  def print()
    IO.print(.to_string())
  end
end
```

> See [Protocols](docs/03-functions-and-types/protocols.md) for generic protocols, retroactive conformance, and conflict resolution.

### Multiple Dispatch

Functions can have multiple definitions that dispatch based on argument types, arity, and precondition guards. Ambiguity is a compile-time error.

```opal
def render(shape::Circle)  then draw_circle(shape)
def render(shape::Rectangle) then draw_rect(shape)
```

> See [Multiple Dispatch](docs/03-functions-and-types/multiple-dispatch.md) for resolution order and dispatch with preconditions.

### Iterators

Two protocols: `Iterable` (provides `iter()`) and `Iterator[T]` (provides `next() -> Option[T]`). Any `Iterable` works with `for...in` and collection methods.

> See [Iterators](docs/03-functions-and-types/iterators.md) for custom iterators, lazy sequences, and the full protocol.

### Enums & Algebraic Data Types

`enum` defines a closed set of variants -- simple constants or data-carrying. Exhaustive pattern matching enforced. Enums support methods, protocols, and type parameters.

```opal
enum Shape
  Circle(radius::Float64)
  Rectangle(width::Float64, height::Float64)
end
```

> See [Enums & Algebraic Data Types](docs/03-functions-and-types/enums.md) for generic enums (`Option`, `Result`), methods, and matching rules.

### Models & Settings

`model` defines validated, immutable data with `where` constraints and automatic serialization. `settings model` adds configuration loading from env vars, config files, and `.env` files.

```opal
model User
  needs name::String where |v| v.length > 0
  needs email::String where valid_email?
  needs age::Int32 where |v| v >= 0
end
```

> See [Models & Settings](docs/03-functions-and-types/models-and-settings.md) for field validation, serialization, cross-field validation, and settings loading.

### FFI

Placeholder `extern` syntax for calling functions from external shared libraries. Signatures must be fully typed. Calling convention and type mapping are runtime-dependent.

> See [FFI](docs/03-functions-and-types/ffi.md) for the full specification.

---

## 7. Error Handling & Safety

### Error Handling

Two-track model: **exceptions** (`fail`/`try`/`catch`/`ensure`) for unexpected errors, **Result types** (`Result[T, E]`) for expected errors. The `!` operator propagates `Err` from the enclosing function. `Result.from do...end` bridges exceptions into Result values.

```opal
def process(path::String) -> Result[Config, Error]
  content = read_file(path)!
  config = parse_json(content)!
  Result.Ok(config)
end
```

> See [Error Handling](docs/04-error-handling/error-handling.md) for custom errors, Result helpers, and bridging.

### Preconditions

`requires` validates conditions at function entry. Reusable validators (functions returning `Bool`) work in both `requires` and model `where` clauses.

```opal
def sqrt(value::Float64) -> Float64
  requires value >= 0, "sqrt requires non-negative input"
  value ** 0.5
end
```

> See [Preconditions](docs/04-error-handling/preconditions.md) for reusable validators and rules.

### Null Objects

Null objects provide default behavior instead of null checks. The `defaults` shorthand auto-generates a subclass with default values.

```opal
class AnonymousPerson < Person defaults {name: "anonymous", age: 0}
```

> See [Null Objects](docs/04-error-handling/null-objects.md) for the full and shorthand forms.

---

## 8. Concurrency

Opal's concurrency model has four layers: **actors** for stateful concurrent entities, **parallel blocks** for structured concurrency, **async/futures** for individual non-blocking calls, and **supervisors** for fault tolerance. All calls are sync by default -- there are no colored functions. Any expression can be made async at the call site with `async`.

| Need | Tool | Syntax |
|---|---|---|
| Stateful concurrent entity | Actor | `actor`, `receive` with `case`, `.send()` |
| Run N things concurrently, wait for all | Parallel block | `parallel ... end` |
| Run N items concurrently | Parallel for | `parallel for x in xs ... end` |
| Limit concurrency | Parallel max | `parallel max: N for ...` |
| Make one call non-blocking | Async/Future | `async expr`, auto-await on use |
| Fault tolerance | Supervisor | `supervisor`, `strategy`, `supervise` |

> See [Concurrency](docs/05-concurrency/concurrency.md) for actors, parallel blocks, async/futures, supervisors, and lifecycle hooks.

---

## 9. Software Engineering Patterns

### Dependency Injection & Events

`needs` declares dependencies with protocol/type constraints, injected at construction via `.new()`. Events are declared with `event`, dispatched with `emit`, and handled with `on`. An optional `Container` class resolves dependencies automatically for large apps.

```opal
class OrderService
  needs db::Database
  needs mailer::Mailer

  def place_order(order)
    .db.save(order)
    emit OrderPlaced(order: order, placed_at: Time.now())
  end
end
```

> See [Dependency Injection](docs/06-patterns/dependency-injection.md) for DI rules, events, Container, and a complete DDD example.

### Specifications

The specification pattern enables composable business rules via `Specification` base class with logical combinators (`.and()`, `.or()`, `.not()`).

> See [Specifications](docs/06-patterns/specifications.md) for the full specification pattern.

---

## 10. Metaprogramming

Opal's metaprogramming is Julia-inspired: `quote...end` captures code as `Expr` AST nodes, `$` interpolates values, `macro...end` defines hygienic macros invoked with `@name`. Annotations (`@[key: val]`) attach queryable metadata to declarations. Subdomains are macro packages that extend the language for specific problem domains.

```opal
macro memoize(fn_def)
  quote
    _cache = {:}
    def $(fn_def.name)($(fn_def.params...))
      key = ($(fn_def.params...),)
      if _cache.has?(key) then return _cache[key] end
      result = $(fn_def.body)
      _cache[key] = result
      result
    end
  end
end
```

> See [Metaprogramming](docs/07-metaprogramming/metaprogramming.md) for quoting, macros, annotations, AST reflection, and subdomain guidelines.

---

## 11. Standard Library

| Module | Purpose |
|---|---|
| `IO` | Standard input/output: `print()`, `println()`, `read_line()`, `read_all()` |
| `File` | File operations: `read()`, `write()`, `exists?()`, `delete()`, `list_dir()` |
| `Net` | HTTP client/server, TCP/UDP sockets |
| `Math` | Mathematical functions: `abs()`, `max()`, `min()`, `sqrt()`, `sin()`, `cos()`, constants (`PI`, `E`) |
| `Collections` | Advanced data structures: Set, Queue, Stack, PriorityQueue |
| `String` | String manipulation: `split()`, `join()`, `trim()`, `replace()`, `upper()`, `lower()` |
| `Time` | Date, time, duration: `Time.now()`, `Time.parse()`, `Duration`, formatting |
| `JSON` | JSON parsing and generation: `JSON.parse()`, `JSON.generate()`, streaming |
| `Test` | Built-in test framework -- `@describe`, `@test`, assertions, lifecycle hooks |
| `Mock` | Mock creation for tests -- `Mock.new(Protocol)`, stubs, call verification |
| `Spec` | Specification pattern base classes |
| `Container` | Optional dependency injection container for large apps |
| `Iter` | `Iterable` and `Iterator[T]` protocols, lazy sequences |
| `Option` | `Option[T]` enum -- `Some(value)` or `None` for explicit nullable handling; used by `Iterator[T]` |
| `Result` | `Result[T, E]` enum -- `Ok(value)` or `Err(error)` for error handling |
| `Settings` | Base for `settings model` definitions -- env/config/file loading with source priority |
| `Reflect` | Runtime introspection: `annotations()`, `field_annotations()`, `typeof()`, `methods()` |

> See [Standard Library](docs/08-stdlib/stdlib.md) for usage examples and module details.

---

## 12. Tooling

Opal provides a unified CLI (`opal`) for all development tasks. Formatting is opinionated and zero-configuration. Packages use `opal.toml` manifests with semantic versioning.

| Command | Purpose |
|---|---|
| `opal run` | Run a program |
| `opal test` | Run tests (`.topl` files) |
| `opal fmt` | Format code (opinionated, zero-config) |
| `opal lint` | Lint code for errors and warnings |
| `opal docs` | Generate documentation |
| `opal init` | Scaffold a new project |
| `opal pkg add` | Add a dependency |
| `opal pkg install` | Install all dependencies |
| `opal pkg remove` | Remove a dependency |

> See [Tooling](docs/09-tooling/tooling.md) for testing framework, assertions, mocking, project scaffolding, formatter, linter, and package management.

---

## 13. Pretotyping

([No, it's not a typo.](http://www.pretotyping.org/))

Opal aims to make simple web applications as concise as possible, comparing favorably with frameworks like Flask while providing built-in routing macros and a batteries-included web framework.

> See [Pretotyping](docs/10-examples/pretotyping.md) for comparison examples between Opal and other languages.

---

## Appendix

Links, references, tutorials, and implementation ideas for building the Opal runtime.

> See [Appendix](docs/appendix/appendix.md) for all reference materials.
