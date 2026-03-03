# Specifications

The specification pattern allows composable business rules. Each specification is a class that implements `is_satisfied_by`, and specifications can be composed with logical operators.

---

## The Pattern

```opal
import Spec.Specification

class Person
  needs name::String
  needs age::Int32
  needs place_of_birth::String
end

class OverAgeSpec < Specification
  def is_satisfied_by(person::Person) -> Bool
    person.age >= 21
  end
end

class BornAtSpec < Specification
  needs born_at::String

  def is_satisfied_by(person::Person) -> Bool
    person.place_of_birth == .born_at
  end
end

claudio = Person.new(name: "claudio", age: 15, place_of_birth: "CA")
andrea = Person.new(name: "andrea", age: 21, place_of_birth: "CT")
people = [claudio, andrea]

over_age = OverAgeSpec.new()
over_age_people = people.filter(|p| over_age.is_satisfied_by(p))  # => [andrea]

californian = BornAtSpec.new(born_at: "CA")

# Logically combining business rules
spec = over_age.not().and(californian)
some_people = people.filter(|p| spec.is_satisfied_by(p))  # => [claudio]
```

---

## How It Works

- Each specification is a class that extends `Specification` from the `Spec` stdlib module.
- Specifications implement `is_satisfied_by(entity) -> Bool` to encode a single business rule.
- Specifications can use `needs` for parameterization (e.g., `BornAtSpec` needs a `born_at` string).
- The base `Specification` class provides `.and()`, `.or()`, and `.not()` combinators for composing rules.
- Composed specifications return a new specification that can be passed around, stored, or further composed.

---

## Summary

The specification pattern provides a clean way to define, compose, and reuse business rules. Each rule is a small class with a single `is_satisfied_by` method, and rules combine with `.and()`, `.or()`, and `.not()` to build complex predicates from simple parts.
