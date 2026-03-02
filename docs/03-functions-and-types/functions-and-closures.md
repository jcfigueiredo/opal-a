# Functions & Closures

---

## Overview

Functions are defined with `def` and are first-class values in Opal. Closures capture their enclosing scope by reference and come in three syntactic forms: `|params| expr` for inline use, `fn(params) ... end` for stored function values, and `do ... end` for no-arg closures and trailing blocks.

---

## 1. Function Definitions

Functions are defined with `def`. They are first-class values.

```opal
# Basic function
def greet(name)
  print(f"Hello, {name}!")
end

# With type annotations
def add(a::Int32, b::Int32) -> Int32
  a + b
end

# Default arguments
def connect(host, port = 8080)
  # ...
end

# Named arguments at call site
connect(host: "localhost", port: 3000)

# Last expression is the return value (explicit return also works)
def square(x)
  x * x
end
```

---

## 2. Closures / Lambdas

Closures use the `|params| body` syntax.

```opal
double = |x| x * 2
apply = |fn, value| fn(value)
apply(double, 5)  # => 10

# Multi-line closure
transform = |items, fn|
  items.map(fn)
end

# Closures capture their enclosing scope
multiplier = 3
triple = |x| x * multiplier
triple(10)  # => 30
```

---

## 3. Capture Semantics

Closures capture variables **by reference** -- they see the live variable, not a snapshot. Mutations inside the closure affect the enclosing scope and vice versa.

```opal
counter = 0
increment = do counter += 1 end

increment()     # counter is now 1
increment()     # counter is now 2
print(counter)  # => 2

# Multi-line no-arg closure
setup = do
  load_config()
  init_db()
end

# Closures see the live variable
x = 10
show_x = do print(x) end
x = 20
show_x()  # => 20 (not 10)
```

---

## 4. Closure Types

Closures can be typed using the `|ParamTypes| -> ReturnType` syntax:

```opal
# Closure type annotation
transform::|Int32| -> Int32 = |x| x * 2

# As a function parameter type
def apply(fn::|Int32| -> Int32, value::Int32) -> Int32
  fn(value)
end

apply(|x| x + 1, 5)  # => 6
```

---

## 5. Trailing Blocks

When the last argument to a function is a closure, it can be written as a trailing `do...end` block after the call.

```opal
# These are equivalent
numbers.each(|x| print(x))
numbers.each do |x| print(x) end

# Trailing blocks shine for multi-line closures
numbers.reduce(0) do |acc, x|
  result = complex_operation(x)
  acc + result
end

# Resource management
File.open("data.txt") do |f|
  data = f.read()
  process(data)
end

# Already used in event handlers -- this formalizes the pattern
on OrderPlaced do |e|
  .mailer.send_confirmation(e.order)
end
```

**Rules:**
- The trailing block becomes the last argument to the function call.
- `f(a, b) do |x| ... end` is equivalent to `f(a, b, |x| ... end)`.
- Only one trailing block per call.

---

## 6. Named Closures with `fn`

The `fn` keyword creates a function value. Use it when assigning closures to variables or passing complex multi-line functions.

```opal
# fn for stored function values
double = fn(x) x * 2 end
greet = fn(name) f"Hello, {name}" end

# Multi-line
handler = fn(request, response)
  user = authenticate(request)
  data = process(request.body)
  response.json(data)
end

# |params| remains preferred for inline/short closures
numbers.map(|x| x * 2)
numbers.filter(|x| x > 0)
```

**When to use which:**
- `|params| expr` -- inline closures passed directly to functions.
- `fn(params) ... end` -- closures stored in variables or with multi-line bodies.
- `do ... end` -- no-arg closures or trailing blocks.
- All create the same type of value -- the choice is stylistic.
