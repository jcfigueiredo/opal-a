# Destructuring Assignment

---

## Overview

Opal extends pattern matching syntax to regular assignment, function parameters, `for` loops, and closures. The same patterns used in `match` expressions work everywhere, giving one consistent way to unpack structured data.

---

## 1. Tuple Destructuring

```opal
(x, y) = get_point()
(status, body) = http_get("/users")

# Ignore with _
(_, y) = get_point()

# Nested
(first, (a, b)) = (1, (2, 3))
# first = 1, a = 2, b = 3
```

## 2. Dict Destructuring

```opal
{name: n, age: a} = {name: "claudio", age: 15, role: "admin"}
# n = "claudio", a = 15 (extra keys ignored)

# Optional keys with ?
{name: n, age?: a} = {name: "claudio"}
# n = "claudio", a = null
```

## 3. List Destructuring (head/tail)

```opal
[first, second | rest] = [1, 2, 3, 4, 5]
# first = 1, second = 2, rest = [3, 4, 5]

[head | _] = [10, 20, 30]
# head = 10
```

## 4. In Function Parameters

```opal
def distance((x1, y1), (x2, y2))
  ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
end

distance((0, 0), (3, 4))  # => 5.0
```

## 5. In For Loops

```opal
pairs = [("alice", 30), ("bob", 25)]
for (name, age) in pairs
  print(f"{name} is {age}")
end
```

## 6. In Closures

```opal
points.map(|(x, y)| x + y)
```

## 7. Design Rationale

Destructuring is one of five self-hosting foundations that enable Opal's standard library and ecosystem to be written in Opal itself. It allows domain models, parsers, and data-processing pipelines to unpack structured data naturally without verbose accessor calls.

### Why One Pattern Syntax Everywhere

Opal uses the same destructuring patterns in assignment `=`, function params, `for` loops, closures, and `match` expressions. This "one way to do it" design means learning destructuring once applies everywhere, reducing cognitive overhead.

### What Destructuring Unlocks

| What You Can Now Write in Opal | Features Used |
|---|---|
| `JSON`, `YAML` parsers | Destructuring + pattern matching |
| Domain models | Destructuring + custom errors + DI |

### Rules

- Destructuring works in assignment `=`, function params, `for` loops, and closures.
- `_` ignores a value.
- `[head | tail]` splits a list into first element(s) and rest.
- Dict destructuring extracts by key; extra keys are ignored.
- Missing required keys = runtime error. Use `?` suffix for optional keys.
- Same pattern syntax as `match` -- one way to do it everywhere.

## Summary

| Pattern | Syntax | Example |
|---|---|---|
| Tuple | `(a, b) = expr` | `(x, y) = get_point()` |
| Nested tuple | `(a, (b, c)) = expr` | `(first, (a, b)) = (1, (2, 3))` |
| Dict | `{key: var} = expr` | `{name: n, age: a} = person` |
| Optional dict key | `{key?: var} = expr` | `{name: n, age?: a} = data` |
| List head/tail | `[h \| t] = expr` | `[first \| rest] = list` |
| Ignore | `_` | `(_, y) = get_point()` |
| Function param | `def f((a, b))` | `def distance((x1, y1), (x2, y2))` |
| For loop | `for (a, b) in expr` | `for (name, age) in pairs` |
| Closure | `\|(a, b)\| expr` | `points.map(\|(x, y)\| x + y)` |
