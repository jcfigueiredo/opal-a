# Class Lifecycle Design

## Goal

Implement four missing class features that the spec defines but aren't working: `init()` field persistence, `to_string()` protocol, `def self.method()` static methods, and `Type(args)` constructor shorthand.

## Feature 1: `init()` Lifecycle

### Current behavior
- `needs` fields are injected during `.new()` — works
- `def init()` is defined but fields set with `.field = value` inside init don't persist on the instance
- Field mutation in regular methods (`.field = value`) works fine

### Root cause
`.new()` creates the instance with `needs` fields, but doesn't call `init()` afterward. The `init()` method exists as a regular method but is never auto-invoked.

### Design
After `.new()` injects `needs` fields, check if the class (or any parent) has an `init` method. If so, call it with `self` set to the new instance.

```opal
class Base
  needs x: Int

  def init()
    .computed = .x * 10    # persists — init runs after needs injection
  end
end

class Child < Base
  needs y: Int

  def init()
    super()                # calls Base.init() — sets .computed
    .total = .computed + .y
  end
end

c = Child.new(x: 3, y: 5)
c.computed  # => 30
c.total     # => 35
```

### Rules
- `init()` runs automatically after `needs` injection during `.new()`
- `init()` receives no arguments (field values come from `needs`)
- `.field = value` in `init()` creates new instance fields (same as in regular methods)
- `super()` in `init()` calls parent's `init()` — works naturally with existing super() dispatch
- If no `init()` defined, `.new()` works as before (needs-only)
- Parent `init()` is NOT auto-called — child must call `super()` explicitly if needed

## Feature 2: `to_string()` Protocol

### Current behavior
- `print(instance)` shows `<ClassName instance>`
- `f"{instance}"` shows `<ClassName instance>`
- Defining `to_string()` on a class has no effect on print/f-string output

### Design
When converting an instance to string (for `print`, f-strings, string concatenation), check if the class has a `to_string` method. If so, call it and use the result.

```opal
class Dog
  needs name: String
  needs breed: String

  def to_string()
    f"{.name} the {.breed}"
  end
end

d = Dog.new(name: "Rex", breed: "Lab")
print(d)           # => "Rex the Lab"
print(f"my {d}")   # => "my Rex the Lab"
```

### Rules
- `to_string()` must return a String
- Called by: `print()`, f-string interpolation, string concatenation (`"hello " + obj`)
- If not defined, falls back to `<ClassName instance>` (current behavior)
- Inherited — if parent defines `to_string()`, child gets it

## Feature 3: `def self.method()` Static Methods

### Current behavior
- `def self.species()` causes parse error: "unexpected token SelfKw"
- The spec shows `Person.species()` as a class-level method

### Design
Allow `self.` prefix in method definitions inside class bodies. These become class methods callable on the class itself, not on instances.

```opal
class MathUtils
  def self.max(a, b)
    if a > b then a else b end
  end
end

MathUtils.max(3, 7)  # => 7
```

### Rules
- `def self.name()` defines a class method
- Called on the class: `ClassName.method()`, not on instances
- No access to `.field` (no instance context)
- Inherited — `Dog.species()` works if `Animal` defines `def self.species()`
- Stored separately from instance methods

## Feature 4: `Type(args)` Constructor Shorthand

### Current behavior
- `Point(x: 1, y: 2)` raises "undefined variable 'Point'" because the parser treats it as a function call on an identifier, which looks up `Point` as a variable instead of a class

### Design
When a call expression like `Identifier(args)` is evaluated and `Identifier` resolves to a `Value::Class`, treat it as `.new(args)`.

```opal
class Point
  needs x: Int
  needs y: Int
end

p = Point(x: 1, y: 2)     # sugar for Point.new(x: 1, y: 2)
p = Point.new(x: 1, y: 2) # still works
```

### Rules
- `Type(args)` is sugar for `Type.new(args)` — identical behavior
- Works for classes, models, enums with fields
- Only for PascalCase names that resolve to classes (no ambiguity with functions)
- Works with inheritance: `Dog(name: "Rex", breed: "Lab")`

## Implementation Order

1. `init()` lifecycle — most critical, unblocks super() in constructors
2. `to_string()` — most impactful for usability/debugging
3. `def self.method()` — enables factory patterns
4. `Type(args)` shorthand — ergonomic sugar

## Not In Scope

- `init()` with parameters (use `needs` for field injection)
- `to_string()` format specs (e.g., `f"{obj:.2f}"`)
- Class variables (shared mutable state)
- `protected` visibility
