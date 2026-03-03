# Multiple Dispatch

---

## Overview

Functions can have multiple definitions that dispatch based on argument types, arity, and guards. The compiler selects the most specific matching definition at call time, with ambiguity treated as a compile-time error.

---

## 1. Dispatch by Type and Arity

```opal
class Renderer
  # Dispatch by type
  def render(shape: Circle)
    draw_circle(shape.center, shape.radius)
  end

  def render(shape: Rectangle)
    draw_rect(shape.origin, shape.width, shape.height)
  end

  # Dispatch by arity
  def render(shape: Circle, color: Color)
    set_color(color)
    draw_circle(shape.center, shape.radius)
  end
end
```

---

## 2. Dispatch with Preconditions

A `requires` clause narrows the valid inputs for a particular overload. When the precondition fails, dispatch falls through to the next matching definition.

```opal
def process(value: Int32)
  print("generic integer")
end

def process(value: Int32)
  requires value > 0
  print("positive integer")
end

process(5)   # => "positive integer" (requires passes)
process(-3)  # => "generic integer"  (requires fails, falls to base)
```

---

## 3. Resolution Order

1. **Exact type match** -- argument types match a definition exactly.
2. **Precondition-constrained match** -- a `requires` narrows the valid inputs.
3. **Signature arity match** -- number of arguments selects among overloads.
4. **Ambiguity = compile-time error** -- if two definitions match equally well, the compiler rejects the program.

---

## Summary

| Feature | Decision |
|---|---|
| Type dispatch | Multiple `def` with different parameter types |
| Arity dispatch | Multiple `def` with different parameter counts |
| Precondition dispatch | `requires` clause narrows valid inputs, falls through on failure |
| Resolution order | Exact type > precondition > arity > ambiguity error |
| Ambiguity | Compile-time error if two definitions match equally well |
