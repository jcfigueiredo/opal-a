# Protocols

---

## Overview

Protocols define a contract that classes must fulfill. Methods without a body are required -- implementors must define them. Methods with a body are defaults -- inherited automatically, overridable. Opal uses nominal typing: a class must declare `implements Protocol` to satisfy it.

---

## 1. Defining Protocols

Protocols declare a set of methods. Required methods have no body. Default methods include a body that implementors inherit automatically.

```opal
protocol Printable
  # Required
  def to_string() -> String

  # Defaults — derived from to_string
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

protocol Comparable
  # Required
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

---

## 2. Implementing Protocols

A class declares `implements ProtocolName` and defines all required methods. Default methods are inherited automatically but can be overridden.

```opal
class Person implements Printable
  def init(name, age)
    .name = name
    .age = age
  end

  def to_string()
    f"{.name}, age {.age}"
  end

  # Override a default
  def inspect()
    f"<Person name={.name} age={.age}>"
  end
end

person = Person.new(name: "claudio", age: 15)
person.println()   # "claudio, age 15" (default, calls to_string)
person.inspect()   # "<Person name=claudio age=15>" (overridden)
```

Only defining the required method is enough -- the implementor gets all defaults for free:

```opal
class Temperature implements Comparable
  needs degrees: Float64

  def compare_to(other: Temperature) -> Int32
    (.degrees - other.degrees) as Int32
  end
end

a = Temperature.new(degrees: 20.0)
b = Temperature.new(degrees: 30.0)
a < b    # => true (from default)
a >= b   # => false (from default)
```

---

## 3. Multiple Protocols

A class can implement multiple protocols and receives all their defaults.

```opal
protocol Hashable
  # Required
  def hash_code() -> Int32

  # Default
  def ==(other) -> Bool
    .hash_code() == other.hash_code()
  end
end

class Temperature implements Printable, Comparable, Hashable
  def init(degrees: Float32)
    .degrees = degrees
  end

  def to_string()
    f"{.degrees}°"
  end

  def compare_to(other: Temperature) -> Int32
    (.degrees - other.degrees) as Int32
  end

  def hash_code() -> Int32
    .degrees as Int32
  end
end

a = Temperature.new(degrees: 20.0)
b = Temperature.new(degrees: 30.0)
a < b     # => true (default from Comparable)
a.println()  # "20.0°" (default from Printable)
```

If two protocols provide conflicting defaults for the same method name, the implementor must explicitly define it (ambiguity = compile-time error).

---

## 4. Nominal Typing

Opal uses **nominal typing** -- a class must declare `implements Protocol` to satisfy it. Having the right methods is not enough:

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
render(Coin.new())               # TYPE ERROR — Coin doesn't implement Drawable
```

---

## 5. Retroactive Conformance

Retroactive conformance lets you add protocol conformance to types you don't own:

```opal
implements Drawable for ThirdPartyShape
  def draw() -> String
    .render()  # delegate to existing method
  end
end

render(ThirdPartyShape.new())  # now works
```

**Rules:**
- `implements Protocol for Type` adds conformance after the fact.
- Can define new methods or delegate to existing ones.
- Cannot access private fields of the target type.
- If two retroactive conformances conflict, the one in the current module wins.

---

## 6. Generic Protocols

Protocols support type parameters like classes:

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

---

## 7. Design Rationale

### Why Protocol Defaults?

Protocol defaults solve the "boilerplate problem" common in interface-heavy languages. By allowing protocols to provide method bodies, Opal lets library authors define rich abstractions where implementors only need one or two required methods to unlock a full suite of derived behavior.

**Key design decisions:**

- **Methods with a body are defaults, methods without are required.** The distinction is purely syntactic -- no extra keyword needed. This keeps protocol definitions readable and intuitive.
- **Implementors can override any default.** Defaults are a convenience, not a constraint. If a class has a more efficient implementation, it can replace the default.
- **Conflicting defaults are a compile-time error.** When two protocols provide defaults for the same method name, the implementor must resolve the ambiguity explicitly. This prevents silent, surprising behavior.

### Why Nominal Typing?

Structural typing (having the right methods is enough) creates fragile contracts -- a `Coin.draw()` and a `Shape.draw()` happen to share a name but mean different things. Nominal typing requires explicit intent, which:
- Makes code greppable (search for `implements Drawable` to find all drawable types).
- Prevents accidental conformance.
- Enables retroactive conformance as an explicit opt-in for types you don't own.

### Why Retroactive Conformance?

Third-party libraries may define types that fit your protocols perfectly but don't know about them. Retroactive conformance bridges this gap without modifying the original type. The rules (no private access, module-local conflict resolution) keep it safe.

---

## Summary

| Feature | Decision |
|---|---|
| Keyword | `protocol` -- defines a contract with required and default methods |
| Required methods | Methods without a body -- implementor must define |
| Default methods | Methods with a body -- inherited automatically, overridable |
| Nominal typing | Must declare `implements Protocol` -- having methods is not enough |
| Multiple protocols | `class X implements A, B, C` -- receives all defaults |
| Conflict resolution | Conflicting defaults = compile-time error, implementor must resolve |
| Retroactive conformance | `implements Protocol for Type` -- add conformance to types you don't own |
| Generic protocols | `protocol Collection[T]` -- type parameters with constraints |
