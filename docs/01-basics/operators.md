# Operators

---

## Overview

Opal provides a full set of arithmetic, comparison, logical, membership, and assignment operators. Operators are methods under the hood, enabling user-defined types to participate in natural syntax via operator overloading and multiple dispatch. The pipe operator and null-safe chaining round out the operator suite for readable data-flow and safe navigation.

---

## 1. Arithmetic Operators

| Operator | Description |
|---|---|
| `+` | Addition |
| `-` | Subtraction / Unary negation |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |
| `**` | Exponentiation |

```opal
2 ** 10          # => 1024
17 % 5           # => 2
```

## 2. Comparison Operators

| Operator | Description |
|---|---|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

```opal
# Comparison chaining
1 < x and x < 10
```

## 3. Logical Operators

| Operator | Description |
|---|---|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT |

```opal
ready = loaded and not errored
```

## 4. Membership Operators

| Operator | Description |
|---|---|
| `in` | Membership test |
| `not in` | Negated membership test |

```opal
3 in [1, 2, 3, 4]         # => true
"key" in {"key": "value"}  # => true (checks keys)
"c" in "a".."z"            # => true
42 not in [1, 2, 3]        # => true

# in is sugar for .contains?()
list.contains?(3)          # equivalent to: 3 in list
```

## 5. Assignment Operators

| Operator | Description |
|---|---|
| `=` | Assignment |
| `+=` | Add and assign |
| `-=` | Subtract and assign |
| `*=` | Multiply and assign |
| `/=` | Divide and assign |

## 6. Operator Overloading

Operators are methods. The method form (inside a class) is sugar for the standalone form (outside a class). Both use the same multiple dispatch mechanism underneath.

### Method Form (in-class)

```opal
class Vector
  needs x::Float64
  needs y::Float64

  # Arithmetic
  def +(other::Vector) -> Vector
    Vector.new(x: .x + other.x, y: .y + other.y)
  end

  def -(other::Vector) -> Vector
    Vector.new(x: .x - other.x, y: .y - other.y)
  end

  def -() -> Vector  # unary negation
    Vector.new(x: -.x, y: -.y)
  end

  # Indexing
  def [](index::Int32) -> Float64
    if index == 0 then .x else .y end
  end

  def []=(index::Int32, value::Float64)
    if index == 0 then .x = value else .y = value end
  end

  # Comparison
  def ==(other::Vector) -> Bool
    .x == other.x and .y == other.y
  end

  # String representation (used by f-strings and print)
  def to_string() -> String
    f"({.x}, {.y})"
  end
end
```

### Standalone Form (cross-type, extension)

```opal
# Cross-type operators — neither class needs to know about the other
def *(scalar::Float64, v::Vector) -> Vector
  Vector.new(x: scalar * v.x, y: scalar * v.y)
end

def *(v::Vector, scalar::Float64) -> Vector
  scalar * v
end

# Third-party extension — add operators to types you don't own
def +(v::Vector, m::Matrix) -> Matrix
  # ...
end
```

### Usage

```opal
a = Vector.new(x: 1.0, y: 2.0)
b = Vector.new(x: 3.0, y: 4.0)
c = a + b          # => (4.0, 6.0)
d = 2.0 * a        # => (2.0, 4.0)
a[0]               # => 1.0
print(f"result: {c}")  # => "result: (4.0, 6.0)"
```

### Overloadable Operators

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `**`, unary `-` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Indexing | `[]`, `[]=` |
| Membership | `in` (delegates to `contains?()`) |
| Conversion | `to_string()`, `to_bool()`, `iter()` |

### Not Overloadable

`=`, `and`, `or`, `not`, `..`, `...`, `is`, `as`, `|>`, `?.`, `??` — these preserve language semantics and readability.

## 7. Pipe Operator

The pipe operator `|>` passes the result of the left-hand expression as the first argument to the right-hand function. It enables readable left-to-right data transformation chains.

```opal
# Basic pipeline — each step feeds into the next
result = read_file("data.csv") |> parse |> validate |> format

# Pipe with additional arguments — the piped value becomes the first argument
active_users = users |> filter(|u| u.active?()) |> take(10)

# Multi-line pipeline
report = transactions
  |> filter(|t| t.amount > 100)
  |> group_by(|t| t.category)
  |> map(|(cat, txns)| (cat, txns.sum(|t| t.amount)))
  |> sort_by(|(_, total)| total, descending: true)
```

The pipe operator is left-associative: `a |> b |> c` is equivalent to `c(b(a))`.

## 8. Null-Safe Chaining and Null Coalescing

The `?.` operator short-circuits a method or property chain when the receiver is `null`, returning `null` instead of raising an error. The `??` operator provides a default value when the left-hand side is `null`.

```opal
# Null-safe property access — returns null if user or address is null
city = user?.address?.city

# Null-safe method calls — returns null if any step is null
length = user?.name?.upper()?.length

# Null coalescing — provide a default when the value is null
city = user?.address?.city ?? "Unknown"
display_name = user?.name ?? "Anonymous"

# Combine with regular method calls
greeting = f"Hello, {user?.name?.capitalize() ?? "Guest"}"
```

`?.` propagates `null` — if the receiver is `null`, the entire chain from that point evaluates to `null` without executing further methods. `??` evaluates its right-hand side only when the left-hand side is `null`.

## 9. Design Rationale

Operator overloading is one of five self-hosting foundations that enable Opal's standard library and ecosystem to be written in Opal itself. Without it, core types like `Range`, `List`, `Dict`, and `String` would need operator behavior hardcoded in the runtime.

### Why Operators Are Methods

- Method form `def +(other::T)` inside a class is sugar for `def +(self::Self, other::T)`.
- Standalone form `def +(a::A, b::B)` dispatches on all argument types.
- Same multiple dispatch resolution as regular functions (exact type > precondition > arity > ambiguity error).
- `to_string()` is called automatically by f-strings and `print`.
- `to_bool()` is called automatically by conditionals.
- `iter()` is called automatically by `for ... in`.

### What Operator Overloading Unlocks

| What You Can Now Write in Opal | Features Used |
|---|---|
| `Range`, `List`, `Dict`, `Set` | Operator overloading + iterator protocol |
| `String` utilities | Operator overloading (`+`, `[]`) |
| Custom collections (trees, graphs) | Iterator + operator overloading |
| Fluent APIs | Operator overloading + method chaining |

## Summary

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `**` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `and`, `or`, `not` |
| Membership | `in`, `not in` |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=` |
| Pipe | `\|>` |
| Null-safe | `?.`, `??` |
| Overloadable | Arithmetic, comparison, indexing (`[]`, `[]=`), membership, conversion |
| Not overloadable | `=`, `and`, `or`, `not`, `..`, `...`, `is`, `as`, `\|>`, `?.`, `??` |
