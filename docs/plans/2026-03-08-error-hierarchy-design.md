# Error Hierarchy Redesign — Design Document

**Date:** 2026-03-08
**Status:** Approved

---

## Problem

Opal's error handling has inconsistencies:
- `EvalError` has 9 variants with unclear catchability rules
- `TypeError`, `RuntimeError`, `UndefinedVariable` are strings, not structured objects
- The `!` propagation operator requires developers to remember to add it — forgetting leads to confusing TypeErrors when using an unwrapped Result
- `RequiresFailed` is a separate variant from `Raise` despite both being catchable
- No unified error object — thrown values have no guaranteed `.message` or `.stack_trace()`

## Design

### Two tracks: Panics and Errors

**Panics** (uncatchable, crash the process):
- TypeError — wrong type for operation
- NameError — undefined variable/method
- RuntimeError — division by zero, index out of bounds

**Errors** (catchable via `try/catch`):
- Everything thrown by `fail`, `Error(...)` returns, or `requires` failures

### The Error wrapper

All thrown values are wrapped in a built-in `Error` object:

```opal
class Error
  needs message: String = ""
  needs cause: Any = null

  def stack_trace() -> List[String]
    # populated by runtime at throw time
  end

  def to_s()
    message
  end
end
```

- `Error("string")` → `Error(message: "string")`
- `Error(MyClass(...))` → `Error(cause: MyClass(...), message: cause.to_s())`
- `fail "string"` → throws `Error(message: "string")`
- `fail MyClass(...)` → throws `Error(cause: MyClass(...), message: cause.to_s())`

### Auto-throw on Error return

When a function returns an `Error(...)`, the caller receives a throw automatically. No `!` operator needed.

```opal
def find_user(id)
  if id < 0
    Error("invalid id")    # throws when caller receives it
  end
  db.get("users", id)
end

user = find_user(42)    # User directly, or throws
```

### The `?` operator

`?` suppresses auto-throw. The caller gets the raw return value — either the value or the Error.

```opal
result = find_user(42)?         # User or Error("not found")

user = find_user(42)? or User.guest()   # Error is falsy, or provides default

if find_user(42)?                        # truthy/falsy check
  print("found")
end

match find_user(42)?
  case Error(msg)
    print(msg)
  case user
    print(user.name)
end
```

### `?` vs `?.` disambiguation

- `f()?` followed by whitespace/newline/end → Result operator
- `f()?.method()` → optional chaining (existing behavior, `?.` is a single token)

### Error truthiness

- `Error(...)` is falsy (alongside `false` and `null`)
- All other values remain truthy
- This is specific to the built-in `Error` type, not a general enum change

### catch syntax

Two forms:

**Simple catch-all** — `e` is the Error wrapper:
```opal
catch as e
  print(e.message)
  print(e.cause)
  print(e.stack_trace())
end
```

**Typed cases** — matches on `cause` type, binds the cause:
```opal
catch
  case NotFoundError as e     # e is the NotFoundError (cause)
    print(e.resource)
  case String as e            # e is the raw string
    print(e)
  case _ as e                 # e is the Error wrapper (catch-all)
    print(e.message)
end
```

### Custom error types

Regular classes, no special base class required:

```opal
class NotFoundError
  needs resource: String
  needs id: Int32

  def to_s()
    f"{resource} {id} not found"
  end
end
```

### Rust implementation

**EvalError enum (after):**
```rust
pub enum EvalError {
    // Panics (uncatchable)
    Panic(PanicKind, String),

    // Errors (catchable by try/catch)
    Raise(Value),        // Value is always an Error instance

    // Control flow
    Return(Value),
    Break,
    Next,
    Reply(Value),
}

pub enum PanicKind {
    TypeError,
    NameError,
    RuntimeError,
}
```

**Changes from current enum:**
- `TypeError(String)` → `Panic(PanicKind::TypeError, msg)`
- `RuntimeError(String)` → `Panic(PanicKind::RuntimeError, msg)`
- `UndefinedVariable(String)` → `Panic(PanicKind::NameError, msg)`
- `RequiresFailed(Value)` → `Raise(Value)` (wraps in Error)
- `Raise(Value)` → `Raise(Value)` (wraps in Error if not already)

### What's removed

- **`!` propagation operator** — replaced by auto-throw on Error return
- **`Ok(...)` wrapper** — functions just return values
- **`Result[T, E]` return type** — functions return values or Error
- **`ok?()`, `err?()`, `unwrap()`, `unwrap_or()`** — replaced by `?`, `or`, `is Error`
- **`ExprKind::Propagate`** — removed from AST
- **`propagation_expression`** — removed from tree-sitter grammar

### What's added

- **`PanicKind` enum** in Rust
- **`Error` built-in class** with `message`, `cause`, `stack_trace()`
- **`?` postfix operator** — suppresses auto-throw, returns value or Error
- **Auto-throw logic** — in function call evaluation, check if return value is Error
- **Error falsy** — Error joins `false` and `null` as falsy values
- **`catch` + `case` blocks** — type matching on cause
- **`fail` wrapping** — strings and classes auto-wrapped in Error

### What changes

- **~180 error sites** in interpreter — `TypeError(msg)` → `Panic(PanicKind::TypeError, msg)` etc.
- **`try/catch` handler** — catches `Raise` only, type-matches on cause
- **`fail` evaluation** — wraps value in Error before raising
- **`requires` failure** — creates Error with cause, uses `Raise`
- **Tree-sitter grammar** — remove `propagation_expression`/`bang`, keep `?` handling
- **Specs** — update all Result/propagation specs

### BEAM compatibility

- `Error(reason)` → `{:error, reason}` tuples
- `fail` → Erlang `throw`
- Panics → Erlang `error` with re-raise guard at catch sites
- Stack traces → `erlang:get_stacktrace()` populates Error wrapper
- Error wrapper → Erlang map `#{message => ..., cause => ..., stack_trace => ...}`
- Future: reserve `exit` mechanism for actor process termination

---

## Impact

| Area | Scope |
|---|---|
| `crates/opal-interp/src/eval.rs` | ~180 error sites, auto-throw logic, `fail` wrapping, `catch` rewrite |
| `crates/opal-parser/src/ast.rs` | Remove `Propagate`, add `SuppressThrow` for `?` |
| `crates/opal-parser/src/parser.rs` | Remove `!` postfix, add `?` postfix, remove `Ok()` built-in |
| `crates/opal-lexer/src/token.rs` | Keep `Bang` for mutation calls, add `Question` if not present |
| `crates/opal-interp/src/eval.rs` | Register `Error` class at startup |
| `crates/opal-lsp/src/goto_def.rs` | No changes needed |
| `tree-sitter-opal/grammar.js` | Remove `propagation_expression`/`bang`, add `suppress_throw_expression` |
| `tests/spec/06-errors/` | Rewrite propagation specs, add `?`/auto-throw specs |
| `docs/04-error-handling/` | Already updated (error-handling-guide.md), update error-handling.md |

## Reference

- [Error Handling Guide](../04-error-handling/error-handling-guide.md) — practical guide with examples
- [Error Handling Spec](../04-error-handling/error-handling.md) — language specification (needs update)
