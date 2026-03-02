# Symbols, Annotations & Keywords Redesign

## Goal

Enhance Opal's symbol system with optional typing, add a declarative annotation/metadata system distinct from macros, introduce optional compile-time actor message checking, and clean up reserved words — all backward compatible and cohesive.

## Architecture

Symbol sets become the connective tissue: they type-constrain symbols, drive exhaustiveness checking in match, and power the new actor `receives` declaration. Annotations (`@[...]`) provide inert metadata queryable at runtime and by macros at parse time, cleanly separated from macro invocation (`@name`). Reserved words are trimmed by removing `within` and making supervisor-specific keywords contextual.

---

## 1. Typed Symbols & Symbol Sets

### Problem

Symbols are untyped atoms. A function accepting `:ok | :error` has no way to express that constraint — the type is just `Symbol`, and typos like `:eror` only fail at runtime.

### Design

Symbol sets are type aliases over unions of symbol literals:

```opal
# Named symbol set
type Status = :ok | :error | :pending

# Use as type annotation
def handle(status::Status)
  match status
    case :ok      then print("success")
    case :error   then print("failure")
    case :pending then print("waiting")
  end
end

handle(:ok)       # works
handle(:unknown)  # TYPE ERROR: :unknown is not in Status
```

Inline symbol constraints (no named type needed):

```opal
def log(level:: :debug | :info | :warn | :error, message::String)
  print(f"[{level}] {message}")
end
```

### Rules

- `type Name = :a | :b | :c` defines a symbol set (a type alias of a union of symbol literals).
- Symbol sets participate in exhaustiveness checking — the compiler warns on incomplete match.
- `Symbol` remains the unconstrained type (accepts any symbol) — gradual typing.
- Symbol sets compose with existing type system features (unions, generics, constraints).
- Bare `:name` literals are still valid everywhere — no breakage.

### Symbol Sets vs Enums

Symbol sets are for **simple tags with no data**. Enums are for **data-carrying variants**:

```opal
# Symbol set — lightweight tags
type Direction = :north | :south | :east | :west

# Enum — data-carrying variants
enum Shape
  Circle(radius::Float64)
  Rect(width::Float64, height::Float64)
end
```

If you need to attach data to a variant, use `enum`. If you just need named constants, use a symbol set.

---

## 2. Annotations as Symbol-Keyed Metadata

### Problem

`@name` is exclusively macro invocation. There's no way to attach declarative metadata to classes, functions, or fields. Metadata (like "this is deprecated") isn't a code transformation — it's data about code. Forcing it through macros conflates two distinct concerns.

### Design

New annotation syntax `@[...]` for declarative metadata, distinct from `@name` macro invocation:

```opal
# Annotation — metadata, not code transformation
@[deprecated, since: "2.0", use: "new_method"]
def old_method()
  # ...
end

# Macro — actual code transformation (unchanged)
@memoize
def fibonacci(n::Int32) -> Int32
  if n <= 1 then n else fibonacci(n - 1) + fibonacci(n - 2) end
end
```

### The Distinction

| Syntax | Purpose | When | What it does |
|---|---|---|---|
| `@name args` | Macro invocation | Parse time | Transforms code (AST to AST) |
| `@[key: val, ...]` | Annotation | Never "runs" | Attaches metadata, queryable at runtime |

### Annotation Syntax

```opal
# Simple tag (boolean — presence means true)
@[deprecated]
def old_api() ... end

# Tag with values
@[deprecated, since: "2.0", replacement: "new_api"]
def old_api() ... end

# On classes
@[serializable, version: 3]
class User
  needs name::String
  needs email::String
end

# On fields
class Config
  @[env: "DATABASE_URL"]
  needs db_url::String

  @[env: "PORT", default: 8080]
  needs port::Int32
end

# Multiple annotations stack
@[deprecated, since: "1.5"]
@[experimental]
def risky_method() ... end
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

Macros can read annotations at parse time — annotations provide data, macros provide transformation:

```opal
@[json_field, name: "user_name"]
needs name::String

# Macro reads annotations during code generation
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
- `@name` remains exclusively macro invocation — unchanged.
- Annotations are inert — no code transformation.
- Annotations are queryable via `annotations()` at runtime and by macros at parse time.
- Annotations stack (multiple `@[...]` on the same target).
- Annotations apply to the immediately following declaration (def, class, needs, etc.).

### BNF Addition

```bnf
<annotation>    ::= "@[" <annot_entry> ("," <annot_entry>)* "]"
<annot_entry>   ::= IDENTIFIER
                   | IDENTIFIER ":" <expression>
```

---

## 3. Actor Message Typing

### Problem

Actor messages use bare symbols with no type checking. `.send(:gett, "key")` (typo) only fails at runtime. No way to know an actor's interface without reading its source.

### Design

Actors can optionally declare their message interface with `receives`:

```opal
actor Cache
  receives :get, :set, :delete

  receive
    case :get(key)
      reply .store[key]
    case :set(key, value)
      .store[key] = value
      reply :ok
    case :delete(key)
      .store.delete(key)
      reply :ok
  end
end

cache = Cache.new()
cache.send(:get, "user:1")     # OK
cache.send(:gett, "user:1")    # COMPILE WARNING: :gett not in Cache.receives
```

### Rules

- `receives :msg1, :msg2, ...` is optional — actors without it accept any symbol (backward compatible).
- When present, `.send()` calls are checked at compile time against the declared set.
- `receives` uses symbol sets under the hood.
- You can use a named symbol set: `receives Status` where `type Status = :ok | :error`.
- The `receive` block must handle all declared messages (exhaustiveness check).
- Queryable: `Cache.receives()` returns the symbol set.

### BNF Addition

```bnf
<actor_body>    ::= ("receives" <symbol_list> NEWLINE)?
                     (<needs_decl> | <function_def> | <receive_clause>)*
<symbol_list>   ::= SYMBOL ("," SYMBOL)*
                   | IDENTIFIER
```

---

## 4. Reserved Word Cleanup

### 4a. Remove `within` keyword

```opal
# Before
max_restarts 3 within 60

# After
max_restarts 3, 60
```

### 4b. Contextual supervisor keywords

`strategy`, `max_restarts`, `supervise` are only keywords inside `supervisor` blocks. Outside, they're normal identifiers:

```opal
strategy = "my game plan"  # fine — not inside a supervisor

supervisor AppSupervisor
  strategy :one_for_one    # keyword here
  max_restarts 3, 60       # keyword here
  supervise Worker.new()   # keyword here
end
```

### Summary

| Change | Effect |
|---|---|
| Remove `within` | -1 keyword |
| Contextual: `strategy`, `max_restarts`, `supervise` | 3 keywords scoped to supervisor blocks |

---

## Complete Example

```opal
type HttpMethod = :get | :post | :put | :delete | :patch
type LogLevel = :debug | :info | :warn | :error

@[deprecated, since: "2.0", use: "fetch_v2"]
def fetch(url::String, method::HttpMethod = :get) -> Response
  requires url.starts_with?("http")
  Net.request(method.to_string().upper(), url)
end

actor ApiGateway
  receives :request, :health_check, :shutdown

  def init(config)
    .config = config
    .request_count = 0
  end

  receive
    case :request(method::HttpMethod, path::String)
      .request_count += 1
      response = route(method, path)
      reply response

    case :health_check
      reply {status: :ok, requests: .request_count}

    case :shutdown
      log(:info, "Shutting down gateway")
      reply :ok
  end

  private def route(method::HttpMethod, path::String) -> Response
    # routing logic...
  end
end

@json_serializable
@[version: 3]
class ApiResponse
  @[json_field, name: "status_code"]
  needs status::Int32

  @[json_field, name: "body"]
  needs data::String

  @[json_field, skip_if_null: true]
  needs error::String?
end

supervisor GatewaySupervisor
  strategy :one_for_one
  max_restarts 5, 30

  supervise ApiGateway.new(config: load_config())
  supervise Logger.new(level: :info)
end

def log(level::LogLevel, message::String)
  match level
    case :debug then IO.println(f"[DEBUG] {message}")
    case :info  then IO.println(f"[INFO]  {message}")
    case :warn  then IO.println(f"[WARN]  {message}")
    case :error then IO.println(f"[ERROR] {message}")
  end
end
```

---

## What's New vs What Stays

| Feature | Before | After |
|---|---|---|
| Symbols | Untyped atoms | Optionally typed via symbol sets |
| Symbol sets | Didn't exist | `type Name = :a \| :b \| :c` |
| Annotations | Didn't exist | `@[key: val, ...]` on any declaration |
| Macros | `@name` | `@name` (unchanged) |
| Actor messages | Untyped | Optional `receives` declaration |
| `within` keyword | Required | Removed (comma args) |
| Supervisor keywords | Global | Contextual |

Everything is additive and backward compatible.
