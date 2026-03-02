# Self-Hosting Foundations

---

## Overview

Five foundational features that enable Opal's standard library and ecosystem to be written in Opal itself. Without these, core types, collections, and error handling must be hardcoded in the runtime.

---

## 1. Operator Overloading

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
| String | `to_string()` (used by f-strings and `print`) |
| Truthiness | `to_bool()` (used by `if`, `while`, `and`, `or`) |
| Iteration | `iter()` (used by `for ... in`) |

### Not Overloadable

`=`, `and`, `or`, `not`, `..`, `...`, `is`, `as` — these preserve language semantics and readability.

### Rules

- Method form `def +(other::T)` inside a class is sugar for `def +(self::Self, other::T)`.
- Standalone form `def +(a::A, b::B)` dispatches on all argument types.
- Same multiple dispatch resolution as regular functions (exact type > precondition > arity > ambiguity error).
- `to_string()` is called automatically by f-strings and `print`.
- `to_bool()` is called automatically by conditionals.
- `iter()` is called automatically by `for ... in`.

---

## 2. Iterator Protocol

Two protocols: `Iterable` (the thing you iterate over) and `Iterator` (the cursor).

### Protocols

```opal
protocol Iterable
  def iter() -> Iterator
end

protocol Iterator(T)
  def next() -> Option(T)
end
```

### Custom Collection Example

```opal
class FileLines implements Iterable
  needs path::String

  def iter()
    FileLinesIterator.new(file: File.open(.path))
  end
end

class FileLinesIterator implements Iterator(String)
  needs file::File

  def next() -> Option(String)
    line = .file.read_line()
    if line == null
      Option.None
    else
      Option.Some(line)
    end
  end
end

# Works with for-in
for line in FileLines.new(path: "data.txt")
  print(line)
end

# Works with collection methods
FileLines.new(path: "data.txt")
  .map(|line| line.trim())
  .filter(|line| line.length > 0)
```

### Lazy Infinite Sequence

```opal
class Counter implements Iterable
  needs start::Int32

  def iter()
    CounterIterator.new(current: .start)
  end
end

class CounterIterator implements Iterator(Int32)
  needs current::Int32

  def next() -> Option(Int32)
    value = .current
    .current += 1
    Option.Some(value)  # never exhausted
  end
end

for n in Counter.new(start: 0).take(5)
  print(n)  # 0, 1, 2, 3, 4
end
```

### Rules

- Any class implementing `Iterable` works with `for ... in`.
- `Iterator.next()` returns `Option(T)` — `Some(value)` for the next element, `None` when exhausted.
- Built-in types (`List`, `Dict`, `Range`, `String`) all implement `Iterable`.
- Collection methods (`map`, `filter`, `reduce`, `take`, `zip`) work on any `Iterable`.

---

## 3. Custom Error Types

Errors are classes that inherit from `Error`. Define domain-specific errors by subclassing.

### Base Error (built-in)

```opal
class Error
  needs message::String

  def stack_trace() -> List(String)
    # provided by runtime
  end
end
```

### Defining Custom Errors

```opal
class FileNotFound < Error
  needs path::String

  def init(path)
    .path = path
    super(message: f"File not found: {path}")
  end
end

class NetworkError < Error
  needs url::String
  needs status::Int32

  def init(url, status)
    .url = url
    .status = status
    super(message: f"HTTP {status} from {url}")
  end
end

class ValidationError < Error
  needs field::String
  needs reason::String

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
catch ValidationError as e
  print(f"Bad field: {e.field} — {e.reason}")
catch Error as e
  print(f"Unexpected: {e.message}")
end
```

### Rules

- `class MyError < Error` defines a custom error type.
- `fail expr` raises any `Error` subclass.
- `catch Type as e` catches errors of that type **and its subclasses**.
- `Error` provides `.message` and `.stack_trace()` by default.
- Custom fields via `needs` (like any class).
- `super(message: ...)` chains to the parent Error constructor.

---

## 4. Destructuring Assignment

Pattern matching syntax extended to regular assignment, function params, for loops, and closures.

### Tuples

```opal
(x, y) = get_point()
(status, body) = http_get("/users")
(_, y) = get_point()        # ignore with _
(first, (a, b)) = (1, (2, 3))  # nested
```

### Dicts

```opal
{name: n, age: a} = {name: "claudio", age: 15, role: "admin"}
# n = "claudio", a = 15 (extra keys ignored)
# Missing required key = runtime error
# Optional: {name: n, age?: a} — a is null if missing
```

### Lists (head/tail)

```opal
[first, second | rest] = [1, 2, 3, 4, 5]
# first = 1, second = 2, rest = [3, 4, 5]

[head | _] = [10, 20, 30]
# head = 10
```

### In Function Parameters

```opal
def distance((x1, y1), (x2, y2))
  ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
end

distance((0, 0), (3, 4))  # => 5.0
```

### In For Loops

```opal
pairs = [("alice", 30), ("bob", 25)]
for (name, age) in pairs
  print(f"{name} is {age}")
end
```

### In Closures

```opal
points.map(|(x, y)| x + y)
```

### Rules

- Destructuring works in assignment `=`, function params, `for` loops, and closures.
- `_` ignores a value.
- `[head | tail]` splits a list into first element(s) and rest.
- Dict destructuring extracts by key; extra keys are ignored.
- Missing required keys = runtime error. Use `?` suffix for optional keys.
- Same pattern syntax as `match` — one way to do it everywhere.

---

## 5. Protocol Default Implementations

Protocols can provide default method bodies. Implementors only need to define required abstract methods.

### Defining Defaults

```opal
protocol Comparable
  # Required — implementor must define this
  def compare_to(other) -> Int32

  # Defaults — derived from compare_to
  def <(other) -> Bool
    .compare_to(other) < 0
  end

  def >(other) -> Bool
    .compare_to(other) > 0
  end

  def <=(other) -> Bool
    .compare_to(other) <= 0
  end

  def >=(other) -> Bool
    .compare_to(other) >= 0
  end
end
```

### Implementing

```opal
# Only define compare_to — get <, >, <=, >= for free
class Temperature implements Comparable
  needs degrees::Float64

  def compare_to(other::Temperature) -> Int32
    (.degrees - other.degrees) as Int32
  end
end

a = Temperature.new(degrees: 20.0)
b = Temperature.new(degrees: 30.0)
a < b    # => true (from default)
a >= b   # => false (from default)
```

### Rich Protocol Example

```opal
protocol Printable
  # Required
  def to_string() -> String

  # Defaults
  def print()
    IO.print(.to_string())
  end

  def println()
    IO.println(.to_string())
  end

  def inspect() -> String
    f"<{typeof(self).name}: {.to_string()}>"
  end
end

class User implements Printable
  needs name::String

  def to_string()
    .name
  end

  # Override default
  def inspect()
    f"<User name={.name}>"
  end
end
```

### Combining Protocols

```opal
protocol Hashable
  def hash_code() -> Int32

  def ==(other) -> Bool
    .hash_code() == other.hash_code()
  end
end

class Point implements Comparable, Hashable, Printable
  needs x::Int32
  needs y::Int32

  def compare_to(other::Point) -> Int32
    (.x + .y) - (other.x + other.y)
  end

  def hash_code() -> Int32
    .x * 31 + .y
  end

  def to_string() -> String
    f"({.x}, {.y})"
  end
end
# Gets: <, >, <=, >=, ==, print(), println(), inspect() — all from defaults
```

### Rules

- Methods **with** a body in a protocol are defaults — inherited by implementors.
- Methods **without** a body are required — implementor must define them.
- Implementors can override any default.
- If two protocols provide conflicting defaults for the same method name, the implementor must explicitly define it (ambiguity = compile-time error).

---

## What These Five Features Unlock

| What You Can Now Write in Opal | Features Used |
|---|---|
| `Range`, `List`, `Dict`, `Set` | Operator overloading + iterator protocol |
| `File`, `Net`, `IO` (streaming) | Custom errors + iterator protocol |
| `String` utilities | Operator overloading (`+`, `[]`) |
| `Test` framework | Macros + custom errors |
| `JSON`, `YAML` parsers | Destructuring + pattern matching |
| Custom collections (trees, graphs) | Iterator + operator overloading |
| `Comparable`, `Hashable`, `Printable` | Protocol defaults |
| Fluent APIs | Operator overloading + method chaining |
| Domain models | Destructuring + custom errors + DI |
