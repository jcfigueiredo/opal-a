# Error Handling

---

## Overview

Opal uses a unified error model with two severity levels: **errors** (recoverable, catchable) and **panics** (unrecoverable, uncatchable). Functions that return `Error(...)` automatically throw — the caller doesn't need to remember to check return values. The `?` operator suppresses auto-throw when you want to handle errors as values.

---

## 1. When to Use Which

| Situation | Mechanism | Why |
|---|---|---|
| File not found | `Error(...)` | Expected — caller can handle with `?` or `catch` |
| Network timeout | `Error(...)` | Expected in distributed systems |
| Invalid user input | `Error(...)` | Expected — validation is normal flow |
| Database constraint violation | `Error(...)` | Expected — caller chooses retry/report |
| Type mismatch at runtime | Panic | Programmer bug — uncatchable |
| Index out of bounds | Panic | Programmer bug — uncatchable |
| Out of memory | Panic | Unrecoverable system limit |

**Rule of thumb:** If it's a programmer bug or truly unrecoverable, it's a panic (crashes). Everything else is an `Error` (catchable).

---

## 2. The Error Model

### Error Class

`Error` is a built-in class with two fields:

```opal
# Built-in (registered at interpreter startup)
class Error
  needs message: String
  needs cause          # the original value passed to Error()
end
```

Create errors with the `Error()` constructor:

```opal
e = Error("something went wrong")
e.message   # => "something went wrong"
e.cause     # => null (string errors have no separate cause)

class NotFoundError
  needs id
end

e = Error(NotFoundError.new(id: 42))
e.message   # => string representation of the cause
e.cause     # => the NotFoundError instance
e.cause.id  # => 42
```

### Error Is Falsy

Error instances are falsy (like `false` and `null`), enabling concise patterns:

```opal
result = find_user(999)? or "default_user"
```

---

## 3. Auto-Throw

When a function, closure, or method returns an `Error(...)` value, the runtime **automatically throws** it. The caller doesn't need to check — errors propagate by default.

```opal
def find_user(id)
  if id > 100
    return Error("user not found")
  end
  f"user_{id}"
end

# Auto-throw: Error return throws, caught by try/catch
try
  find_user(999)
catch e
  print(f"caught: {e.message}")   # => caught: user not found
end

# Success: returns value directly
user = find_user(1)   # => "user_1"
```

### What Auto-Throws

Auto-throw applies to:
- User-defined functions (`def`)
- Closures (`|params| expr` and `do |params| ... end`)
- Instance methods
- Static methods

Auto-throw does **not** apply to the `Error()` constructor itself — it returns the Error value for assignment.

---

## 4. The `?` Operator — Suppress Auto-Throw

The `?` postfix operator catches the auto-throw and returns the Error as a value instead of propagating it. This lets you handle errors as data.

```opal
def divide(a, b)
  if b == 0
    return Error("division by zero")
  end
  a / b
end

# Without ? — auto-throw propagates
try
  divide(10, 0)       # throws!
catch e
  print(e.message)
end

# With ? — suppress throw, get the Error value
result = divide(10, 0)?
# result is now the Error instance (falsy)

# ? or default — concise fallback
safe = divide(10, 0)? or 0    # => 0

# ? for conditional check
if divide(10, 2)?
  print("success")    # prints this (non-Error is truthy)
end
```

### Pattern: Match on `?` Result

```opal
result = some_operation()?

r = match result
  case Error(msg)
    f"failed: {msg}"
  case val
    f"got: {val}"
end
```

---

## 5. Panics — Uncatchable Errors

Panics are runtime errors that indicate bugs or unrecoverable conditions. They **cannot** be caught by `try/catch` and always crash the program.

```opal
# These panic — they cannot be caught
x = 42 + "hello"     # TypeError panic
y = undefined_var     # NameError panic

# try/catch does NOT catch panics
try
  x = 42 + "hello"
catch e
  print("never reached")
end
# => TypeError: unsupported operation Add on Integer(42) and String("hello")
```

Panics are for programmer errors. Don't try to recover from them — fix the code instead.

---

## 6. try / catch / ensure

### Basic Usage

```opal
try
  result = risky_operation()
catch e
  print(f"Error: {e.message}")
ensure
  cleanup()
end
```

### Typed Catch — Match on Cause Type

`catch e as Type` matches the `.cause` field of the Error wrapper and binds the cause (not the wrapper) to the variable:

```opal
class NotFoundError
  needs id
end

def find_user(id)
  if id > 100
    return Error(NotFoundError.new(id: id))
  end
  f"user_{id}"
end

try
  find_user(999)
catch e as NotFoundError
  # e is the NotFoundError instance (the cause)
  print(f"not found: {e.id}")
catch e
  # e is the full Error wrapper
  print(f"other: {e.message}")
end
```

### Rules

- `raise expr` throws a value (wrapped in Error if not already).
- `catch e` (no type) catches any error, binds the Error wrapper.
- `catch e as Type` matches on the cause type and binds the cause directly.
- `ensure` always executes, whether the block succeeded or failed.
- Panics are **not** caught by `try/catch`.

---

## 7. Custom Error Types

Custom errors are regular classes. No base class inheritance required — wrap them with `Error()` to throw.

```opal
class NotFoundError
  needs path: String
end

class ValidationError
  needs field: String
  needs reason: String
end

# Throw custom error
def read_config(path)
  if not File.exists?(path)
    return Error(NotFoundError.new(path: path))
  end
  JSON.parse(File.read(path))
end

# Catch by cause type
try
  read_config("missing.json")
catch e as NotFoundError
  print(f"Missing: {e.path}")
catch e as ValidationError
  print(f"Bad field: {e.field} — {e.reason}")
catch e
  print(f"Unexpected: {e.message}")
end
```

---

## 8. Choosing Your Approach — A Practical Guide

### Decision Flowchart

```
Can the caller recover from this error?
├── No (bug, OOM, assertion) ─────────► Panic (crash — don't handle)
└── Yes ──────────────────────────────► Error(...)
    │
    Does the caller need the error value?
    ├── Yes ──────────────────────────► f()? then match/check
    └── No, just crash or fallback ──► let auto-throw propagate, or f()? or default
```

### The Same Problem, Three Ways

**Scenario:** Parse an integer from user input, with fallback.

#### Approach 1: Let Auto-Throw Propagate

```opal
def parse_config(input)
  value = parse_int(input)  # throws if Error
  value * 2
end

# Caller catches
try
  parse_config("abc")
catch e
  print(f"bad input: {e.message}")
end
```

**When to use:** You want errors to bubble up. The caller (or a top-level handler) will deal with them.

#### Approach 2: `?` or Default

```opal
def parse_config(input)
  value = parse_int(input)? or 0
  value * 2
end
```

**When to use:** A sensible default exists and you don't care why it failed.

#### Approach 3: `?` with Match

```opal
def parse_config(input)
  result = parse_int(input)?
  match result
    case Error(msg)
      print(f"warning: {msg}, using default")
      0
    case val
      val * 2
  end
end
```

**When to use:** You want per-error recovery with visibility into what went wrong.

### Quick Reference

| Pattern | Effect | Best for |
|---------|--------|----------|
| `f()` | Auto-throws if Error | Default — let errors propagate |
| `f()?` | Suppresses throw, returns value or Error | Inspecting or matching on errors |
| `f()? or default` | Fallback value | Silent defaults |
| `try/catch` | Catch thrown errors | Error boundaries, cleanup |
| `catch e as Type` | Match on cause type | Typed error dispatch |

---

## Summary

| Feature | Purpose |
|---|---|
| `Error(val)` | Create an Error with `.message` and `.cause` |
| Auto-throw | Functions returning Error automatically throw |
| `?` operator | Suppress auto-throw — return Error as value |
| `? or default` | Fallback when Error (Error is falsy) |
| `raise expr` | Throw explicitly (wraps in Error) |
| `try / catch / ensure` | Catch thrown errors, run cleanup |
| `catch e as Type` | Match on cause type, bind cause |
| Panics | Uncatchable runtime errors (bugs) |

### Keywords

| Keyword / Operator | Role |
|---|---|
| `raise` | Throw a value (wrapped in Error) |
| `try` | Begin a block that may throw |
| `catch` | Handle a thrown error |
| `catch e as Type` | Handle a specific cause type |
| `ensure` | Always-run block after try/catch |
| `?` | Postfix — suppress auto-throw |
