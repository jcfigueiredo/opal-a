# Opal Syntax Improvement Proposals

> Bold exploration of syntax changes to maximize developer experience.
> Each proposal has: Problem, Current syntax, Proposed syntax, Rationale.

---

## Category A: Missing Language Features

### A1. Pipe Operator `|>`

**Problem:** Opal emphasizes functional patterns (map, filter, reduce) but has no way to express left-to-right data transformation pipelines. Nested function calls read inside-out; method chaining only works when the object owns the method.

**Current — nested calls or chaining:**
```opal
# Nested (reads inside-out)
result = format(validate(parse(read_file("data.csv"))))

# Method chaining works, but only for methods on the object
[1, 2, 3, 4, 5]
  .filter(|x| x > 2)
  .map(|x| x * 2)
```

**Proposed — pipe operator:**
```opal
# Linear, left-to-right
result = read_file("data.csv")
  |> parse
  |> validate
  |> format

# With arguments (piped value becomes first arg)
users
  |> filter(|u| u.active?())
  |> sort_by(|u| u.name)
  |> take(10)
  |> format_table

# Mix with method chaining
raw_data
  |> CSV.parse
  |> .filter(|row| row["age"].to_int() > 18)
  |> .map(|row| row["name"])
  |> .join(", ")
```

**Rationale:** Elixir, F#, Elm, OCaml, and R all have this. It's the single biggest DX improvement for functional-style code. Aligns with "readability is paramount."

**BNF addition:**
```bnf
<expression> ::= ... | <expression> "|>" <expression>
```

---

### A2. `elsif` / `elif` for Chained Conditionals

**Problem:** The current BNF only allows `if/else/end`. Chained conditions require deeply nested if/else blocks. This is a *critical* gap.

**Current — nested if/else (the only option):**
```opal
if status == :ok
  handle_success()
else
  if status == :redirect
    follow_redirect()
  else
    if status == :error
      handle_error()
    else
      unknown()
    end
  end
end
```

**Proposed — add `elsif`:**
```opal
if status == :ok
  handle_success()
elsif status == :redirect
  follow_redirect()
elsif status == :error
  handle_error()
else
  unknown()
end
```

**Rationale:** Every block-based language has this (Ruby: `elsif`, Python: `elif`, Elixir: `else if` inline). The current omission forces either deeply nested code or using `match` for simple conditionals, which is overkill.

**Recommendation:** Use `elsif` (Ruby convention, consistent with Opal's Ruby-influenced syntax).

---

### A3. Null-Safe Chaining `?.`

**Problem:** Accessing nested nullable values requires verbose `if`/`is` checks or pattern matching. Modern languages (Kotlin, Swift, TypeScript, C#) all provide null-safe chaining.

**Current:**
```opal
# Must check at each level
if user is User
  if user.address is Address
    if user.address.city is String
      print(user.address.city)
    end
  end
end

# Or with match
match user?.address?.city
  # wait — this syntax doesn't exist
```

**Proposed — `?.` operator:**
```opal
# Returns null if any part of the chain is null
city = user?.address?.city

# With method calls
length = user?.name?.upper()?.length

# Combine with null coalescing (??)
city = user?.address?.city ?? "Unknown"

# In conditions
if user?.subscription?.active?()
  grant_access()
end
```

**Also add null coalescing `??`:**
```opal
name = user?.name ?? "anonymous"
port = config["port"] ?? 8080
```

**Rationale:** This is the #1 most-requested feature in every language that lacks it. Opal already has `T?` nullable types — this completes the ergonomic story.

**BNF additions:**
```bnf
<expression> ::= ...
    | <expression> "?." IDENTIFIER
    | <expression> "?." IDENTIFIER "(" <args> ")"
    | <expression> "??" <expression>
```

---

### A4. List and Dict Comprehensions

**Problem:** Creating derived collections requires `.map()` and `.filter()` chains, which can be verbose for simple transformations. Comprehensions are a well-established alternative.

**Current:**
```opal
# Filter and transform
squares = (1..10).filter(|x| x % 2 == 0).map(|x| x ** 2)

# Nested iteration — awkward with chaining
pairs = []
for x in 1..3
  for y in 1..3
    if x != y
      pairs.push((x, y))
    end
  end
end
```

**Proposed — comprehensions:**
```opal
# List comprehension
squares = [x ** 2 for x in 1..10 if x % 2 == 0]

# Dict comprehension
name_lengths = {name: name.length for name in names}

# Nested
pairs = [(x, y) for x in 1..3 for y in 1..3 if x != y]

# With destructuring
adults = [name for (name, age) in people if age >= 18]
```

**Rationale:** Python, Haskell, Elixir, Scala, and Kotlin all have them. They're more readable than `.filter().map()` for simple cases, while the method chain style remains available for complex transformations.

**Design decision:** Comprehensions are sugar for `filter` + `map`. They don't replace the method-chain style — both coexist ("one way" applies to semantics, not spelling).

---

### A5. Iterator Protocol: Use `Option(T)` Instead of `(value, done)`

**Problem:** The current iterator protocol returns `(value, done::Bool)`, which is the JavaScript pattern. This is clunky — what's the `value` when `done` is true? The idiomatic Opal way would be to use the language's own `Option(T)` type.

**Current:**
```opal
protocol Iterator
  def next() -> (value, done::Bool)
end

# Implementing
def next()
  if .has_more
    (compute_next(), false)
  else
    (null, true)  # what type is null here?
  end
end
```

**Proposed:**
```opal
protocol Iterator(T)
  def next() -> Option(T)
end

# Implementing
def next() -> Option(String)
  line = .file.read_line()
  if line == null
    Option.None
  else
    Option.Some(value: line)
  end
end
```

**Rationale:** Rust, Kotlin, Swift, and Scala all use `Option`/`Optional` for iterators. It's type-safe, avoids the "what value when done?" problem, and dogfoods Opal's own `Option(T)` enum.

---

## Category B: Syntax Simplifications

### B1. Drop `unless` and `until`

**Problem:** Negated control flow hurts readability. `unless` is a double negative waiting to happen. `until` is rarely clearer than `while not`. This contradicts "readability is paramount."

**Current — confusing negation:**
```opal
unless a != b     # double negative! means: if a == b
  c = 1
end

until count >= 10  # means: while count < 10
  count += 1
end

# Suffix form is even worse
print("odd") unless n % 2 == 0
```

**Proposed — remove both, use `if not` / `while not`:**
```opal
if a == b
  c = 1
end

while count < 10
  count += 1
end

print("odd") if n % 2 != 0
```

**Rationale:** Ruby's `unless` is widely considered a mistake — the Ruby style guide itself says "don't use unless with else." Python deliberately lacks `unless`. Swift lacks it. Removing it aligns with "one explicit way."

If you want to keep suffix-`if`, that's fine — it's clear and useful:
```opal
return early_result if cache.has?(key)
```

---

### B2. Constructor Sugar: `Type(args)` as Shorthand for `Type.new(args)`

**Problem:** `.new()` is explicit but verbose, especially for data types and nested construction.

**Current:**
```opal
point = Point.new(x: 1.0, y: 2.0)
circle = Shape.Circle(radius: 5.0)  # enums don't use .new()

# Nested construction is painful
user = User.new(
  name: "claudio",
  address: Address.new(
    street: "123 Main",
    city: City.new(name: "Springfield", state: State.new(code: "IL"))
  )
)
```

**Proposed — callable classes:**
```opal
# Type(...) is sugar for Type.new(...)
point = Point(x: 1.0, y: 2.0)

# Enum construction stays the same (already concise)
circle = Shape.Circle(radius: 5.0)

# Nested becomes much cleaner
user = User(
  name: "claudio",
  address: Address(
    street: "123 Main",
    city: City(name: "Springfield", state: State(code: "IL"))
  )
)

# .new() still works — sugar, not replacement
verbose = Point.new(x: 1.0, y: 2.0)
```

**Rationale:** Python, Kotlin, Scala, Swift all use this. It's the most natural syntax — "create a Point with these values." `.new()` remains available for cases where explicitness helps (e.g., factory methods).

**Potential issue:** Could conflict with function calls if class names and function names collide. But Opal already enforces PascalCase for classes and snake_case for functions, so there's no ambiguity.

---

### B3. Simplify Error Handling Keywords

**Problem:** `fail` / `on fail` / `ensure` are non-standard and create a learning barrier. Every developer already knows `raise`/`throw` + `catch` + `finally`.

**Current:**
```opal
try
  config = read_config("missing.json")
on fail FileNotFound as e
  print(f"Missing: {e.path}")
on fail as e
  log(f"Unexpected: {e.message}")
  fail(e)  # re-raise
ensure
  cleanup()
end
```

**Option 1 — Use `raise`/`catch`/`finally` (familiar):**
```opal
try
  config = read_config("missing.json")
catch FileNotFound as e
  print(f"Missing: {e.path}")
catch as e
  log(f"Unexpected: {e.message}")
  raise(e)
finally
  cleanup()
end
```

**Option 2 — Use `raise`/`rescue`/`ensure` (Ruby-aligned):**
```opal
try
  config = read_config("missing.json")
rescue FileNotFound as e
  print(f"Missing: {e.path}")
rescue as e
  log(f"Unexpected: {e.message}")
  raise(e)
ensure
  cleanup()
end
```

**Option 3 — Keep `fail` but fix `on fail`:**
```opal
try
  config = read_config("missing.json")
catch FileNotFound as e    # just replace "on fail" with "catch"
  print(f"Missing: {e.path}")
catch as e
  log(f"Unexpected: {e.message}")
  fail(e)
ensure                      # keep ensure
  cleanup()
end
```

**Recommendation:** Option 3. `fail` is distinctive and reads well as a verb ("fail with this error"). But `on fail` is grammatically awkward and unfamiliar — `catch` is universally understood. `ensure` is fine (Ruby precedent).

---

### B4. Simplify No-Arg Closures

**Problem:** `|| expr` looks like a logical OR operator. No-arg closures should be more readable.

**Current:**
```opal
counter = 0
increment = || counter += 1

# Multi-line
setup = ||
  load_config()
  init_db()
end
```

**Proposed — use `do` blocks for no-arg closures:**
```opal
counter = 0
increment = do counter += 1 end

# Multi-line — natural
setup = do
  load_config()
  init_db()
end

# Keep |args| for closures with arguments
double = |x| x * 2
transform = |items, fn|
  items.map(fn)
end
```

**Alternative — arrow syntax `=> expr`:**
```opal
increment = => counter += 1
double = |x| => x * 2   # nah, this is worse
```

**Recommendation:** The `do ... end` approach for no-arg closures. It's already used in blocks (`on EventType do |e| ... end`) so it's consistent.

---

### B5. Streamline `def :init` to `def init`

**Problem:** Using a symbol for the constructor name (`:init`) is clever but inconsistent — no other method uses this convention. It adds cognitive load for no practical benefit.

**Current:**
```opal
class Person
  def :init(name, age)
    .name = name
    .age = age
  end
end
```

**Proposed:**
```opal
class Person
  def init(name, age)
    .name = name
    .age = age
  end
end
```

**Rationale:** `:init` was chosen to distinguish the constructor from regular methods, but `init` already reads differently by convention (it's a well-known name). Python uses `__init__`, Ruby uses `initialize`, Opal can just use `init`. The `:` prefix adds no information and makes it look like an actor message.

---

## Category C: Consistency Fixes

### C1. Unify Enum Construction and Destructuring

**Problem:** Enum variants are constructed with named args but destructured positionally. This is inconsistent.

**Current — inconsistent:**
```opal
# Construction: named args
s = Shape.Circle(radius: 5.0)
r = Response.Success(body: "hello", headers: {:})

# Destructuring: positional!
match s
  case Shape.Circle(r)          # positional — r binds to radius
    Math.PI * r ** 2
  case Shape.Rectangle(w, h)    # positional — w, h bind by position
    w * h
end
```

**Proposed — allow both, favor positional for simple cases:**
```opal
# Construction: positional OR named
s = Shape.Circle(5.0)                    # positional (ok for 1 field)
s = Shape.Circle(radius: 5.0)            # named (explicit)
r = Response.Success("hello", {:})       # positional
r = Response.Success(body: "hello", headers: {:})  # named

# Destructuring: same as today (positional) — consistent with construction now
match s
  case Shape.Circle(r)
    Math.PI * r ** 2
end
```

**Rationale:** Making construction support positional args removes the inconsistency. Named args remain available for multi-field variants where position is ambiguous.

---

### C2. Reduce `as` Keyword Overload

**Problem:** `as` currently means 5 different things:

1. Type cast: `x as Int32`
2. Pattern binding: `case Circle(r) as shape`
3. Import alias: `import Math as M`
4. Settings marker: `model X as Settings`
5. Null object marker: `class X as Y defaults {...}`

**Proposed — keep `as` for 1-3, use different syntax for 4-5:**

| Current | Proposed | Rationale |
|---------|----------|-----------|
| `x as Int32` | `x as Int32` | Keep — universal meaning |
| `case Circle(r) as shape` | `case Circle(r) as shape` | Keep — pattern languages use this |
| `import Math as M` | `import Math as M` | Keep — universal meaning |
| `model X as Settings` | `model X is Settings` or `settings model X` | Different concept (is-a, not aliasing) |
| `class X as Y defaults {...}` | `class X < Y defaults {...}` | Already uses `<` for inheritance |

**Proposed examples:**
```opal
# Settings — use modifier keyword
settings model AppSettings
  needs debug::Bool = false
  needs secret_key::String
end

# Or: keep `as` but change null objects:
# Null objects — use inheritance syntax
class AnonymousPerson < Person defaults {name: "anonymous", age: 0}
```

---

### C3. Unify Guard and Validation Syntax

**Problem:** Guards (`guard` keyword + `@decorator`) and model validation (`where` clause) are separate systems for the same concept: "validate this value."

**Current — two systems:**
```opal
# System 1: Guards (for functions)
guard positive(value) fails :must_be_positive
  return value > 0
end

@positive
def sqrt(value::Float64) -> Float64
  value ** 0.5
end

# System 2: Where clauses (for models)
model Account
  needs age::Int32 where |v| v >= 0
  needs deposit::Float64 where positive   # reuses guard, but different syntax
end
```

**Proposed — unify around `where`:**
```opal
# Validators are just named functions returning Bool
def positive?(value) -> Bool
  value > 0
end

# On functions — where clause after params
def sqrt(value::Float64) -> Float64 where value: positive?
  value ** 0.5
end

# On models — same syntax
model Account
  needs age::Int32 where |v| v >= 0
  needs deposit::Float64 where positive?
end
```

**Alternative — keep guards but simplify:**
```opal
# Drop the separate `guard` keyword — a guard is just a function
def positive?(value) -> Bool
  value > 0
end

# Use `requires` for function preconditions (not @decorator)
def sqrt(value::Float64) -> Float64
  requires positive?(value)
  value ** 0.5
end
```

**Recommendation:** The `requires` approach. It's explicit ("requires this condition"), doesn't overload `@` (which is for macros), and reads naturally.

---

### C4. Clarify `@` — Macros Only, Not Guards

**Problem:** `@` is used for both macro invocation and guard decorators. This ambiguity is noted in the spec: "Guards are resolved first; if no guard matches, it's treated as a macro."

**Proposed:** Reserve `@` exclusively for macros. Move guard/precondition checking to `requires` (see C3).

```opal
# @ = always a macro
@memoize
@json_serializable
@test "addition" do ... end
@describe "Math" do ... end

# Preconditions = requires (never @)
def sqrt(value::Float64) -> Float64
  requires value > 0
  value ** 0.5
end

def register_voter(name::String, age::Int32)
  requires age >= 18, f"Must be 18+, got {age}"
  print(f"{name} registered")
end
```

**Rationale:** `@` having a single meaning makes the language more predictable. "Is `@positive` a macro or a guard?" is a question that should never need answering.

---

## Category D: Bold Redesigns

### D1. Rethink the Closure / Block Passing Model

**Problem:** Opal closures use Ruby's `|params| body` syntax, but the language lacks Ruby's block passing mechanism. This makes higher-order patterns verbose.

**Current:**
```opal
# Inline closure — fine
numbers.map(|x| x * 2)

# Multi-line closure — heavy
numbers.reduce(0, |acc, x|
  result = complex_operation(x)
  acc + result
end)

# Passing closures as last arg — no special syntax
File.open("data.txt", |f|
  f.read()
end)
```

**Proposed — trailing block syntax:**
```opal
# When the last argument is a closure, it can trail the call
numbers.map |x| x * 2

# Multi-line trailing block
numbers.reduce(0) do |acc, x|
  result = complex_operation(x)
  acc + result
end

# Resource management pattern
File.open("data.txt") do |f|
  f.read()
end

# Combine with pipe operator
urls
  |> parallel_map do |url|
    Net.fetch(url)
  end
  |> filter |resp| resp.ok?()
  |> map |resp| resp.body
```

**Rationale:** Trailing blocks are one of Ruby's best features. They make DSLs (testing, routing, resource management) read naturally. The `do |params| ... end` form is already used in event handlers (`on Event do |e| ... end`), so this extends existing syntax.

**Rule:** If the last parameter is a closure, the closure can be written after the closing `)` as a trailing block.

---

### D2. Consider `fn` Keyword for Named Closures

**Problem:** Assigning closures to variables looks different from defining functions:
- `def greet(name) ... end` for functions
- `greet = |name| ...` for closures

This dual syntax is fine but could be more unified.

**Proposed — `fn` for anonymous/inline functions:**
```opal
# `fn` creates a callable value (like |params| but more readable)
double = fn(x) x * 2 end
greet = fn(name) f"Hello, {name}" end

# Multi-line
transform = fn(data, config)
  validated = validate(data)
  process(validated, config)
end

# |params| syntax remains for inline/short closures
numbers.map(|x| x * 2)
numbers.filter(|x| x > 0)

# fn shines for complex closures and closures stored in variables
handler = fn(request, response)
  user = authenticate(request)
  data = process(request.body)
  response.json(data)
end
```

**Rationale:** `fn` is used by Elixir, Erlang, and is proposed in many languages. It's explicit ("this is a function value") and reads better than `|params|` for multi-line closures.

**However:** This adds a second way to write closures. If Opal's "one way" principle is strict, keep only `|params|`. If the principle is "one way per use case," then `fn` for stored closures and `|params|` for inline makes sense.

---

### D3. Expression-Oriented `if` Without `end`

**Problem:** Simple conditional expressions require `if/then/else/end`, which is verbose for one-liners.

**Current:**
```opal
status = if active then "on" else "off" end
```

**Proposed — keep this, but also allow `if/else` as expression without end in certain contexts:**

Actually, the current syntax is fine. The `then` keyword makes inline-if readable. No change needed here.

**What IS needed:** Ensure `if/elsif/else/end` works for multi-branch (see A2).

---

### D4. Rethink `receive` in Actors — Pattern Matching on Messages

**Problem:** Actor `receive` blocks use symbol-based message names with a mini-pattern-matching system. But Opal already has a powerful `match` expression. Why have two pattern matching systems?

**Current:**
```opal
actor Cache
  receive :get(key)
    reply .store[key]
  end

  receive :set(key, value)
    .store[key] = value
    reply :ok
  end
end

cache.send(:set, "user:1", "claudio")
```

**Proposed — use `receive/match` with typed messages:**
```opal
# Define message types
enum CacheMsg
  Get(key::String)
  Set(key::String, value::String)
  Clear
end

actor Cache
  receive msg::CacheMsg
    match msg
      case CacheMsg.Get(key)
        reply .store[key]
      case CacheMsg.Set(key, value)
        .store[key] = value
        reply :ok
      case CacheMsg.Clear
        .store = {:}
        reply :ok
    end
  end
end

cache.send(CacheMsg.Set("user:1", "claudio"))
cache.send(CacheMsg.Get("user:1"))
```

**Alternative — keep symbols but add pattern matching inside receive:**
```opal
actor Cache
  receive
    case :get(key)
      reply .store[key]
    case :set(key, value)
      .store[key] = value
      reply :ok
    case :clear
      .store = {:}
      reply :ok
  end
end
```

**Recommendation:** The alternative (second option). It unifies `receive` with `match`/`case` syntax, keeping the symbol-based message convenience while allowing pattern matching. Simpler than typed messages for small actors, but compatible with typed messages when needed.

---

### D5. Introduce `let` for Immutable Bindings

**Problem:** All variables are mutable by default. There's no way to mark a binding as immutable. This hurts code reasoning — you can never be sure a variable won't be reassigned later.

**Current:**
```opal
name = "claudio"   # mutable
name = "different"  # allowed
PI = 3.14159        # SCREAMING_CASE convention says "constant" but not enforced
```

**Proposed:**
```opal
let name = "claudio"    # immutable binding
name = "different"      # COMPILE ERROR — reassignment of `let` binding

pi = 3.14               # mutable (default, backward compatible)

# let in destructuring
let (x, y) = get_point()
x = 0  # COMPILE ERROR

# Function params are implicitly let
def greet(name::String)
  name = "override"  # COMPILE ERROR — params are immutable
end
```

**Rationale:** Rust, Swift, Kotlin, and JavaScript (`const`) all distinguish mutable from immutable bindings. This is a powerful tool for code correctness without any runtime cost.

**Design consideration:** Should `let` be the default and `mut`/`var` be the opt-in? That would be Rust's approach. But it breaks backward compatibility with every example in the spec.

---

## Category E: Cosmetic / Ergonomic Tweaks

### E1. Multiline Comment Syntax

**Current:** `#{ }#` — unusual, hard to remember which side gets the `#`.

**Options:**
```opal
# Option A: Keep #{ }# — it's consistent with # for comments
#{
  This works.
}#

# Option B: Use ###
###
  More symmetrical, easier to type.
  Triple # is analogous to triple-quote strings.
###

# Option C: Use /* */
/*
  Universal. Everyone knows it.
  But conflicts with the "not C-family" aesthetic.
*/
```

**Recommendation:** Option B (`###`). It's consistent with the `#` comment character, symmetrical, easy to type, and visually distinct from single-line comments.

---

### E2. Add `in` Operator for Membership Testing

**Current:**
```opal
if list.contains?("item")
  # ...
end

if dict.has?("key")
  # ...
end
```

**Proposed — `in` operator:**
```opal
if "item" in list
  # ...
end

if "key" in dict
  # ...
end

if char in 'a'..'z'
  # ...
end

# Already used in for loops!
for item in list
  # ...
end
```

**Rationale:** `in` is already a keyword (used in `for...in`). Extending it to membership testing is natural and reads like English. Python, Kotlin, and Swift do this.

---

### E3. String Repeat Operator

**Missing entirely from spec.**

```opal
"ha" * 3      # => "hahaha"
"-" * 40      # => "----------------------------------------"
```

**Rationale:** Nearly every language supports this. Useful for formatting, padding, and separators.

---

### E4. Ternary/Conditional Expression Cleanup

**Current:**
```opal
status = if active then "on" else "off" end
```

**This is fine.** But consider also supporting:
```opal
# When-style for single conditions (sugar)
status = "on" if active else "off"
```

**Recommendation:** Keep the current `if/then/else/end` as the only form. It's explicit and readable. Adding alternatives would violate "one explicit way."

---

## Summary: Recommended Changes by Priority

### Must Have (High impact, clear improvement)
| # | Change | Impact |
|---|--------|--------|
| A2 | Add `elsif` | Fixes a critical gap in control flow |
| A3 | Null-safe chaining `?.` and `??` | Essential for nullable types ergonomics |
| A1 | Pipe operator `|>` | Major DX improvement for functional style |
| B3 | `catch` instead of `on fail` | Reduces learning barrier |
| C4 | `@` for macros only (not guards) | Eliminates ambiguity |

### Should Have (Strong improvement, some design work needed)
| # | Change | Impact |
|---|--------|--------|
| B2 | Constructor sugar `Type(args)` | Cleaner object creation |
| C1 | Positional enum construction | Consistency with destructuring |
| A4 | List/dict comprehensions | Readable collection transforms |
| B1 | Drop `unless`/`until` | Simplifies language, removes foot-guns |
| D1 | Trailing block syntax | DSL ergonomics |

### Worth Considering (Bold but debatable)
| # | Change | Impact |
|---|--------|--------|
| A5 | Iterator with `Option(T)` | Type-safe, dogfoods own types |
| B5 | `def init` not `def :init` | Simplification |
| C3 | Unify guards + `requires` keyword | Conceptual simplification |
| D5 | `let` for immutable bindings | Correctness |
| D4 | Pattern-matching `receive` | Unifies actor messages with match |
| E1 | `###` multiline comments | Minor polish |
| E2 | `in` operator | Natural membership test |
| E3 | String repeat `*` | Missing basic feature |

### Probably Skip
| # | Change | Reason |
|---|--------|--------|
| D2 | `fn` keyword | Adds a second way — conflicts with "one way" principle |
| D3 | Expression `if` without `end` | Current syntax is already clean |
| C2 | Reduce `as` overload | Low impact, high disruption |
| B4 | `do` for no-arg closures | Minor issue, many alternatives |
