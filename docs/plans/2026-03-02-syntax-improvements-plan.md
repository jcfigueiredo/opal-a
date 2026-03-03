# Syntax Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate all accepted syntax proposals from `docs/plans/2026-03-02-syntax-proposals.md` into the Opal language specification.

**Architecture:** This is a spec-editing project — no code to compile or test. Each task modifies `Opal.md` (BNF grammar, prose sections, and code examples) and occasionally feature docs in `docs/features/`. Tasks are grouped so that related changes happen together, minimizing spec passes. After each task, the BNF, prose, and examples must be internally consistent.

**Tech Stack:** Markdown editing of the Opal language specification.

**Key files:**
- `Opal.md` — The master spec (~4150 lines). Sections: BNF (3), Basics (4), Control Flow (5), Functions & Types (6), Error Handling (7), Concurrency (8), Patterns (9), Metaprogramming (10), Stdlib (11), Tooling (12), Pretotyping (13).
- `docs/features/*.md` — 11 feature documents with extended rationale and examples.
- `docs/plans/2026-03-02-syntax-proposals.md` — The proposals document (reference, don't modify).

**Cross-reference checklist for every task:**
After each change, verify consistency across:
1. BNF grammar (section 3, lines 38-251)
2. The relevant prose section
3. All code examples in that section
4. The stdlib table (section 11, lines 3765-3812)
5. The philosophy line (section 1, line 13) if adding a new first-class concept
6. Any feature doc that covers the same topic

---

## Task 1: Add `elsif` to conditionals

**Proposal:** A2 — Add `elsif` for chained conditionals.

**Files:**
- Modify: `Opal.md` — BNF `<conditional>` rule (line ~127), section 5.1 Conditionals (lines 873-898)

**Step 1: Update BNF**

In section 3, change the `<conditional>` rule from:
```bnf
<conditional>   ::= "if" <expression> NEWLINE <block> ("else" NEWLINE <block>)? "end"
                   | "unless" <expression> NEWLINE <block> ("else" NEWLINE <block>)? "end"
```
to:
```bnf
<conditional>   ::= "if" <expression> NEWLINE <block>
                     ("elsif" <expression> NEWLINE <block>)*
                     ("else" NEWLINE <block>)? "end"
                   | "unless" <expression> NEWLINE <block> ("else" NEWLINE <block>)? "end"
```

(Note: `unless` will be removed in Task 3, but keep it here for now to isolate changes.)

**Step 2: Update section 5.1 with `elsif` examples**

Add after the existing if/else example:
```opal
# elsif for chained conditions
if status == :ok
  handle_success()
elsif status == :redirect
  follow_redirect()
elsif status == :error
  handle_error()
else
  unknown()
end

# Expression form works too
label = if score >= 90
  "A"
elsif score >= 80
  "B"
elsif score >= 70
  "C"
else
  "F"
end
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add elsif for chained conditionals (proposal A2)"
```

---

## Task 2: Add pipe operator `|>`

**Proposal:** A1 — Pipe operator for left-to-right data flow.

**Files:**
- Modify: `Opal.md` — BNF `<expression>` and `<binary_op>` rules (lines ~69, ~246), section 4.4 Operators (lines 549-670), new subsection after 4.4

**Step 1: Update BNF**

Add `"|>"` to `<binary_op>`:
```bnf
<binary_op>     ::= "+" | "-" | "*" | "/" | "%" | "**"
                   | "==" | "!=" | "<" | ">" | "<=" | ">="
                   | "and" | "or"
                   | ".." | "..."
                   | "|>"
                   | "?." | "??"
```

(Adding `?.` and `??` here too for Task 4 — they're in the same BNF rule.)

**Step 2: Add pipe operator section after Operators**

After the operator tables and before section 4.5, add a new subsection:

```markdown
#### Pipe Operator

The pipe operator `|>` passes the result of the left expression as the first argument to the right expression. It enables left-to-right data transformation pipelines.

\```opal
# Without pipe (reads inside-out)
result = format(validate(parse(read_file("data.csv"))))

# With pipe (reads left-to-right)
result = read_file("data.csv")
  |> parse
  |> validate
  |> format

# Piped value becomes the first argument
users
  |> filter(|u| u.active?())
  |> sort_by(|u| u.name)
  |> take(10)

# Works with method calls via dot
raw_data
  |> CSV.parse
  |> .filter(|row| row["age"].to_int() > 18)
  |> .map(|row| row["name"])
  |> .join(", ")
\```

**Pipe rules:**
- `a |> f` is equivalent to `f(a)`.
- `a |> f(b, c)` is equivalent to `f(a, b, c)`.
- `a |> .method()` calls `method()` on the result of `a`.
- Pipes are left-associative: `a |> f |> g` is `g(f(a))`.
```

**Step 3: Add `|>` to the "Not overloadable" row**

In the operator overloading section, add `|>` to the not-overloadable list.

**Step 4: Commit**

```
git add Opal.md
git commit -m "spec: add pipe operator |> for data flow pipelines (proposal A1)"
```

---

## Task 3: Drop `unless` and `until`

**Proposal:** B1 — Remove negated control flow.

**Files:**
- Modify: `Opal.md` — BNF `<conditional>` and `<loop>` rules (lines ~127, ~130), section 5.1 (lines 873-898), section 5.2 (lines 900-933)

**Step 1: Remove `unless` from BNF**

Change `<conditional>` to:
```bnf
<conditional>   ::= "if" <expression> NEWLINE <block>
                     ("elsif" <expression> NEWLINE <block>)*
                     ("else" NEWLINE <block>)? "end"
```

Remove `until` from `<loop>`:
```bnf
<loop>          ::= "while" <expression> NEWLINE <block> "end"
                   | "for" IDENTIFIER "in" <expression> NEWLINE <block> "end"
```

**Step 2: Remove unless/until examples from sections 5.1 and 5.2**

In 5.1, remove the `unless` block and its suffix form. Keep suffix-`if`:
```opal
# Suffix form (single expression)
print("even") if n % 2 == 0
```

In 5.2, remove the `until` block.

**Step 3: Search for any `unless`/`until` usage in other examples throughout the spec**

Grep and replace any remaining uses in other sections.

**Step 4: Commit**

```
git add Opal.md
git commit -m "spec: remove unless/until — use if not/while not instead (proposal B1)"
```

---

## Task 4: Add null-safe chaining `?.` and null coalescing `??`

**Proposal:** A3 — Essential ergonomics for nullable types.

**Files:**
- Modify: `Opal.md` — BNF (already added operators in Task 2), section 4.4 Operators, section 6.2 Type System (lines 1205-1251)

**Step 1: Add BNF rules for `?.` expressions**

Add to `<expression>`:
```bnf
<expression>    ::= ...
                   | <expression> "?." IDENTIFIER
                   | <expression> "?." IDENTIFIER "(" <args> ")"
                   | <expression> "??" <expression>
```

**Step 2: Add a subsection after the pipe operator section**

```markdown
#### Null-Safe Chaining and Null Coalescing

The `?.` operator short-circuits to `null` if the left side is null, instead of raising an error. The `??` operator provides a default value when the left side is null.

\```opal
# Without null-safe chaining
city = null
if user is User
  addr = user.address
  if addr is Address
    city = addr.city
  end
end

# With null-safe chaining
city = user?.address?.city

# Null-safe method calls
length = user?.name?.upper()?.length

# Null coalescing — default when null
city = user?.address?.city ?? "Unknown"
port = config["port"] ?? 8080
name = get_name() ?? "anonymous"
\```

**Rules:**
- `a?.b` returns `null` if `a` is null, otherwise returns `a.b`.
- `a?.b()` returns `null` if `a` is null, otherwise calls `a.b()`.
- `a ?? b` returns `a` if `a` is not null, otherwise returns `b`.
- `??` is short-circuit: `b` is not evaluated if `a` is not null.
- `?.` chains: `a?.b?.c` returns `null` if any part is null.
```

**Step 3: Update the type system section (6.2)**

In the nullable types discussion, add a note:
```
Use `?.` for safe access and `??` for defaults — see [4.4 Operators](#44-operators).
```

**Step 4: Commit**

```
git add Opal.md
git commit -m "spec: add null-safe chaining ?. and null coalescing ?? (proposal A3)"
```

---

## Task 5: Add list and dict comprehensions

**Proposal:** A4 — Readable collection creation.

**Files:**
- Modify: `Opal.md` — BNF (add `<comprehension>` rule), section 4.5 Collections (after 4.5.1 Lists, line ~694)

**Step 1: Add BNF rule**

Add after `<list>`:
```bnf
<list_comp>     ::= "[" <expression> "for" IDENTIFIER "in" <expression>
                     ("for" IDENTIFIER "in" <expression>)*
                     ("if" <expression>)? "]"
<dict_comp>     ::= "{" <expression> ":" <expression> "for" IDENTIFIER "in" <expression>
                     ("if" <expression>)? "}"
```

Add `<list_comp>` and `<dict_comp>` to `<expression>`.

**Step 2: Add comprehension section after Collection Methods**

```markdown
#### Comprehensions

List and dict comprehensions provide a concise way to create collections from iteration and filtering.

\```opal
# List comprehension
squares = [x ** 2 for x in 1..10]
# => [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]

# With filter
even_squares = [x ** 2 for x in 1..10 if x % 2 == 0]
# => [4, 16, 36, 64, 100]

# Dict comprehension
name_lengths = {name: name.length for name in ["alice", "bob", "carol"]}
# => {"alice": 5, "bob": 3, "carol": 5}

# Nested iteration
pairs = [(x, y) for x in 1..3 for y in 1..3 if x != y]
# => [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]

# With destructuring
adults = [name for (name, age) in people if age >= 18]
\```

Comprehensions are sugar for `filter` + `map`. Both styles are available — use whichever reads better for the situation.
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add list and dict comprehensions (proposal A4)"
```

---

## Task 6: Change `on fail` to `catch`, keep `fail` and `ensure`

**Proposal:** B3 — Familiar error handling keywords.

**Files:**
- Modify: `Opal.md` — BNF `<try_expr>` (line ~168), section 7.1 Error Handling (lines 2325-2460), ALL examples throughout spec that use `on fail`
- Modify: `docs/features/error-handling.md` — all `on fail` instances
- Modify: `docs/features/concurrency.md` — any `on fail` in actor examples

**Step 1: Update BNF**

Change:
```bnf
<try_expr>      ::= "try" NEWLINE <block>
                     ("on" "fail" TYPE ("as" IDENTIFIER)? NEWLINE <block>)*
                     ("ensure" NEWLINE <block>)?
                     "end"
```
to:
```bnf
<try_expr>      ::= "try" NEWLINE <block>
                     ("catch" TYPE ("as" IDENTIFIER)? NEWLINE <block>)*
                     ("catch" ("as" IDENTIFIER)? NEWLINE <block>)?
                     ("ensure" NEWLINE <block>)?
                     "end"
```

**Step 2: Replace ALL `on fail` with `catch` throughout Opal.md**

Global search-and-replace in section 7.1 and all other sections that have try blocks (concurrency section 8 has some, DDD example in 9.2, etc.). The keyword `fail` for raising stays unchanged.

Key examples to update:
- Section 7.1: main error handling examples (lines ~2369-2381)
- Section 7.1: bridging examples (lines ~2427-2449)
- Section 8.1: actor receive with try (if any)
- Section 8.3: async error handling (lines ~2697-2701)
- Section 9.2: DDD example (lines ~3099-3106)
- Section 10.2: test macro (line ~3488)

**Step 3: Update error handling feature doc**

In `docs/features/error-handling.md`, replace all `on fail` with `catch`.

**Step 4: Update section 7.1 prose**

Change the explanation text from "on fail catches" to "catch handles" and similar.

**Step 5: Commit**

```
git add Opal.md docs/features/error-handling.md
git commit -m "spec: replace 'on fail' with 'catch' for error handling (proposal B3)"
```

---

## Task 7: Reserve `@` for macros only — add `requires` for preconditions

**Proposal:** C4 + C3 — Clean up `@` ambiguity, unify guard system.

**Files:**
- Modify: `Opal.md` — BNF (add `<requires_expr>`, remove guard decorator from `<macro_invoke>`), section 7.2 Guards & Rules (lines 2462-2501), section 6.7 Multiple Dispatch (lines 1960-1973), section 10.2 Macros (lines 3384-3391)
- Modify: `docs/features/self-hosting-foundations.md` — if guard decorators are referenced
- Modify: `docs/features/validation-and-settings.md` — if guard decorators are referenced

**Step 1: Add `requires` to BNF**

Add new rule:
```bnf
<requires_expr> ::= "requires" <expression> ("," STRING)?
```

Add `<requires_expr>` to `<block>` (valid as first statement in a function body).

Remove the guard-as-decorator note from `<macro_invoke>`.

**Step 2: Rewrite section 7.2 Guards & Rules**

Replace the current guard system with:

```markdown
### 7.2 Preconditions & Validation

#### Function Preconditions (`requires`)

`requires` validates conditions at the start of a function body. If the condition is false, raises a `PreconditionError`.

\```opal
def sqrt(value::Float64) -> Float64
  requires value >= 0, "sqrt requires non-negative input"
  value ** 0.5
end

sqrt(4.0)   # => 2.0
sqrt(-1.0)  # raises PreconditionError: "sqrt requires non-negative input"

# Multiple preconditions
def transfer(from::Account, to::Account, amount::Float64)
  requires amount > 0, "amount must be positive"
  requires from.balance >= amount, "insufficient funds"
  from.withdraw(amount)
  to.deposit(amount)
end
\```

#### Reusable Validators

Validators are regular functions returning `Bool`. They work in both `requires` and model `where` clauses.

\```opal
def positive?(value) -> Bool
  value > 0
end

def valid_email?(value) -> Bool
  /^[^@]+@[^@]+\.[^@]+$/.match?(value)
end

# In function preconditions
def sqrt(value::Float64) -> Float64
  requires positive?(value)
  value ** 0.5
end

# Same validators in model fields
model Account
  needs email::String where valid_email?
  needs age::Int32 where |v| v >= 0
  needs deposit::Float64 where positive?
end
\```
```

**Step 3: Remove `@` for guards from Multiple Dispatch section (6.7)**

In section 6.7, the `@positive` guard decorator example (lines ~1966-1973) should be rewritten using `requires`:

```opal
def process(value::Int32)
  print("generic integer")
end

def process(value::Int32)
  requires value > 0
  print("positive integer")
end

process(5)   # => "positive integer" (requires passes)
process(-3)  # => "generic integer"  (requires fails, falls to base)
```

**Step 4: Update macro section to clarify `@` is macros only**

In section 10.2, update the rules to remove the "Guards are resolved first" note:
```
- `@name args` invokes a macro at parse time. `@` is reserved exclusively for macros.
```

**Step 5: Commit**

```
git add Opal.md docs/features/self-hosting-foundations.md docs/features/validation-and-settings.md
git commit -m "spec: reserve @ for macros, add requires for preconditions (proposals C3/C4)"
```

---

## Task 8: Constructor sugar `Type(args)`

**Proposal:** B2 — Callable classes shorthand.

**Files:**
- Modify: `Opal.md` — section 6.3 Classes & Methods (lines ~1417-1456)

**Step 1: Add constructor sugar explanation**

After the `.new()` example in section 6.3, add:

```markdown
#### Constructor Shorthand

`Type(args)` is sugar for `Type.new(args)`. Both forms are equivalent.

\```opal
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
\```

**Rules:**
- `Type(args)` is sugar for `Type.new(args)`.
- Works for classes, models, and error types.
- No ambiguity: class names are PascalCase, functions are snake_case.
- Enum variants already use this form: `Shape.Circle(radius: 5.0)`.
```

**Step 2: Update examples throughout spec to use the shorthand where cleaner**

Selectively update construction examples to use the shorthand. Keep `.new()` in the first introduction of each concept (for explicitness), then use shorthand in subsequent examples.

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add Type(args) constructor sugar for Type.new(args) (proposal B2)"
```

---

## Task 9: Positional enum construction

**Proposal:** C1 — Align construction with destructuring.

**Files:**
- Modify: `Opal.md` — section 6.9 Enums (lines 2056-2200)
- Modify: `docs/features/enums-and-algebraic-types.md`

**Step 1: Update enum construction examples**

In section 6.9, add positional construction:

```opal
# Construction: positional or named
s = Shape.Circle(5.0)                     # positional
s = Shape.Circle(radius: 5.0)             # named (equivalent)
r = Response.Success("hello", {:})        # positional
r = Response.Success(body: "hello", headers: {:})  # named

# Single-field variants: positional is natural
opt = Option.Some(42)
```

**Step 2: Update the enum rules list**

Add:
```
- Variants with fields support both positional and named arguments.
- Positional construction matches field declaration order.
```

**Step 3: Update feature doc**

In `docs/features/enums-and-algebraic-types.md`, add positional construction to the examples.

**Step 4: Commit**

```
git add Opal.md docs/features/enums-and-algebraic-types.md
git commit -m "spec: allow positional enum variant construction (proposal C1)"
```

---

## Task 10: Iterator protocol uses `Option(T)`

**Proposal:** A5 — Dogfood the type system.

**Files:**
- Modify: `Opal.md` — section 6.8 Iterator Protocol (lines 1975-2054)
- Modify: `docs/features/self-hosting-foundations.md` — iterator section

**Step 1: Update protocol definitions**

Change:
```opal
protocol Iterator
  def next() -> (value, done::Bool)
end
```
to:
```opal
protocol Iterator(T)
  def next() -> Option(T)
end
```

**Step 2: Update all iterator examples**

Update `FileLines`, `Counter`, and any other iterator examples to return `Option.Some(value)` or `Option.None` instead of `(value, false)` / `(null, true)`.

```opal
class FileLinesIterator implements Iterator(String)
  needs file::File

  def next() -> Option(String)
    line = .file.read_line()
    if line == null
      Option.None
    else
      Option.Some(line)
    end
  end
end
```

**Step 3: Update rules text**

Change:
```
- `Iterator.next()` returns a tuple `(value, done::Bool)`.
```
to:
```
- `Iterator.next()` returns `Option(T)` — `Option.Some(value)` for the next element, `Option.None` when exhausted.
```

**Step 4: Update feature doc**

In `docs/features/self-hosting-foundations.md`, update the iterator protocol and examples.

**Step 5: Commit**

```
git add Opal.md docs/features/self-hosting-foundations.md
git commit -m "spec: iterator protocol uses Option(T) instead of (value, done) tuple (proposal A5)"
```

---

## Task 11: Simplify `def :init` to `def init`

**Proposal:** B5 — Remove symbol syntax for constructor.

**Files:**
- Modify: `Opal.md` — ALL occurrences of `:init` (dozens throughout the spec)
- Modify: `docs/features/classes-and-inheritance.md`
- Modify: `docs/features/concurrency.md` — actor `:init`
- Modify: `docs/features/self-hosting-foundations.md` — custom error `:init`

**Step 1: Global replace `:init` with `init` in Opal.md**

Search for `def :init` and replace with `def init`. Also search for `super()` in `:init` context — those references stay, but the prose "In `:init`" becomes "In `init`".

Key sections to update:
- Section 6.3 Classes (lines ~1421-1505): multiple `:init` references
- Section 7.1 Error Handling: custom error `:init` (lines ~2348-2352)
- Section 7.3 Null Objects: NullPerson `:init` (line ~2518)
- Section 8.1 Actors: actor `:init` (lines ~2567, 2602)
- Section 8.5 Complete Example (line ~2779)
- Section 9.1 DI: OrderService `:init` (lines ~1486-1491)
- Section 9.2 Events and DDD example

**Step 2: Update BNF**

The BNF doesn't have an explicit `:init` rule (it's parsed as a symbol-named function). Add clarity:
- In `<function_def>`, note that `init` is the constructor name (no symbol prefix).

**Step 3: Update all feature docs**

Replace `:init` with `init` in:
- `docs/features/classes-and-inheritance.md`
- `docs/features/concurrency.md`
- `docs/features/self-hosting-foundations.md`

**Step 4: Update prose**

Change "Classes use `def :init()` for construction" to "Classes use `def init()` for construction" and similar.

**Step 5: Commit**

```
git add Opal.md docs/features/classes-and-inheritance.md docs/features/concurrency.md docs/features/self-hosting-foundations.md
git commit -m "spec: simplify def :init to def init for constructors (proposal B5)"
```

---

## Task 12: Reduce `as` overload — settings and null objects

**Proposal:** C2 — Give `as` fewer meanings.

**Files:**
- Modify: `Opal.md` — BNF `<model_def>` (line ~217), `<null_object_def>` (line ~242), section 6.10 Models (line ~2201), section 7.3 Null Objects (line ~2502), section 9.4 Settings (line ~3179)
- Modify: `docs/features/validation-and-settings.md`

**Step 1: Change settings model syntax**

Replace `model X as Settings` with `settings model X`:

BNF change:
```bnf
<model_def>     ::= "model" IDENTIFIER ...
<settings_def>  ::= "settings" "model" IDENTIFIER ...
```

**Step 2: Change null object syntax**

Replace `class X as Y defaults {...}` with `class X < Y defaults {...}` (using inheritance operator):

BNF change:
```bnf
<null_object_def> ::= "class" IDENTIFIER "<" IDENTIFIER "defaults" <dict>
```

**Step 3: Update section 9.4 Settings examples**

```opal
# Before
model AppSettings as Settings
  needs debug::Bool = false
end

# After
settings model AppSettings
  needs debug::Bool = false
end
```

**Step 4: Update section 7.3 Null Objects examples**

```opal
# Before
class AnonymousPerson as Person defaults {name: "anonymous", age: 0}

# After
class AnonymousPerson < Person defaults {name: "anonymous", age: 0}
```

**Step 5: Update feature doc and Specification section 9.3**

Update `docs/features/validation-and-settings.md` to use `settings model`.

Check section 9.3 Specifications — the `class OverAgeSpec as Specification` pattern. This is inheritance, so it should use `<`:
```opal
class OverAgeSpec < Specification
```

**Step 6: Commit**

```
git add Opal.md docs/features/validation-and-settings.md
git commit -m "spec: reduce 'as' overload — settings model, < for null objects (proposal C2)"
```

---

## Task 13: No-arg closure syntax cleanup

**Proposal:** B4 — Fix `||` looking like logical OR.

**Files:**
- Modify: `Opal.md` — BNF `<lambda>` rule (line ~116), section 6.1 Closures (lines 1150-1203)

**Step 1: Update BNF**

Change lambda rule to allow `do...end` for no-arg closures:
```bnf
<lambda>        ::= "|" <params> "|" <expression>
                   | "|" <params> "|" NEWLINE <block> "end"
                   | "do" <expression> "end"
                   | "do" NEWLINE <block> "end"
                   | "do" "|" <params> "|" NEWLINE <block> "end"
```

**Step 2: Update closure examples in section 6.1**

Replace `||` no-arg closures:
```opal
# No-arg closure
counter = 0
increment = do counter += 1 end

# Multi-line no-arg
setup = do
  load_config()
  init_db()
end

# With arguments — |params| syntax
double = |x| x * 2
transform = |items, fn|
  items.map(fn)
end
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: use do...end for no-arg closures instead of || (proposal B4)"
```

---

## Task 14: Trailing block syntax

**Proposal:** D1 — Ruby-style trailing blocks for DSL ergonomics.

**Files:**
- Modify: `Opal.md` — section 6.1 Closures (after lambda section), BNF `<function_call>`

**Step 1: Update BNF**

Extend `<function_call>`:
```bnf
<function_call> ::= IDENTIFIER "(" <args> ")" <trailing_block>?
                   | <expression> "." IDENTIFIER "(" <args> ")" <trailing_block>?
                   | IDENTIFIER <trailing_block>
<trailing_block>::= "do" ("|" <params> "|")? NEWLINE <block> "end"
```

**Step 2: Add trailing block explanation to section 6.1**

```markdown
#### Trailing Blocks

When the last argument to a function is a closure, it can be written as a trailing `do...end` block after the call.

\```opal
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

# Already used in event handlers — this formalizes the pattern
on OrderPlaced do |e|
  .mailer.send_confirmation(e.order)
end
\```

**Rules:**
- The trailing block becomes the last argument to the function call.
- `f(a, b) do |x| ... end` is equivalent to `f(a, b, |x| ... end)`.
- Only one trailing block per call.
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add trailing block syntax for last-arg closures (proposal D1)"
```

---

## Task 15: Add `fn` keyword for named closures

**Proposal:** D2 — Alternative closure syntax for stored functions.

**Files:**
- Modify: `Opal.md` — BNF `<lambda>` (extend), section 6.1 Closures

**Step 1: Update BNF**

Add to lambda rule:
```bnf
<lambda>        ::= ...
                   | "fn" "(" <params> ")" <expression> "end"
                   | "fn" "(" <params> ")" NEWLINE <block> "end"
```

**Step 2: Add `fn` section to 6.1**

```markdown
#### Named Closures with `fn`

The `fn` keyword creates a function value. Use it when assigning closures to variables or passing complex multi-line functions.

\```opal
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
\```

**When to use which:**
- `|params| expr` — inline closures passed directly to functions.
- `fn(params) ... end` — closures stored in variables or with multi-line bodies.
- Both create the same type of value — the choice is stylistic.
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add fn keyword for named closure values (proposal D2)"
```

---

## Task 16: Pattern-matching `receive` in actors

**Proposal:** D4 — Unify actor messages with match/case.

**Files:**
- Modify: `Opal.md` — BNF `<receive_clause>` (line ~175), section 8.1 Actors (lines 2561-2620)
- Modify: `docs/features/concurrency.md`

**Step 1: Update BNF**

Change:
```bnf
<receive_clause>::= "receive" SYMBOL ("(" <params> ")")? NEWLINE <block> "end"
```
to:
```bnf
<receive_clause>::= "receive" NEWLINE <case_clause>+ "end"
```

This reuses the existing `<case_clause>` from match expressions.

**Step 2: Rewrite actor examples in section 8.1**

```opal
actor Counter
  def init()
    .count = 0
  end

  receive
    case :increment
      .count += 1
      reply .count
    case :get_count
      reply .count
    case :reset
      .count = 0
      reply :ok
  end
end

# Messages with arguments
actor Cache
  def init(ttl::Int32)
    .store = {:}
    .ttl = ttl
  end

  receive
    case :get(key)
      reply .store[key]
    case :set(key, value)
      .store[key] = value
      reply :ok
    case :delete(key)
      .store.delete(key)
      reply :ok
  end
end
```

**Step 3: Update complete example in 8.5 and DDD example in 9.2**

**Step 4: Update concurrency feature doc**

**Step 5: Commit**

```
git add Opal.md docs/features/concurrency.md
git commit -m "spec: unify actor receive with match/case pattern matching (proposal D4)"
```

---

## Task 17: Add `let` for immutable bindings

**Proposal:** D5 — Distinguish mutable from immutable variables.

**Files:**
- Modify: `Opal.md` — BNF `<assignment>` (line ~67), section 4.2 Variables & Assignment (lines 272-294)

**Step 1: Update BNF**

Change:
```bnf
<assignment>    ::= IDENTIFIER "=" <expression>
```
to:
```bnf
<assignment>    ::= IDENTIFIER "=" <expression>
                   | "let" IDENTIFIER "=" <expression>
                   | "let" <destructure> "=" <expression>
```

**Step 2: Add `let` explanation to section 4.2**

```markdown
#### Immutable Bindings

`let` creates an immutable binding — the variable cannot be reassigned after initialization.

\```opal
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
\```

**Rules:**
- `let x = expr` creates an immutable binding.
- `x = expr` (without `let`) creates a mutable binding — backward compatible.
- Function parameters are implicitly immutable.
- `let` works with destructuring and type annotations.
- Reassigning a `let` binding is a compile-time error.
```

**Step 3: Update naming conventions**

Add to the conventions:
```
- `let` for values that shouldn't change after assignment
- Bare assignment for values that need mutation
```

**Step 4: Commit**

```
git add Opal.md
git commit -m "spec: add let for immutable variable bindings (proposal D5)"
```

---

## Task 18: Multiline comment syntax `###`

**Proposal:** E1 — Symmetrical, easier to type.

**Files:**
- Modify: `Opal.md` — section 4.1 Comments (lines 257-270)

**Step 1: Update comment section**

Change from `#{ }#` to `###`:

```markdown
### 4.1 Comments

Single-line comments begin with `#`. Multiline comments are delimited by `###`.

\```opal
# This is a single-line comment

###
  This is a multiline comment.
  It can span as many lines as needed.
###

x = 42  # inline comment
\```
```

**Step 2: Search for any `#{ }#` usage elsewhere in spec**

Replace any remaining occurrences.

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: change multiline comments from #{ }# to ### (proposal E1)"
```

---

## Task 19: Add `in` operator for membership testing

**Proposal:** E2 — Natural membership checks.

**Files:**
- Modify: `Opal.md` — BNF `<binary_op>` (add `in`), section 4.4 Operators

**Step 1: Add `in` to operators**

Add to the Logical operators table:
```
| `in` | Membership test |
| `not in` | Negated membership test |
```

Add to "Not overloadable" list: `in`.

Actually — `in` SHOULD be overloadable (via a `contains?` protocol method). Add to overloadable instead:
```
| Membership | `in` (delegates to `contains?()`) |
```

**Step 2: Add examples**

```markdown
#### Membership

\```opal
3 in [1, 2, 3, 4]         # => true
"key" in {"key": "value"}  # => true (checks keys)
'c' in 'a'..'z'            # => true
42 not in [1, 2, 3]        # => true

# in is sugar for .contains?()
list.contains?(3)          # equivalent to: 3 in list
\```
```

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add in/not in operators for membership testing (proposal E2)"
```

---

## Task 20: Add string repeat operator

**Proposal:** E3 — Missing basic feature.

**Files:**
- Modify: `Opal.md` — section 4.3.5 Strings (String Methods subsection, around line 503)

**Step 1: Add string repeat to String Methods**

Under the Transforming section:
```opal
"ha" * 3                    # => "hahaha"
"-" * 40                    # => "----------------------------------------"
```

**Step 2: Note in operator overloading section**

This is enabled by `def *(other::Int32) -> String` on the String class. Already covered by the general operator overloading mechanism.

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: add string repeat operator * (proposal E3)"
```

---

## Task 21: Update philosophy line and design facts

**Files:**
- Modify: `Opal.md` — section 1 Design Philosophy (line 13), section 2 Facts (lines 19-33)

**Step 1: Update philosophy to reflect new features**

Ensure the philosophy mentions pipe operator and comprehensions as part of the functional toolkit. Add immutable bindings to the "explicitness" line if appropriate.

**Step 2: Review Facts table**

No changes needed — the facts are high-level enough to still be accurate.

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: update philosophy section for new syntax features"
```

---

## Task 22: Update stdlib table and feature doc cross-references

**Files:**
- Modify: `Opal.md` — section 11 Standard Library (lines 3765-3812)
- Modify: `Opal.md` — any broken cross-references

**Step 1: Update stdlib table**

Add `Option` usage note about iterators. Ensure `Result` and `Option` entries are consistent with new iterator protocol.

**Step 2: Scan for broken cross-references**

After all changes, section numbers may have shifted. Scan for `[section X.Y]` style references and internal links.

**Step 3: Commit**

```
git add Opal.md
git commit -m "spec: update stdlib table and fix cross-references"
```

---

## Task 23: Final consistency review

**Files:**
- Read: `Opal.md` — full document
- Read: All `docs/features/*.md`

**Step 1: BNF audit**

Read the entire BNF (section 3) end-to-end and verify every rule matches the prose and examples.

**Step 2: Example audit**

Spot-check 10-15 code examples throughout the spec to ensure they use updated syntax:
- No `on fail` (should be `catch`)
- No `:init` (should be `init`)
- No `unless`/`until`
- No `model X as Settings` (should be `settings model X`)
- No `#{ }#` (should be `###`)

**Step 3: Feature doc audit**

Ensure all 11 feature docs are consistent with the updated spec.

**Step 4: Final commit**

```
git add -A
git commit -m "spec: final consistency review after syntax improvements"
```

---

## Execution Order & Dependencies

```
Task 1  (elsif)           — no deps
Task 2  (pipe |>)         — no deps
Task 3  (drop unless)     — after Task 1 (elsif replaces some unless patterns)
Task 4  (?. and ??)       — after Task 2 (BNF already extended)
Task 5  (comprehensions)  — no deps
Task 6  (catch)           — no deps
Task 7  (requires, @)     — no deps
Task 8  (Type(args))      — no deps
Task 9  (positional enum) — no deps
Task 10 (Option iterator) — no deps
Task 11 (def init)        — no deps
Task 12 (reduce as)       — no deps
Task 13 (no-arg closures) — no deps
Task 14 (trailing blocks) — after Task 13 (uses do...end from B4)
Task 15 (fn keyword)      — after Task 13 (closure section reorganized)
Task 16 (receive match)   — after Task 11 (uses def init not :init)
Task 17 (let bindings)    — no deps
Task 18 (### comments)    — no deps
Task 19 (in operator)     — no deps
Task 20 (string repeat)   — no deps
Task 21 (philosophy)      — after all feature tasks
Task 22 (stdlib/xrefs)    — after all feature tasks
Task 23 (consistency)     — LAST
```

**Parallelizable groups:**
- Group A (no deps): Tasks 1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 17, 18, 19, 20
- Group B (after deps): Tasks 3, 4, 14, 15, 16
- Group C (after all): Tasks 21, 22, 23
