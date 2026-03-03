# Classes & Inheritance

---

## Overview

Opal classes use `needs` for declarative field/dependency injection and `init` for imperative construction logic. Both can coexist -- `needs` fields are injected first, then `init` runs. Inheritance is single-parent only; protocols fill the multiple-behavior gap.

---

## 1. Classes & Methods

Classes use `def init()` for construction. Instance variables are accessed with the `.` prefix.

```opal
class Person
  def init(name = "anonymous", age = 0)
    .name = name
    .age = age
    .started = true
  end

  def greet()
    print(f"Hi, my name is {.name}")
  end

  # Names ending in ? are for predicates
  def adult?()
    .age >= 18
  end

  # Names ending in ! are for mutations
  def rename!(new_name)
    .name = new_name
  end

  # Static method (defined with self.)
  def self.species()
    "Homo sapiens"
  end
end

# Object creation with .new() and named arguments
claudio = Person.new(name: "claudio", age: 15)
claudio.greet()         # => "Hi, my name is claudio"
claudio.adult?()        # => false
Person.species()        # => "Homo sapiens"
```

---

## 2. Constructor Shorthand

`Type(args)` is sugar for `Type.new(args)`. Both forms are equivalent.

```opal
# These are identical
point = Point.new(x: 1.0, y: 2.0)
point = Point(x: 1.0, y: 2.0)

# The shorthand is especially useful for nested construction
user = User(
  name: "claudio",
  address: Address(
    street: "123 Main",
    city: "Springfield"
  )
)

# .new() remains available and is equivalent
user = User.new(
  name: "claudio",
  address: Address.new(street: "123 Main", city: "Springfield")
)
```

**Rules:**
- `Type(args)` is sugar for `Type.new(args)`.
- Works for classes, models, and error types.
- No ambiguity: class names are PascalCase, functions are snake_case.
- Enum variants already use this form: `Shape.Circle(radius: 5.0)`.

---

## 3. Construction Model

### `needs` Only (Most Common)

For classes that are pure data or DI containers, `needs` is all you need:

```opal
class Point
  needs x: Float64
  needs y: Float64
end

p = Point.new(x: 1.0, y: 2.0)
p.x   # => 1.0
```

### `needs` + `init` Together

When you need setup logic beyond field assignment, add `init`. The `needs` fields are injected *before* `init` runs, so they're available inside it:

```opal
class OrderService
  needs db: Database
  needs mailer: Mailer

  def init(retry_count = 3)
    # .db and .mailer already available
    .retry_count = retry_count
    .cache = {:}
  end
end

# Construction -- needs + init args combined
service = OrderService.new(
  db: PostgresDB.new(),
  mailer: SMTPMailer.new(),
  retry_count: 5
)
```

### `init` Only

For classes with complex construction logic or no declarative fields:

```opal
class Parser
  def init(source: String)
    .source = source
    .position = 0
    .tokens = []
  end
end

p = Parser.new(source: "hello world")
```

### Construction Order

1. Parent `needs` fields injected (if inheriting)
2. Own `needs` fields injected
3. Parent `init` runs (if present)
4. Own `init` runs (if present)

All `needs` fields from the entire hierarchy are required at `.new()`.

---

## 4. Inheritance

Opal uses **single inheritance** only. For multiple behaviors, use protocols.

```opal
class Animal
  needs name: String
  needs sound: String

  def speak()
    print(f"{.name} says {.sound}")
  end
end

class Dog < Animal
  needs breed: String

  def speak()
    super()  # calls Animal.speak
    print(f"({.breed})")
  end
end

rex = Dog.new(name: "Rex", sound: "Woof", breed: "Labrador")
rex.speak()
# => "Rex says Woof"
# => "(Labrador)"
```

### Inherited `needs`

A subclass inherits all parent `needs`. The subclass constructor requires both parent and own `needs`:

```opal
class Animal
  needs name: String
  needs sound: String
end

class Dog < Animal
  needs breed: String
end

# Must provide ALL needs (parent + own)
rex = Dog.new(
  name: "Rex",       # from Animal
  sound: "Woof",     # from Animal
  breed: "Labrador"  # from Dog
)

rex.name    # => "Rex" (inherited field)
rex.breed   # => "Labrador" (own field)
```

### `super`

`super` calls the parent's version of the current method:

```opal
class Animal
  def describe() -> String
    f"{.name} the animal"
  end
end

class Dog < Animal
  def describe() -> String
    f"{super()} (breed: {.breed})"
  end
end

rex.describe()  # => "Rex the animal (breed: Labrador)"
```

In `init`, `super()` calls the parent's `init` if it exists. If the parent has no `init` (only `needs`), `super()` is not needed -- `needs` injection is automatic.

```opal
class Base
  def init()
    .created_at = Time.now()
  end
end

class Child < Base
  needs value: Int32

  def init()
    super()           # calls Base.init -- .created_at set
    .computed = .value * 2
  end
end
```

---

## 5. No Abstract Classes

Protocols replace abstract classes. Use protocols for contracts with optional default implementations:

```opal
# Instead of abstract class Shape:
protocol Shape
  def area() -> Float64       # required
  def perimeter() -> Float64  # required

  def describe() -> String    # default
    f"Shape with area {.area()}"
  end
end

class Circle implements Shape
  needs radius: Float64

  def area() -> Float64
    Math.PI * .radius ** 2
  end

  def perimeter() -> Float64
    2.0 * Math.PI * .radius
  end
end
```

---

## 6. No Multiple Inheritance

For multiple behaviors, combine single inheritance with protocols:

```opal
class Dog < Animal implements Trainable, Printable
  needs breed: String

  def to_string() -> String
    f"{.name} the {.breed}"
  end

  def learn(command: String)
    print(f"{.name} learned {command}!")
  end
end
```

---

## 7. Design Rationale

### Why `needs` + `init`?

The two-phase construction model separates declarative dependencies (`needs`) from imperative setup logic (`init`). This split makes dependency injection natural -- `needs` fields are the class's contract with the outside world, while `init` handles internal wiring. Most classes only need `needs`, which keeps the common case simple.

### Why Single Inheritance Only?

Multiple inheritance introduces the diamond problem and makes construction order ambiguous. Single inheritance with protocols gives the same expressiveness: inherit state and behavior from one parent, and compose additional behaviors via protocol conformance. This is the same trade-off made by Rust (traits), Swift (protocols), and Kotlin (interfaces with defaults).

---

## Summary

| Feature | Decision |
|---|---|
| Construction | `needs` for declarative fields, `init` for setup logic, both allowed |
| Constructor shorthand | `Type(args)` is sugar for `Type.new(args)` |
| Construction order | Parent needs -> own needs -> parent init -> own init |
| Inheritance | Single parent only via `<` |
| Inherited needs | Automatic -- subclass `.new()` requires all ancestor needs |
| `super` | Calls parent's version of current method |
| Abstract classes | No -- use protocols with default methods |
| Multiple inheritance | No -- use `implements Protocol` for multi-behavior |
| Multiple behaviors | Combine `< Parent` with `implements Protocol, Protocol` |
