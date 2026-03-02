# Variables & Assignment

---

## Overview

Variables in Opal are dynamically typed and need no declaration keyword. Opal supports parallel assignment, immutable bindings via `let`, and Unicode identifiers. Naming conventions follow a consistent pattern across the language.

---

## 1. Variables & Assignment

Variables are dynamically typed and need no declaration keyword. Unicode identifiers are supported and encouraged.

```opal
pi = 3.14
𝛑 = 3.14
alpha = 1

# Parallel assignment
x, y = 1, 2

# Swap
x, y = y, x
```

## 2. Immutable Bindings

`let` creates an immutable binding — the variable cannot be reassigned after initialization.

```opal
let name = "claudio"
name = "different"     # COMPILE ERROR — reassignment of let binding

# Mutable (default) — no keyword needed
counter = 0
counter += 1           # ok

# let with destructuring
let (x, y) = get_point()
x = 0                  # COMPILE ERROR

# let with type annotation
let pi::Float64 = 3.14159
```

**Rules:**
- `let x = expr` creates an immutable binding.
- `x = expr` (without `let`) creates a mutable binding — backward compatible.
- Function parameters are implicitly immutable.
- `let` works with destructuring and type annotations.
- Reassigning a `let` binding is a compile-time error.

## 3. Naming Conventions

Variable naming conventions:
- `snake_case` for local variables and functions
- `PascalCase` for classes, modules, and actors
- `SCREAMING_SNAKE` for constants
- `.name` for instance variables (inside classes)
- `:name` for symbols
- `let` for values that shouldn't change after assignment
