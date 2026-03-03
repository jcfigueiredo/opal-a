# Syntax Familiarity Redesign

## Goal

Reduce friction for developers coming from Python, Ruby, and Rust by aligning 5 high-surprise syntax patterns with mainstream conventions, while preserving Opal's identity.

## Changes

### 1. Generics: `List(T)` → `List[T]`

Parentheses for generics look like function calls. Square brackets are used by Python, TypeScript, C#, Kotlin, and Scala.

```opal
# Before
class Stack(T)
  needs items::List(T)
end
s = Stack(Int32).new(items: [])
type Result(T, E) = T | Error

# After
class Stack[T]
  needs items: List[T]
end
s = Stack[Int32].new(items: [])
type Result[T, E] = T | Error
```

No ambiguity with indexing: types are PascalCase (`Stack[Int32]`), values are snake_case (`list[0]`).

### 2. Type Annotations: `::` → `:`

Every mainstream typed language uses single colon. Double-colon is unnecessary friction.

```opal
# Before
name::String = "claudio"
def add(a::Int32, b::Int32) -> Int32
needs db::Database

# After
name: String = "claudio"
def add(a: Int32, b: Int32) -> Int32
needs db: Database
```

No ambiguity with named arguments: definition context (`param: Type`) vs call site context (`param: value`) — same as Python, Swift, Kotlin.

### 3. Imports: Brace-style → `from/import`

Opal is a dynamic language closer to Python/Ruby than Rust. `from X import Y` reads like English.

```opal
# Before
import Math.{abs, max}
import Math.{abs, max as maximum}
export Router.{get, post, put}

# After
from Math import abs, max
from Math import abs, max as maximum
export get, post, put from Router

# Multi-line selective (new)
from Math import (
  sin, cos, tan,
  sqrt, PI
)
```

Unchanged forms: `import Math`, `import Math.Vector`, `import Math as M`.

No type/value import distinction needed — Opal is dynamic, everything is a runtime value, and PascalCase convention already distinguishes types visually.

### 4. Function Types: `|Type| -> Type` → `Fn(Type) -> Type`

Pipe-delimited function types collide visually with closure syntax and union types.

```opal
# Before
transform::|Int32| -> Int32 = |x| x * 2
def apply(fn::|Int32, String| -> Bool)
type Handler = |Request, Response| -> Null

# After
transform: Fn(Int32) -> Int32 = |x| x * 2
def apply(fn: Fn(Int32, String) -> Bool)
type Handler = Fn(Request, Response) -> Null
```

`Fn` is a PascalCase type like all others. Pipes (`|x|`) remain exclusively for closure parameter syntax.

### 5. Closures: Drop `fn`, Keep Pipes + `do...end`

Three closure forms is too many. `fn(params) ... end` is redundant — `do |params| ... end` covers the same case.

```opal
# Inline (pipes) — unchanged
double = |x| x * 2
numbers.filter(|x| x > 0)

# Multi-line / stored (do...end replaces fn)
handler = do |request, response|
  user = authenticate(request)
  response.json(user)
end

# No-arg
setup = do
  load_config()
  init_db()
end

# Trailing block — unchanged
File.open("data.txt") do |f|
  process(f.read())
end
```

Two forms, clear roles: pipes for inline, `do...end` for everything else.

## Kept As-Is

- **Instance variables (`.name`)** — short, unambiguous, avoids `@` collision with macros
- **Error propagation (`!`)** — works well, `?` would collide with predicate methods

## Impact

These changes affect: BNF grammar, every code example in every doc, CLAUDE.md rules. The BNF changes are mechanical (swap `::` for `:`, `(` for `[` in type params, add `from/import` form, remove `fn` lambda form, add `Fn` type).
