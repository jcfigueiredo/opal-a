# Enums & Algebraic Data Types

---

## Overview

The `enum` keyword defines a closed set of variants. Variants without fields are simple named constants. Variants with fields carry data. One keyword covers both simple enums and full algebraic data types. Enums are closed, immutable, support methods and protocols, and enable exhaustive pattern matching.

---

## 1. Defining Enums

### Simple Enums (No Data)

```opal
enum Direction
  North
  South
  East
  West
end

enum Color
  Red
  Green
  Blue
end
```

### Data-Carrying Variants

```opal
enum Shape
  Circle(radius::Float64)
  Rectangle(width::Float64, height::Float64)
  Triangle(base::Float64, height::Float64)
end
```

### Mixed -- Some Variants Carry Data, Some Don't

```opal
enum Response
  Success(body::String, headers::Dict(String, String))
  NotFound(path::String)
  ServerError(reason::String, code::Int32)
  Unauthorized
end
```

### Construction

```opal
# Construction: positional or named
d = Direction.North
s = Shape.Circle(5.0)                     # positional (for 1-2 fields)
s = Shape.Circle(radius: 5.0)             # named (explicit)
r = Response.Unauthorized
r = Response.Success("hello", {:})        # positional
resp = Response.Success(body: "hello", headers: {:})  # named
```

### Rules

- Variants are accessed as `EnumName.VariantName`.
- Variants with fields support both positional and named arguments.
- Positional construction matches field declaration order.
- Variants without fields are singletons -- `Direction.North is Direction.North` is always true.
- Enums are closed -- you cannot add variants after definition.
- Enum values are immutable.

---

## 2. Pattern Matching & Exhaustiveness

Matching on an enum must cover all variants or include a catch-all `case _`. Missing a variant is a compile-time error.

### Exhaustive Matching

```opal
# All variants covered — ok
match direction
  case Direction.North
    move(0, 1)
  case Direction.South
    move(0, -1)
  case Direction.East
    move(1, 0)
  case Direction.West
    move(-1, 0)
end

# Catch-all satisfies exhaustiveness
match direction
  case Direction.North
    move(0, 1)
  case _
    stay()
end

# COMPILE ERROR — missing variants
match direction
  case Direction.North
    move(0, 1)
  case Direction.South
    move(0, -1)
end
```

### Destructuring Data-Carrying Variants

```opal
match shape
  case Shape.Circle(r)
    Math.pi * r ** 2
  case Shape.Rectangle(w, h)
    w * h
  case Shape.Triangle(b, h)
    0.5 * b * h
end
```

### Guards

```opal
match response
  case Response.ServerError(reason, code) if code >= 500
    retry()
  case Response.ServerError(reason, code)
    log(f"client error {code}: {reason}")
  case Response.Success(body, _)
    process(body)
  case _
    handle_other()
end
```

### Rules

- `match` on an enum type without covering all variants = compile-time error.
- `case _` satisfies exhaustiveness as a catch-all.
- Variants with fields are destructured directly in the `case` pattern.
- Guards (`if condition`) can further narrow a variant.
- Exhaustiveness checking only applies to enum types -- `match` on other types (Int32, String, etc.) still works without exhaustiveness.

---

## 3. Methods & Protocols

Enums can have methods and implement protocols. Methods operate on `self` and typically match across variants.

```opal
enum Shape implements Printable, Comparable
  Circle(radius::Float64)
  Rectangle(width::Float64, height::Float64)
  Triangle(base::Float64, height::Float64)

  def area() -> Float64
    match self
      case Shape.Circle(r)
        Math.pi * r ** 2
      case Shape.Rectangle(w, h)
        w * h
      case Shape.Triangle(b, h)
        0.5 * b * h
    end
  end

  def to_string() -> String
    match self
      case Shape.Circle(r)
        f"Circle(r={r})"
      case Shape.Rectangle(w, h)
        f"Rect({w}x{h})"
      case Shape.Triangle(b, h)
        f"Tri(b={b}, h={h})"
    end
  end

  def compare_to(other::Shape) -> Int32
    (.area() - other.area()) as Int32
  end
end

s = Shape.Circle(radius: 5.0)
s.area()        # => 78.539...
s.to_string()   # => "Circle(r=5.0)"
s.println()     # "Circle(r=5.0)" — default from Printable
```

### Rules

- Methods are defined inside the `enum` block, after the variants.
- Methods operate on `self`, which is one of the variants.
- Enums can `implements` protocols -- same syntax as classes.
- Retroactive conformance works on enums: `implements Printable for Direction`.

---

## 4. Generic Enums

Enums support type parameters, enabling foundational stdlib types like `Option(T)` and `Result(T, E)`.

```opal
enum Option(T)
  Some(value::T)
  None
end

enum Result(T, E)
  Ok(value::T)
  Err(error::E)
end
```

### Usage

```opal
# Type inferred from construction
opt = Option.Some(value: 42)       # Option(Int32)
opt = Option(String).None           # explicit when ambiguous

# Result in practice
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

### Constraints

```opal
enum SortedPair(T implements Comparable)
  Pair(first::T, second::T)
  Empty
end
```

### Relationship to `?` Nullable

- `T?` stays as `T | Null` -- lightweight nullable for everyday use.
- `Option(T)` is a stdlib enum for explicit `Some`/`None` with exhaustive matching.
- They are separate types. `Option(T)` does not replace `?`.

---

## 5. Composition with Existing Features

### Enums in Union Types

```opal
type ApiResponse = Response | Timeout | RateLimited

# Enum variants as function parameter types
def log_error(e::Response.ServerError | Response.NotFound)
  print(f"error: {e}")
end
```

### Enums in Collections

```opal
shapes = [Shape.Circle(radius: 1.0), Shape.Rectangle(width: 2.0, height: 3.0)]
total_area = shapes.map(|s| s.area()).reduce(0.0, |acc, a| acc + a)
```

### Enums with DI

```opal
class Renderer
  needs default_color::Color

  def render(shape::Shape)
    match shape
      case Shape.Circle(r)
        draw_circle(r, .default_color)
      case Shape.Rectangle(w, h)
        draw_rect(w, h, .default_color)
      case Shape.Triangle(b, h)
        draw_tri(b, h, .default_color)
    end
  end
end
```

### Enums in Events

```opal
enum OrderStatus
  Pending
  Confirmed(at::Time)
  Shipped(tracking::String)
  Delivered(at::Time)
  Cancelled(reason::String)
end

event OrderStatusChanged(order::Order, from::OrderStatus, to::OrderStatus)
```

### Enums with Type Aliases

```opal
type Result(T) = Result(T, Error)  # alias with default error type
```

---

## 6. Design Rationale

### Why One Keyword?

Many languages split enums (simple constants) and algebraic data types (data-carrying variants) into separate constructs. Opal unifies them under `enum` because the mental model is the same: a closed set of alternatives. Whether a variant carries data is an implementation detail, not a conceptual difference.

### Why Exhaustive Matching?

Exhaustiveness checking catches bugs at compile time. When you add a new variant to an enum, every `match` that doesn't handle it becomes a compile error -- the compiler tells you everywhere that needs updating. The `case _` catch-all is an explicit opt-out for cases where you genuinely want a default.

### Why Closed?

Enums are closed (no variants can be added after definition) because exhaustive matching requires it. If variants could be added dynamically, the compiler could not verify coverage. This is the fundamental trade-off: closedness enables safety.

### Why Immutable?

Enum values are immutable because they represent facts, not state. A `Direction.North` is always north. A `Shape.Circle(radius: 5.0)` always has radius 5.0. Immutability makes enums safe to share across threads and simple to reason about.

---

## Summary

| Feature | Decision |
|---|---|
| Keyword | `enum` -- one construct for simple constants and data-carrying variants |
| Data | Variants can optionally carry named, typed fields |
| Methods | Enums can have methods, operating on `self` via `match` |
| Protocols | Enums can `implements` protocols, including retroactive conformance |
| Generics | Type parameters with constraints, same as classes |
| Exhaustiveness | `match` on enum must cover all variants or have `case _` -- compile-time error |
| Nullable | `T?` stays as `T | Null`. `Option(T)` is a separate stdlib enum |
| Immutability | Enum values are immutable |
| Composition | Works with unions, collections, DI, events, type aliases |

### New Keywords

| Keyword | Purpose |
|---|---|
| `enum` | Define a closed set of variants (simple constants or data-carrying) |
