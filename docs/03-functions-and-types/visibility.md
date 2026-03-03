# Visibility / Access Control

---

## Overview

Opal defaults to public visibility for all definitions. The `private` and `protected` modifiers restrict access to class methods, module-level definitions, and other constructs. Visibility is explicit -- mark what you want to hide.

---

## 1. Class-Level Visibility

```opal
class Account
  def init(owner, balance)
    .owner = owner
    .balance = balance
  end

  public def balance()
    .balance
  end

  public def deposit(amount)
    .balance += amount
  end

  private def calculate_interest()
    .balance * 0.05
  end

  protected def transfer_to(other: Account, amount)
    .balance -= amount
    other.deposit(amount)
  end
end

acct = Account.new(owner: "alice", balance: 1000)
acct.balance()              # => 1000
acct.calculate_interest()   # Error: private method called
```

Default visibility is `public`. Mark methods `private` (accessible only within the class) or `protected` (accessible within the class and subclasses).

---

## 2. Module-Level Visibility

```opal
# file: src/math.opl

# Public (default) -- importable
def abs(x: Number)
  if x < 0 then -x else x end
end

# Private -- only usable within this module
private def validate_input(x)
  x != null
end

# Private class -- implementation detail
private class InternalCache
  # ...
end
```

```opal
import Math
Math.abs(-5)              # ok
Math.validate_input(3)    # COMPILE ERROR -- private
```

---

## 3. Visibility Rules

- `public` (default) -- accessible from anywhere. All top-level definitions are public unless marked otherwise.
- `private` -- accessible only within the defining class or module.
- `protected` -- accessible within the class and its subclasses (classes only, not applicable to modules).
- Applies to: methods, functions, classes, enums, models, constants.
- `needs` fields are public by default -- they define the class's interface.
- Default visibility is `public` for all constructs. Mark `private` or `protected` explicitly.

---

## 4. Visibility Summary

| Modifier | Class methods | Module definitions | `needs` fields |
|---|---|---|---|
| `public` (default) | Accessible everywhere | Importable | Readable |
| `private` | Same class only | Same module only | N/A (needs are always public) |
| `protected` | Class + subclasses | N/A | N/A |
