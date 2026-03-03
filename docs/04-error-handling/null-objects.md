# Null Objects

---

## Overview

Null objects provide default behavior instead of null checks. Opal supports both full-form null object subclasses and a `defaults` shorthand for simple cases.

---

## 1. Full Form

Define a subclass that provides default values and overridden behavior.

```opal
class Person
  needs name: String
  needs age: Int32

  def greet()
    print(f"Hi, I'm {.name}")
  end
end

# Full form — subclass with overridden behavior
class NullPerson < Person
  def init()
    super(name: "anonymous", age: 0)
  end

  def greet()
    print("Hi, I don't want to say my name")
  end
end
```

---

## 2. `defaults` Shorthand

Auto-generates a subclass with default values. All methods delegate to the parent — only construction differs.

```opal
# Shortcut — auto-generates a subclass with default values
class AnonymousPerson < Person defaults {name: "anonymous", age: 0}
# Equivalent to a subclass whose init calls super with these defaults.
# All methods delegate to Person — only construction differs.
```

---

## 3. Usage

```opal
def find_person(id)
  result = database.find(id)
  if result == null
    NullPerson.new()
  else
    result
  end
end

person = find_person(999)
person.greet()  # no null check needed — NullPerson handles it
```
