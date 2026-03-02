# Error Handling

---

## Overview

Opal has two error handling mechanisms for different situations:

- **Exceptions** (`fail` / `try` / `catch`) — for truly exceptional, unrecoverable, or unexpected errors. These propagate implicitly up the call stack.
- **Result types** (`Result(T, E)`) — for expected, recoverable errors. These are explicit return values that force the caller to handle both cases.

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

Exceptions are the existing `fail` / `try` / `catch` / `ensure` mechanism. Errors are classes inheriting from `Error`.

### Custom Error Types

```opal
class Error
  needs message::String

  def stack_trace() -> List(String)
    # provided by runtime
  end
end

class FileNotFound < Error
  needs path::String

  def init(path)
    .path = path
    super(message: f"File not found: {path}")
  end
end
```

### Raising and Catching

```opal
def read_config(path::String) -> Dict
  if not File.exists?(path)
    fail FileNotFound.new(path: path)
  end
  JSON.parse(File.read(path))
end

try
  config = read_config("missing.json")
catch FileNotFound as e
  print(f"Missing: {e.path}")
catch as e
  log(f"Unexpected: {e.message}")
  fail(e)  # re-raise
ensure
  cleanup()
end
```

### Error Hierarchies

```opal
class AppError < Error end
class AuthError < AppError end
class PermissionDenied < AuthError end
class TokenExpired < AuthError end

# Catches both PermissionDenied and TokenExpired
try
  authenticate(token)
catch AuthError as e
  print(f"Auth failed: {e.message}")
end
```

### Rules

- `fail expr` raises any `Error` subclass.
- `catch Type as e` catches errors of that type and its subclasses.
- `catch as e` (no type) catches any error.
- `ensure` always executes, whether the block succeeded or failed.
- `Error` provides `.message` and `.stack_trace()` by default.

---

## 3. Result Types

`Result(T, E)` is an enum for expected, recoverable errors. The caller must handle both `Ok` and `Err` cases.

```opal
enum Result(T, E)
  Ok(value::T)
  Err(error::E)
end
```

### Basic Usage

```opal
def parse_int(s::String) -> Result(Int32, String)
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
def process(path::String) -> Result(Config, Error)
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
def process(path::String) -> Result(Config, Error)
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
def parse_config(data::String) -> Result(Config, ValidationError)
  # ...
end

# Your code uses exceptions
def start_app()
  data = File.read("config.json")  # throws FileNotFound
  config = parse_config(data).unwrap()  # converts Err to exception
  App.new(config: config).run()
end

# Or: your code wraps exceptions into Results
def start_app() -> Result(App, Error)
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

## Summary

| Feature | Purpose |
|---|---|
| `fail` / `try` / `catch` | Exceptions — unrecoverable/unexpected errors |
| `Result(T, E)` | Explicit return values — expected/recoverable errors |
| `!` operator | Postfix on Result — unwraps Ok or propagates Err |
| `.unwrap()` | Converts Result.Err to an exception |
| `Result.from do ... end` | Catches exceptions into Result.Err |
| `.ok?`, `.err?` | Query Result state |
| `.unwrap_or(default)` | Unwrap with fallback |
| `.map()`, `.map_err()` | Transform Ok or Err value |
