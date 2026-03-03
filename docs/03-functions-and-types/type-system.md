# Type System

---

## Overview

Opal's type system is **gradual** -- unannotated code is fully dynamic, annotated code is checked at boundaries (function entry, return, annotated assignment). Types serve two equal purposes: catching bugs early and documenting intent.

---

## 1. Core Rules

### Annotation Syntax

- `: Type` annotates parameters, variables, and return types.
- No annotation = dynamic, no checking.
- `as` performs explicit casting.
- `?` suffix = nullable (`T?` is sugar for `T | Null`).

### Boundary Checking

Types are checked when values cross boundaries: function call sites, return points, and annotated variable assignments. Internal code within a function body is not checked.

```opal
# No annotations -- fully dynamic
def add(a, b)
  a + b
end

# Annotated -- type-checked at boundaries
def add(a: Int32, b: Int32) -> Int32
  x = a + b   # x is NOT checked (internal)
  x            # checked against -> Int32 on return
end

add(1, 2)      # checked: args are Int32
add(1, "hi")   # TYPE ERROR at call site

# Annotated assignment -- checked
name: String = "claudio"
age: Int32 = 15

# Explicit casting with `as`
x = 3.14 as Int32   # => 3

# Optional types (nullable)
def find(id: Int32) -> Person?
  # may return null -- Person? is sugar for Person | Null
end
```

### Boundary Checking Rules

- Unannotated parameters and variables are dynamic -- no checking.
- Annotated parameters are checked at call sites.
- Return type annotations are checked at function exit.
- Annotated variable assignments are checked when assigned.
- Internal variables without annotations are unchecked.
- `as` performs explicit type conversion. Raises a runtime error if conversion fails.
- `?` suffix denotes a nullable type (e.g., `String?` means `String | Null`).

### Core Types

`Int8`, `Int16`, `Int32`, `Int64`, `Float32`, `Float64`, `Bool`, `String`, `Template`, `Symbol`, `Null`, `List[T]`, `Tuple[...]`, `Dict[K, V]`, `Range[T]`, `Regex`.

### Symbol Sets

Symbol sets are type aliases over unions of symbol literals:

```opal
type Status = :ok | :error | :pending
type Direction = :north | :south | :east | :west
```

Symbol sets enable exhaustiveness checking in `match` and type-safe function signatures. They are for simple tags with no data -- use `enum` for data-carrying variants.

---

## 2. Generics

Type parameters are declared explicitly on classes, protocols, and type aliases. At call sites, they're inferred from arguments.

### Defining Generic Classes

```opal
class Stack[T]
  needs items: List[T]

  def push(item: T)
    .items.append(item)
  end

  def pop() -> T?
    .items.pop()
  end
end
```

### Using Generic Classes

```opal
# Inferred at call site
s = Stack.new(items: [1, 2, 3])   # T = Int32
s.push(42)    # ok
s.push("hi")  # type error

# Explicit when ambiguous (e.g., empty collection)
s = Stack[Int32].new(items: [])
```

### Generic Functions

The type parameter is inferred from annotated arguments -- no separate declaration needed:

```opal
def first(items: List[T]) -> T?
  items[0]
end

first([1, 2, 3])       # T inferred as Int32, returns Int32?
first(["a", "b"])       # T inferred as String, returns String?
```

### Rules

- Type parameters are declared at the definition site: `class Name[T]`, `protocol Name[T]`, `type Name[T] = ...`.
- At call sites, type parameters are inferred from arguments when possible.
- Explicit type parameters at call sites (`Stack[Int32]`) only needed when inference is ambiguous.
- Type parameters are scoped to their definition -- `T` in `Stack[T]` is not the same as `T` in another class.

---

## 3. Generic Constraints

Constraints restrict what types can fill a type parameter. Simple constraints go inline, complex ones use a `where` clause.

### Inline Constraints

```opal
# Single constraint on one parameter
class SortedList[T implements Comparable]
  needs items: List[T]

  def insert(item: T)
    # compare_to guaranteed available
  end
end
```

### Where Clause

```opal
# Multiple parameters or multiple constraints
class Cache[K, V]
    where K implements Hashable,
          V implements Printable
  needs store: Dict[K, V]
end
```

### On Functions

```opal
# Where clause on functions
def max(a: T, b: T) -> T
    where T implements Comparable
  if a > b then a else b end
end

# Inline on functions for simple cases
def sort(items: List[T implements Comparable]) -> List[T]
  # ...
end
```

### Rules

- `T implements Protocol` restricts a type parameter to types that implement the protocol.
- Inline for single constraint on a single parameter.
- `where` clause for multiple parameters or multiple constraints.
- Both forms are equivalent -- choose whichever reads better.
- Constraint violations are caught at the call site when the concrete type is known.

---

## 4. Union Types

A value can be one of several types, expressed with `|`. The nullable `?` suffix is sugar for `T | Null`.

```opal
# Union return type
def parse(input: String) -> Int32 | Float64 | Error
  # can return any of these
end

# Pattern match to narrow
match parse("42")
  case n: Int32
    print(f"integer: {n}")
  case f: Float64
    print(f"float: {f}")
  case e: Error
    print(f"error: {e.message}")
end

# Nullable is sugar
def find(id: Int32) -> Person?
  # identical to -> Person | Null
end

# Union in variable annotations
result: String | Int32 = get_value()

# Union in parameters
def display(value: String | Int32 | Float64)
  print(f"{value}")
end
```

### Rules

- `A | B` is a union -- the value is one of the listed types.
- `T?` is exactly `T | Null`.
- Unions are checked at boundaries like all other type annotations.
- Pattern matching with `case x: Type` narrows a union to a specific type.
- Unions are unordered -- `Int32 | String` is the same type as `String | Int32`.

---

## 5. Type Aliases

The `type` keyword names a complex type. Aliases are transparent -- the alias and the original type are fully interchangeable.

```opal
# Simple aliases -- semantic names for primitives
type UserID = Int64
type Email = String

# Parameterized aliases
type Result[T] = T | Error
type Pair[A, B] = (A, B)

# Function type alias
type Handler = Fn(Request, Response) -> Null

# Usage
def find_user(id: UserID) -> Result[User]
  # returns User | Error
end

def register(handler: Handler)
  handler(req, res)
end
```

### Rules

- `type Name = Type` creates a transparent alias.
- `type Name[T] = ...` creates a parameterized alias.
- Aliases are interchangeable with their underlying type -- `UserID` and `Int64` are the same type. No "newtype" distinction.
- Aliases can reference other aliases, unions, generics, and function types.

---

## 6. Nominal Typing with Retroactive Conformance

Types are nominal -- a class must declare `implements Protocol` to satisfy it. For types you don't own, retroactive conformance provides an escape hatch.

### Nominal by Default

```opal
protocol Drawable
  def draw() -> String
end

class Circle implements Drawable
  def draw() -> String
    f"circle at ({.x}, {.y})"
  end
end

class Coin
  def draw() -> String  # same shape, but NOT Drawable
    "coin"
  end
end

def render(shape: Drawable)
  shape.draw()
end

render(Circle.new(x: 0, y: 0))  # ok
render(Coin.new())               # TYPE ERROR -- Coin doesn't implement Drawable
```

### Retroactive Conformance

```opal
# Add conformance to types you don't own
implements Drawable for ThirdPartyShape
  def draw() -> String
    .render()  # delegate to existing method
  end
end

render(ThirdPartyShape.new())  # now works

# Conform built-in types
implements Printable for Int32
  def to_string() -> String
    # already exists, but now Int32 formally implements Printable
  end
end
```

### Rules

- Classes must declare `implements Protocol` -- having the right methods isn't enough.
- `implements Protocol for Type` adds conformance after the fact, for types you don't own.
- Retroactive conformance can define new methods or delegate to existing ones.
- Retroactive conformance cannot access private fields of the target type.
- If two retroactive conformances conflict, the one in the current module wins (same as multiple dispatch resolution).

---

## 7. Runtime Introspection

```opal
# Type of a value
typeof(42)          # => Int32
typeof("hello")     # => String
typeof([1, 2, 3])   # => List[Int32]

# Type narrowing with `is`
if value is String
  # value is known to be String here
  print(value.length)
end

# `is` with unions
def handle(result: Int32 | String | Error)
  if result is Error
    print(f"failed: {result.message}")
  else
    print(f"ok: {result}")
  end
end

# `is` with protocols
if shape is Drawable
  shape.draw()
end
```

### Rules

- `typeof(expr)` returns the runtime type as a Type object.
- `is` checks if a value is an instance of a type, protocol, or union member.
- `is` narrows the type in the enclosing branch (flow-sensitive narrowing).
- `as` converts between compatible types. Raises a runtime error if conversion fails.

---

## 8. Composition with Existing Features

### Generics + Multiple Dispatch

```opal
def serialize(value: Int32) -> String
  f"{value}"
end

def serialize(value: String) -> String
  f"\"{value}\""
end

def serialize(value: List[T]) -> String
  items = value.map(|v| serialize(v)).join(", ")
  f"[{items}]"
end

serialize(42)            # dispatches to Int32 variant
serialize([1, 2, 3])    # dispatches to List[T] variant, T = Int32
```

### Generics + Protocols

```opal
protocol Collection[T]
  def add(item: T)
  def contains?(item: T) -> Bool
  def size() -> Int32
end

class Set[T implements Hashable] implements Collection[T]
  def add(item: T)
    # ...
  end

  def contains?(item: T) -> Bool
    # ...
  end

  def size() -> Int32
    .items.length
  end
end
```

### Type Aliases + Generics + Constraints

```opal
type SortedPair[T implements Comparable] = (T, T)

def make_sorted_pair(a: T, b: T) -> SortedPair[T]
    where T implements Comparable
  if a <= b then (a, b) else (b, a) end
end
```

### Unions + Pattern Matching

```opal
type Result[T] = T | Error

def divide(a: Float64, b: Float64) -> Result[Float64]
  if b == 0.0
    fail Error.new(message: "division by zero")
  end
  a / b
end

match divide(10.0, 3.0)
  case value: Float64
    print(f"result: {value}")
  case e: Error
    print(f"error: {e.message}")
end
```

---

## 9. Design Rationale

### Why Gradual Typing?

Gradual typing lets Opal serve both scripting and systems use cases. Unannotated code behaves like a dynamic language -- fast to write, no ceremony. Annotated code gets static-like guarantees at boundaries. This means teams can start dynamic and add types incrementally as code matures, rather than choosing up-front between "typed" and "untyped."

### Why Nominal (Not Structural)?

Structural typing (if it has the right methods, it fits) causes accidental conformance: a `Coin.draw()` method would accidentally satisfy a `Drawable` protocol. Nominal typing requires explicit intent, which is safer for large codebases. Retroactive conformance (`implements Protocol for Type`) provides the escape hatch for types you don't own, giving the flexibility of structural typing without the accidents.

### Why Boundary-Only Checking?

Checking only at boundaries (call sites, returns, annotated assignments) avoids the verbosity of full static typing while catching the most common bugs. Internal variables within a function body are trusted -- the programmer knows what they're doing locally. This keeps the annotation burden low while still providing safety at API boundaries.

---

## Summary

| Feature | Decision |
|---|---|
| Philosophy | Gradual typing -- safety AND documentation equally |
| Generics | Explicit at definition, inferred at call site |
| Constraints | Inline for simple (`T implements P`), `where` clause for complex |
| Union types | `A \| B` syntax, `T?` is sugar for `T \| Null` |
| Type aliases | `type Name = Type`, transparent, parameterizable |
| Nominal typing | Must declare `implements`, with `implements P for T` escape hatch |
| Checking | Boundaries only -- call sites, returns, annotated assignments |
| Runtime introspection | `typeof()`, `is` (flow-sensitive narrowing), `as` |

### New Keywords

| Keyword | Purpose |
|---|---|
| `type` | Declare a type alias |
| `implements ... for` | Retroactive protocol conformance |
| `where` | Generic constraint clause |
| `is` | Runtime type check with flow narrowing |
