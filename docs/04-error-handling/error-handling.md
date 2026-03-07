# Error Handling

---

## Overview

Opal has two error handling mechanisms for different situations: **exceptions** for truly exceptional or unrecoverable errors that propagate implicitly, and **Result types** for expected, recoverable errors that force explicit handling. This two-track model lets each error kind use the mechanism that fits best, avoiding the pitfalls of using only one approach.

---

## 1. When to Use Which

| Situation | Use | Why |
|---|---|---|
| File not found | `Result` | Expected — caller should decide what to do |
| Network timeout | `Result` | Expected in distributed systems |
| Out of memory | Exception | Unrecoverable — can't meaningfully handle |
| Index out of bounds | Exception | Programmer bug |
| Invalid user input | `Result` | Expected — validation is normal flow |
| Database constraint violation | `Result` | Expected — caller chooses retry/report |
| Stack overflow | Exception | Unrecoverable system limit |

**Rule of thumb:** If the caller should *always* handle it, use `Result`. If the caller *shouldn't need to know* how to handle it, use exceptions.

---

## 2. Exceptions

Exceptions are the `fail` / `try` / `catch` / `ensure` mechanism. Errors are classes inheriting from `Error`.

### Raising and Catching

```opal
def read_config(path: String) -> Dict
  if not File.exists?(path)
    fail FileNotFound.new(path: path)
  end
  JSON.parse(File.read(path))
end

try
  config = read_config("missing.json")
catch FileNotFound as e
  print(f"Missing: {e.path}")
catch AuthError as e
  # Catches both PermissionDenied and TokenExpired
  print(f"Auth failed: {e.message}")
catch as e
  log(f"Unexpected: {e.message}")
  fail(e)  # re-raise
ensure
  cleanup()
end
```

### Rules

- `fail expr` raises any `Error` subclass.
- `catch Type as e` catches errors of that type and its subclasses.
- `catch as e` (no type) catches any error.
- `ensure` always executes, whether the block succeeded or failed.
- `Error` provides `.message` and `.stack_trace()` by default.

---

## 3. Result Types & the `!` Operator

`Result[T, E]` is an enum (see [Enums & Algebraic Data Types](../03-functions-and-types/enums-and-algebraic-types.md)) for expected, recoverable errors. The caller must handle both `Ok` and `Err` cases.

```opal
enum Result[T, E]
  Ok(value: T)
  Err(error: E)
end
```

### Basic Usage

```opal
def parse_int(s: String) -> Result[Int32, String]
  # ...
end

match parse_int("42")
  case Result.Ok(n)
    print(f"parsed: {n}")
  case Result.Err(msg)
    print(f"failed: {msg}")
end
```

### The `!` Propagation Operator

The `!` postfix operator unwraps `Ok` or propagates `Err` — the enclosing function must return a `Result`.

```opal
# Without ! — nested matching
def process(path: String) -> Result[Config, Error]
  match read_file(path)
    case Result.Ok(content)
      match parse_json(content)
        case Result.Ok(config)
          Result.Ok(config)
        case Result.Err(e)
          Result.Err(e)
      end
    case Result.Err(e)
      Result.Err(e)
  end
end

# With ! — linear and clean
def process(path: String) -> Result[Config, Error]
  content = read_file(path)!
  config = parse_json(content)!
  Result.Ok(config)
end
```

### Helper Methods on Result

```opal
result.ok?                    # => true if Ok
result.err?                   # => true if Err
result.unwrap()               # => value if Ok, raises exception if Err
result.unwrap("msg")          # => value if Ok, raises with custom message if Err
result.unwrap_or(default)     # => value if Ok, default if Err
result.map(|v| v + 1)        # => Ok(v + 1) if Ok, passes Err through
result.map_err(|e| wrap(e))  # => passes Ok through, transforms Err
```

### Rules

- `expr!` on a `Result` unwraps `Ok` or returns `Err` from the enclosing function.
- The enclosing function must have a `Result` return type — using `!` in a non-Result function is a compile-time error.
- `!` is postfix (after the expression), not prefix.
- `.unwrap()` is different from `!` — it raises an exception instead of returning `Err`.

---

## 4. Bridging Exceptions and Results

Convert between the two error worlds when needed.

### Exceptions to Result

```opal
# Catch any exception into a Result
result = Result.from do
  read_config("missing.json")
end
# => Result.Err(FileNotFound(...)) if it threw
# => Result.Ok(config) if it succeeded

# Catch a specific error type
result = Result.from(FileNotFound) do
  read_config("missing.json")
end
# catches FileNotFound into Err, other errors still propagate
```

### Result to Exception

```opal
# .unwrap() raises the Err value as an exception
config = parse_config(data).unwrap()
# if Err: raises the error as an exception
# if Ok: returns the value

# .unwrap() with custom message
config = parse_config(data).unwrap("config parsing failed")
```

### Mixing the Two Worlds

```opal
# Library returns Result
def parse_config(data: String) -> Result[Config, ValidationError]
  # ...
end

# Your code uses exceptions
def start_app()
  data = File.read("config.json")  # throws FileNotFound
  config = parse_config(data).unwrap()  # converts Err to exception
  App.new(config: config).run()
end

# Or: your code wraps exceptions into Results
def start_app() -> Result[App, Error]
  data = Result.from do
    File.read("config.json")
  end!  # propagate if Err
  config = parse_config(data)!
  Result.Ok(App.new(config: config))
end
```

### Rules

- `Result.from do ... end` catches exceptions into `Result.Err`.
- `Result.from(ErrorType) do ... end` catches only that type (and subclasses).
- `.unwrap()` converts `Err` to an exception. The error value is raised directly.
- `!` and `.unwrap()` are different: `!` propagates as `Result.Err`, `.unwrap()` raises as exception.

---

## 5. Custom Error Types

Errors are classes that inherit from `Error`. Define domain-specific errors by subclassing.

### Base Error (built-in)

```opal
class Error
  needs message: String

  def stack_trace() -> List[String]
    # provided by runtime
  end
end
```

### Defining Custom Errors

```opal
class FileNotFound < Error
  needs path: String

  def init(path)
    .path = path
    super(message: f"File not found: {path}")
  end
end

class NetworkError < Error
  needs url: String
  needs status: Int32

  def init(url, status)
    .url = url
    .status = status
    super(message: f"HTTP {status} from {url}")
  end
end

class ValidationError < Error
  needs field: String
  needs reason: String

  def init(field, reason)
    .field = field
    .reason = reason
    super(message: f"Validation failed on {field}: {reason}")
  end
end
```

### Error Hierarchies

```opal
class AppError < Error end
class AuthError < AppError end
class PermissionDenied < AuthError end
class TokenExpired < AuthError end

# catch AuthError catches both PermissionDenied and TokenExpired
try
  authenticate(token)
catch AuthError as e
  print(f"Auth failed: {e.message}")
end
```

### Raising and Catching Custom Errors

```opal
def read_config(path: String) -> Dict
  if not File.exists?(path)
    fail FileNotFound.new(path: path)
  end
  JSON.parse(File.read(path))
end

try
  config = read_config("missing.json")
catch FileNotFound as e
  print(f"Missing: {e.path}")
catch ValidationError as e
  print(f"Bad field: {e.field} — {e.reason}")
catch Error as e
  print(f"Unexpected: {e.message}")
end
```

### Rules

- `class MyError < Error` defines a custom error type.
- Custom fields via `needs` (like any class).
- `super(message: ...)` chains to the parent Error constructor.
- `Error` provides `.message` and `.stack_trace()` by default.
- `catch Type as e` catches errors of that type **and its subclasses**.

---

## 6. Design Rationale

### Why Two Tracks?

Most languages pick one error mechanism and use it everywhere. This creates friction:

- **Exception-only languages** (Python, Ruby, Java) force you to catch expected errors with try/catch, mixing normal control flow with exceptional cases. Callers don't know which functions can throw, or what they throw.
- **Result-only languages** (Rust, Haskell) force verbose error handling for truly exceptional cases like out-of-memory or assertion failures, where the caller can't meaningfully recover.

Opal provides both mechanisms with clear guidance on when to use each. The two tracks are designed to interoperate cleanly through bridging.

### Why `!` Instead of Automatic Propagation?

The `!` operator makes error propagation explicit at the call site. Every function call that might fail is visibly marked, so you can scan a function body and immediately see all the potential failure points. This is a direct adoption of Rust's `?` operator (using `!` instead to avoid conflict with Opal's `?` predicate convention).

### Why `Result.from` for Bridging?

Rather than inventing special syntax, bridging uses a standard library method that wraps a throwing block in a Result. This keeps the language small — no new keywords for bridging — while making the conversion explicit and composable.

---

## 7. Choosing Your Approach — A Practical Guide

Opal gives you multiple tools for handling errors. Here's how to choose the right one.

### Decision Flowchart

```
Is the error expected in normal operation?
├── No (bug, OOM, assertion) ──────────► fail / try / catch (exceptions)
└── Yes (validation, I/O, parsing) ───► Result[T, E]
    │
    Does the caller need to handle the error?
    ├── Yes, always ──────────────────► return Result, let caller match/propagate
    └── No, just crash if it fails ──► .unwrap() or .unwrap_or(default)
        │
        Are you chaining multiple Result calls?
        ├── Yes ──────────────────────► use ! propagation
        └── No, just one call ────────► match or .unwrap_or()
```

### The Same Problem, Four Ways

**Scenario:** Read a file, parse it as JSON, extract a "name" field.

#### Approach 1: match — Full Control

Use `match` when you need different recovery strategies for different errors, or when the error handling *is* the interesting logic.

```opal
def get_name(path: String) -> String
  match read_file(path)
    case Ok(content)
      match parse_json(content)
        case Ok(data)
          data["name"] ?? "anonymous"
        case Error(e)
          print(f"Bad JSON: {e}")
          "anonymous"
      end
    case Error(e)
      print(f"Can't read {path}: {e}")
      "anonymous"
  end
end
```

**When to use:** You need per-step error recovery. Each failure has a different fallback. You don't want to propagate — you want to handle it right here.

**Downside:** Deep nesting. Hard to follow the happy path.

#### Approach 2: `!` propagation — Clean Chaining

Use `!` when the function should return a `Result` and the caller will decide what to do with errors.

```opal
def get_name(path: String) -> Result[String, Error]
  content = read_file(path)!
  data = parse_json(content)!
  Ok(data["name"] ?? "anonymous")
end

# Caller decides
match get_name("config.json")
  case Ok(name)
    print(f"Hello, {name}")
  case Error(e)
    print(f"Couldn't get name: {e}")
end
```

**When to use:** You're writing a library or intermediate function. You don't know how the caller wants to handle the error. Multiple fallible steps in sequence. The function's job is the happy path, not error recovery.

**Downside:** Caller must handle the Result. If every caller just unwraps, the propagation adds ceremony without value.

#### Approach 3: .unwrap_or() — Quick Fallback

Use `unwrap_or` when a default value is acceptable and you don't care *why* it failed.

```opal
def get_name(path: String) -> String
  content = read_file(path).unwrap_or("{}")
  data = parse_json(content).unwrap_or({:})
  data["name"] ?? "anonymous"
end
```

**When to use:** Errors are expected and non-critical. A sensible default exists. You don't need to log or report the failure.

**Downside:** Silently swallows errors. If something goes wrong, you won't know.

#### Approach 4: try / catch — Exception Boundary

Use `try`/`catch` when operations `fail` with exceptions (bugs, system errors, or libraries that throw).

```opal
def get_name(path: String) -> String
  try
    content = File.read(path)
    data = JSON.parse(content)
    data["name"] ?? "anonymous"
  catch FileNotFound as e
    "anonymous"
  catch as e
    log(f"Unexpected error: {e.message}")
    "anonymous"
  end
end
```

**When to use:** The operations throw exceptions (not Result). You want a catch-all safety net. You're at a system boundary (HTTP handler, CLI entry point, event loop).

**Downside:** Hard to know which line threw. Easy to catch too broadly. Doesn't force the caller to think about failure.

### Quick Reference: When to Use What

| Approach | Returns | Best for | Error visibility |
|----------|---------|----------|------------------|
| `match` on Result | Any type | Per-error recovery | Explicit, verbose |
| `!` propagation | `Result[T, E]` | Chaining, libraries | Explicit, concise |
| `.unwrap_or(val)` | Inner type | Silent defaults | Hidden |
| `.unwrap()` | Inner type | "Must succeed" cases | Crashes on error |
| `try / catch` | Any type | Exception boundaries | Catch-all |
| `ok?()` / `err?()` | Bool | Conditional checks | Query only |

### Combining Approaches

Real code often mixes approaches at different layers:

```opal
# Low-level: returns Result (let caller decide)
def fetch_user(id: Int32) -> Result[User, Error]
  response = http_get(f"/users/{id}")!
  parse_user(response.body)!
end

# Mid-level: propagates with !
def fetch_user_profile(id: Int32) -> Result[Profile, Error]
  user = fetch_user(id)!
  settings = fetch_settings(user.id).unwrap_or(Settings.default())
  Ok(Profile.new(user: user, settings: settings))
end

# Top-level: handles everything
def show_profile(id: Int32)
  match fetch_user_profile(id)
    case Ok(profile)
      render(profile)
    case Error(e)
      render_error(f"Couldn't load profile: {e}")
  end
end
```

**Pattern:** Propagate `!` at lower layers, `match` at the top, `unwrap_or` for non-critical branches.

### Anti-Patterns

| Don't do this | Do this instead | Why |
|---------------|-----------------|-----|
| `result.unwrap()` everywhere | `result!` or `match` | `unwrap` crashes; `!` propagates gracefully |
| Nested `match` 3+ levels deep | Chain with `!` | Flat code is easier to read |
| `try/catch` around Result operations | `match` or `!` | Result errors aren't exceptions |
| Ignoring Result (discard return) | `match`, `!`, or `unwrap_or` | Silent failure hides bugs |
| `!` in a top-level script | `match` or `unwrap_or` | `!` at top level has nowhere to propagate |

---

## Summary

| Feature | Purpose |
|---|---|
| `fail` / `try` / `catch` | Exceptions — unrecoverable/unexpected errors |
| `ensure` | Always-run cleanup block |
| `Result[T, E]` | Explicit return values — expected/recoverable errors |
| `!` operator | Postfix on Result — unwraps Ok or propagates Err |
| `.unwrap()` | Converts Result.Err to an exception |
| `Result.from do ... end` | Catches exceptions into Result.Err |
| `.ok?`, `.err?` | Query Result state |
| `.unwrap_or(default)` | Unwrap with fallback |
| `.map()`, `.map_err()` | Transform Ok or Err value |
| `class MyError < Error` | Custom error types with domain-specific fields |
| Error hierarchies | `catch` matches type and all subclasses |

### New Keywords

| Keyword | Role |
|---|---|
| `fail` | Raise an exception |
| `try` | Begin a block that may throw |
| `catch` | Handle a thrown exception by type |
| `ensure` | Always-run block after try/catch |
| `!` | Postfix operator — unwrap Ok or propagate Err |
