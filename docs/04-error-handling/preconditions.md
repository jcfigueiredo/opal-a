# Preconditions & Validation

---

## Overview

`requires` validates conditions at the start of a function body. If the condition is false, raises a `PreconditionError`. Validators are regular functions that work in both `requires` clauses and model `where` clauses.

---

## 1. Function Preconditions (`requires`)

```opal
def sqrt(value: Float64) -> Float64
  requires value >= 0, "sqrt requires non-negative input"
  value ** 0.5
end

sqrt(4.0)   # => 2.0
sqrt(-1.0)  # raises PreconditionError: "sqrt requires non-negative input"

# Multiple preconditions
def transfer(from: Account, to: Account, amount: Float64)
  requires amount > 0, "amount must be positive"
  requires from.balance >= amount, "insufficient funds"
  from.withdraw(amount)
  to.deposit(amount)
end
```

---

## 2. Reusable Validators

Validators are regular functions returning `Bool`. They work in both `requires` and model `where` clauses.

```opal
def positive?(value) -> Bool
  value > 0
end

def valid_email?(value) -> Bool
  /^[^@]+@[^@]+\.[^@]+$/.match?(value)
end

# In function preconditions
def sqrt(value: Float64) -> Float64
  requires positive?(value)
  value ** 0.5
end

# Same validators in model fields
model Account
  needs email: String where valid_email?
  needs age: Int32 where |v| v >= 0
  needs deposit: Float64 where positive?
end
```

---

## Rules

- `requires expr` at the start of a function body validates a condition.
- `requires expr, "message"` provides a custom error message.
- If the condition is false, raises `PreconditionError`.
- Multiple `requires` are checked in order.
- Validators are regular functions — reuse them in `requires` and model `where`.
