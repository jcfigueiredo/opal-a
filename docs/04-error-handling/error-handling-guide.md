# Error Handling in Opal — A Practical Guide

---

## The Core Idea

Functions return values. If something goes wrong, they return `Error(reason)`. That's it.

```opal
def find_user(id: Int32)
  user = db.get("users", id)
  if user == null
    Error("user {id} not found")
  end
  user
end
```

When you call a function that returns `Error`, it **throws automatically**. You get the value or you get an exception — no wrapping, no unwrapping.

```opal
user = find_user(42)    # User, or throws
```

### Everything thrown is an Error object

Strings, custom classes, and any other value get wrapped into a built-in `Error` object:

```opal
Error("not found")
# => Error(message: "not found")

Error(NotFoundError(resource: "User", id: 42))
# => Error(cause: NotFoundError(...), message: "User 42 not found")

raise "something broke"
# => throws Error(message: "something broke")

raise NotFoundError(resource: "User", id: 42)
# => throws Error(cause: NotFoundError(...), message: "User 42 not found")
```

The built-in `Error` class:

```opal
class Error
  needs message: String = ""
  needs cause: Any = null

  def to_s()
    message
  end
end
```

When you pass a string, it becomes the `message`. When you pass a class instance, it becomes the `cause` and `message` is populated from `.to_s()`.

---

## The `?` Operator

`?` suppresses the throw. You get back whatever the function returned — the value or the `Error`.

```opal
result = find_user(42)?    # User or Error("not found")
```

`Error` is falsy. So `?` works naturally with `or`, `if`, and `match`:

```opal
# Default value
user = find_user(42)? or User.guest()

# Boolean check
if find_user(42)?
  print("found")
end

# Match for details
match find_user(42)?
  case Error(msg)
    print("not found: {msg}")
  case user
    print(user.name)
end
```

That's the whole surface area:

| Syntax | What you get | On Error |
|---|---|---|
| `f()` | Value | Throws |
| `f()?` | Value or Error | You handle it |
| `f()? or default` | Value or default | Uses default |
| `if f()?` | Truthy/falsy check | Error is falsy |

---

## Panics

Panics are programmer bugs. They crash. You can't catch them.

```opal
42 + "hello"          # Panic: TypeError
print(undefined_var)  # Panic: NameError
list[999]             # Panic: index out of bounds
```

**The rule:** If you could have prevented it by writing better code, it's a panic. Everything else is an `Error`.

Supervisors restart crashed processes in production.

---

## try/catch

Wrap code in `try/catch` to handle thrown errors:

```opal
try
  user = find_user(id)
  settings = fetch_settings(user.id)
  Profile.new(user: user, settings: settings)
catch e
  log("Failed: {e}")
  Profile.guest()
end
```

This reads like normal code. If any call returns `Error`, it throws, and `catch` handles it.

### catch e

`catch e` (no cases) gives you the `Error` wrapper:

```opal
catch e
  print(e.message)    # the error message
  print(e.cause)      # the inner value (NotFoundError, String, etc.)
end
```

### Catching by type

Use `case` blocks to match on the **cause** type inside the Error:

```opal
try
  user = find_user(42)
  withdraw(user, 500.0)
catch
  case NotFoundError as e         # e is the NotFoundError (the cause)
    print(e.resource)
  case InsufficientFunds as e     # e is the InsufficientFunds
    print(e.amount)
  case String as e                # e is the raw string
    print(e)
  case _ as e                     # e is the Error wrapper (catch-all)
    print(e.message)
end
```

Cases match on the cause type and bind the cause directly. The catch-all `case _` gives you the Error wrapper itself. Cases are checked top-to-bottom — put specific types first.

---

## The `raise` Keyword

`raise` throws directly. Strings and class instances are wrapped into `Error` automatically:

```opal
def withdraw(account, amount)
  if amount > account.balance
    raise InsufficientFunds(amount: amount, balance: account.balance)
  end
  account.debit(amount)
end
```

`raise` and returning `Error(...)` do the same thing — they throw. `raise` is for explicit "stop here" failures. `Error(...)` is for functions that signal failure as part of their return.

---

## Custom Error Types

Error types are just regular classes. No special base class, no inheritance required:

```opal
class NotFoundError
  needs resource: String
  needs id: Int32

  def to_s()
    f"{resource} {id} not found"
  end
end

class InsufficientFunds
  needs amount: Float64
  needs balance: Float64

  def to_s()
    f"Need {amount}, have {balance}"
  end
end
```

Use them with `Error(...)` or `raise`:

```opal
def find_user(id: Int32)
  user = db.get("users", id)
  if user == null
    Error(NotFoundError(resource: "User", id: id))
  end
  user
end

def withdraw(account, amount)
  if amount > account.balance
    raise InsufficientFunds(amount: amount, balance: account.balance)
  end
  account.debit(amount)
end
```

---

## Choosing the Right Approach

```
Can the caller recover from this?
├── No (bug, wrong type) ────────► Don't handle it. It's a panic.
└── Yes (expected failure)
    │
    Is failure acceptable here?
    ├── Yes, use a default ──────► f()? or default
    ├── Yes, need error details ─► match f()? case Error...
    └── No, it must succeed
        │
        Want a safety net?
        ├── Yes ─────────────────► try/catch
        └── No ──────────────────► Just call it
```

### Same Problem, Four Ways

**Scenario:** Load user, fetch settings, build profile.

**Just call it** — throws on error, caller deals with it:
```opal
def build_profile(id: Int32)
  user = find_user(id)
  settings = fetch_settings(user.id)
  Profile.new(user: user, settings: settings)
end
```

**try/catch** — safety net:
```opal
def build_profile(id: Int32)
  try
    user = find_user(id)
    settings = fetch_settings(user.id)
    Profile.new(user: user, settings: settings)
  catch e
    log("Profile build failed: {e}")
    Profile.guest()
  end
end
```

**? or** — silent defaults:
```opal
def build_profile(id: Int32)
  user = find_user(id)? or User.guest()
  settings = fetch_settings(user.id)? or Settings.default()
  Profile.new(user: user, settings: settings)
end
```

**Mixed** — critical + optional:
```opal
def build_profile(id: Int32)
  try
    user = find_user(id)                                       # must succeed
    settings = fetch_settings(user.id)? or Settings.default()  # optional
    avatar = fetch_avatar(user.id)? or Avatar.default()        # optional
    Profile.new(user: user, settings: settings, avatar: avatar)
  catch e
    log("Can't build profile: {e}")
    Profile.guest()
  end
end
```

---

## Layering

Different strategies at different layers:

```opal
# Bottom: signals failure with Error
def find_user(id: Int32)
  row = db.query("SELECT * FROM users WHERE id = ?", id)
  if row == null
    Error("user {id} not found")
  end
  User.from_row(row)
end

# Middle: calls throw through, adds own errors
def transfer(from_id: Int32, to_id: Int32, amount: Float64)
  sender = find_user(from_id)
  receiver = find_user(to_id)
  if sender.balance < amount
    Error("insufficient funds")
  end
  sender.debit(amount)
  receiver.credit(amount)
  Receipt.new(from: sender, to: receiver, amount: amount)
end

# Top: catches everything
def handle_transfer(request: Request)
  try
    from_id = Int32.parse(request.params["from"])
    to_id = Int32.parse(request.params["to"])
    amount = Float64.parse(request.params["amount"])
    receipt = transfer(from_id, to_id, amount)
    Response.json(receipt, status: 200)
  catch
    case NotFoundError as e
      Response.json({error: f"{e}"}, status: 404)
    case InsufficientFunds as e
      Response.json({error: f"{e}"}, status: 422)
    case _ as e
      Response.json({error: e.message}, status: 400)
  end
end
```

---

## Anti-Patterns

| Don't | Do | Why |
|---|---|---|
| Nested `match` 3+ deep | Just call + try/catch | Flat code reads better |
| `? or default` when failure matters | Call normally + try/catch | Don't silently swallow important errors |
| Catch-all at every layer | Catch at boundaries | Let errors reach where they can be handled |
| Match on error strings | Custom error types | `case NotFoundError` beats `if e.message.contains?("not found")` |
| Ignore returned errors | Call normally or use `?` | Silent failure hides bugs |

---

## Summary

1. **Functions return values or `Error`.** That's the only primitive.
2. **Everything thrown is an `Error` object.** Strings and classes are wrapped automatically.
3. **Errors throw by default.** Call a function, get the value.
4. **`?` suppresses the throw.** Get the value or the Error, your choice.
5. **`? or default` for fallbacks.** Error is falsy, `or` provides the default.
6. **`catch case` matches on cause type.** Match on what's inside the Error.
7. **Panics crash.** Wrong types, undefined names — fix the bug, don't catch it.
8. **Catch at boundaries.** Bottom layers return Errors, top layers catch.
