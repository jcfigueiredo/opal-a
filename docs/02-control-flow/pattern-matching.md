# Pattern Matching

---

## Overview

Opal's `match` expression supports a rich set of pattern forms. Patterns are tried top-to-bottom; the first matching case wins. Destructuring in patterns mirrors destructuring assignment but within a match context.

---

## 1. Literals & Ranges

```opal
match value
  case 0
    "zero"
  case 1..10
    "small"
  case "hello"
    "greeting"
  case :ok
    "symbol"
  case null
    "nothing"
  case true
    "yes"
  case _
    "other"
end
```

## 2. Variable Binding & Type Matching

```opal
match response
  case s::String
    print(s)
  case n::Int32
    print(f"code: {n}")
  case (status, body)
    print(f"{status}: {body}")
end
```

## 3. Or-Patterns

Use `|` to match any of several alternatives in a single case arm. Or-patterns cannot bind variables (it would be ambiguous which alternative produced the binding).

```opal
match status_code
  case 200 | 201 | 204
    "success"
  case 301 | 302
    "redirect"
  case 400 | 404 | 422
    "client error"
  case _
    "other"
end

# Works with enums
match direction
  case Direction.North | Direction.South
    "vertical"
  case Direction.East | Direction.West
    "horizontal"
end
```

## 4. Tuple & Dict Destructuring

Tuple patterns work like destructuring assignment inside match arms. Dict patterns extract values by key.

```opal
# Tuples
match point
  case (0, 0)
    "origin"
  case (x, 0)
    f"on x-axis at {x}"
  case (0, y)
    f"on y-axis at {y}"
  case (x, y)
    f"at ({x}, {y})"
end

# Dicts
match config
  case {host: h, port: p}
    connect(h, p)
  case {host: h}
    connect(h, 8080)
end
```

## 5. List Patterns

Lists can be matched by exact shape or split into head and tail with `|`.

```opal
match items
  case []
    "empty"
  case [only]
    f"just {only}"
  case [first, second]
    f"{first} and {second}"
  case [head | tail]
    f"{head} and {tail.length} more"
end
```

## 6. Enum Patterns & Nesting

Enum variant patterns extract associated data. Patterns nest arbitrarily. See [Enums & Algebraic Data Types](../03-functions-and-types/enums-and-algebraic-types.md) for enum definitions.

```opal
# Enum variant matching
match shape
  case Shape.Circle(r)
    Math.PI * r ** 2
  case Shape.Rectangle(w, h)
    w * h
  case Shape.Triangle(b, h)
    0.5 * b * h
end

# Nested patterns — arbitrarily deep
match result
  case Result.Ok(Option.Some(value))
    use(value)
  case Result.Ok(Option.None)
    use_default()
  case Result.Err(e)
    handle(e)
end
```

## 7. Guards

A `case` arm can include an `if` guard -- the arm only matches when both the pattern and the condition are true.

```opal
match value
  case x if x > 100
    "large"
  case x if x > 0
    "positive"
  case x
    "non-positive"
end
```

## 8. As-Bindings

`as name` binds the entire matched value while still destructuring its contents.

```opal
match shape
  case Shape.Circle(r) as original
    log(original)          # the whole Circle value
    Math.PI * r ** 2
  case _ as s
    log(f"unknown: {s}")
    0.0
end
```

## 9. Pattern Summary

| Pattern | Example | Matches |
|---|---|---|
| Literal | `case 42`, `case "hello"`, `case null` | Exact value |
| Range | `case 1..10` | Value in range |
| Variable | `case x` | Anything, binds to `x` |
| Wildcard | `case _` | Anything, no binding |
| Type | `case s::String` | Value of type, binds |
| Tuple | `case (x, y)` | Tuple destructure |
| List | `case []`, `case [h \| t]` | List destructure |
| Dict | `case {key: v}` | Dict destructure |
| Enum | `case Shape.Circle(r)` | Variant + extract fields |
| Nested | `case Result.Ok(Option.Some(v))` | Arbitrarily nested |
| Or | `case 1 \| 2 \| 3` | Any of listed (no bindings) |
| Guard | `case x if x > 0` | Pattern + condition |
| As-binding | `case Shape.Circle(r) as s` | Destructure + bind whole |

When matching on an `enum` type, the compiler enforces **exhaustive matching** -- all variants must be covered or a `case _` catch-all must be present. See [Enums & Algebraic Data Types](../03-functions-and-types/enums-and-algebraic-types.md) for details.

## 10. Exhaustive Matching with Symbol Sets

When matching on a symbol set type, the compiler checks for exhaustiveness:

```opal
type Color = :red | :green | :blue

def describe(c::Color) -> String
  match c
    case :red   then "warm"
    case :green then "cool"
    # COMPILE WARNING: non-exhaustive match, missing :blue
  end
end
```
