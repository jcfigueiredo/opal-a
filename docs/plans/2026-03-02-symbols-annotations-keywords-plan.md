# Symbols, Annotations & Keywords Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add typed symbol sets, declarative annotations (`@[...]`), actor `receives` declarations, and clean up reserved words — all backward compatible — across Opal.md and feature docs.

**Architecture:** This is a spec-only project (no compiler). Each task edits markdown documentation. "Testing" means verifying BNF/prose/example consistency. Changes flow: BNF grammar first (foundation), then prose sections, then feature docs, then cross-references.

**Tech Stack:** Markdown specification files. No code to compile or tests to run.

---

## Dependency Graph

```
Task 1 (BNF) ──→ Task 2 (Symbol Sets section)
              ──→ Task 3 (Annotations section)
              ──→ Task 4 (Actor receives + Supervisor cleanup)
Tasks 2-4    ──→ Task 5 (Feature docs: concurrency.md, type-system.md, DI)
              ──→ Task 6 (Feature docs: metaprogramming.md, testing.md)
Tasks 5-6    ──→ Task 7 (CLAUDE.md + philosophy + stdlib table)
Task 7       ──→ Task 8 (Final consistency audit)
```

Tasks 2, 3, 4 are independent of each other (different sections of Opal.md).
Tasks 5 and 6 are independent of each other (different feature docs).

---

### Task 1: Update BNF Grammar

**Files:**
- Modify: `Opal.md:37-279` (Section 3 — Formal Grammar)

**What to change:**

**Step 1: Add annotation rule** after `<macro_invoke>` (line 218)

Add these new BNF rules:

```bnf
<annotation>    ::= "@[" <annot_entry> ("," <annot_entry>)* "]"
<annot_entry>   ::= IDENTIFIER
                   | IDENTIFIER ":" <expression>
```

**Step 2: Add `receives` to actor_body** (line 198)

Change:
```bnf
<actor_body>    ::= (<needs_decl> | <function_def> | <receive_clause>)*
```

To:
```bnf
<actor_body>    ::= ("receives" <symbol_list> NEWLINE)?
                     (<needs_decl> | <function_def> | <receive_clause>)*
<symbol_list>   ::= SYMBOL ("," SYMBOL)*
                   | IDENTIFIER
```

**Step 3: Update supervisor_body** — remove `within` keyword (lines 202-204)

Change:
```bnf
<supervisor_body>::= ("strategy" SYMBOL NEWLINE)?
                     ("max_restarts" INTEGER "within" INTEGER NEWLINE)?
                     ("supervise" <expression> NEWLINE)*
```

To:
```bnf
<supervisor_body>::= ("strategy" SYMBOL NEWLINE)?
                     ("max_restarts" INTEGER "," INTEGER NEWLINE)?
                     ("supervise" <expression> NEWLINE)*
```

**Step 4: Add `<annotation>` to `<statement>` rule** (around line 42)

Add `| <annotation>` to the statement productions, or more accurately, add it as an optional prefix to declarations. The simplest approach: add it to `<class_def>`, `<function_def>`, and `<needs_decl>` as optional prefixes.

Update:
```bnf
<function_def>  ::= <annotation>* "def" IDENTIFIER "(" <params> ")" ("->" <type_expr>)? NEWLINE <block> "end"
<class_def>     ::= <annotation>* "class" IDENTIFIER ("(" <type_params> ")")? ("<" IDENTIFIER)?
                     (<where_clause>)? NEWLINE <class_body> "end"
<needs_decl>    ::= <annotation>* "needs" IDENTIFIER "::" TYPE ("=" <expression>)?
```

**Step 5: Verify** — Read back the full BNF section and check no rules reference `"within"`, that `<annotation>` is well-formed, and `<actor_body>` includes the optional `receives`.

**Step 6: Commit**

```
git add Opal.md
git commit -m "BNF: add annotation, receives, symbol list rules; remove within"
```

---

### Task 2: Add Symbol Sets to Basics Section

**Files:**
- Modify: `Opal.md:595-606` (Section 4.3.6 — Symbols)
- Modify: `Opal.md:~1440-1450` (Core types table, if it exists)

**Step 1: Expand the Symbols section** (currently just 6 lines at 595-605)

Replace the current minimal symbols section with:

```markdown
#### 4.3.6 Symbols

Symbols are self-identifying constants. They do not need to be assigned a value.

\```opal
:hi
:bye
:"I have spaces."
:really?
:yes!
\```

#### Symbol Sets (Typed Symbols)

Symbols can form **symbol sets** — lightweight type aliases that constrain which symbols are valid in a given context. This bridges dynamic atoms with static safety.

\```opal
# Named symbol set — a union of symbol literals
type Status = :ok | :error | :pending
type HttpMethod = :get | :post | :put | :delete | :patch
type LogLevel = :debug | :info | :warn | :error

# Use as a type annotation
def handle(status::Status)
  match status
    case :ok      then print("success")
    case :error   then print("failure")
    case :pending then print("waiting")
  end
end

handle(:ok)       # works
handle(:unknown)  # TYPE ERROR: :unknown is not in Status

# Inline symbol constraint (no named type needed)
def log(level:: :debug | :info | :warn | :error, message::String)
  print(f"[{level}] {message}")
end
\```

**Symbol set rules:**
- `type Name = :a | :b | :c` defines a symbol set (a type alias of a union of symbol literals).
- Symbol sets participate in exhaustiveness checking — the compiler warns on incomplete match.
- `Symbol` remains the unconstrained type (accepts any symbol) — gradual typing.
- Symbol sets compose with unions, generics, and constraints.

**Symbol sets vs enums:** Symbol sets are for simple tags with no data. `enum` is for data-carrying variants:

\```opal
# Symbol set — lightweight tags
type Direction = :north | :south | :east | :west

# Enum — data-carrying variants
enum Shape
  Circle(radius::Float64)
  Rect(width::Float64, height::Float64)
end
\```
```

**Step 2: Add exhaustiveness checking example** to the match section (around line 1087 where match patterns are documented). Add a brief note that symbol sets enable exhaustiveness checking in match expressions:

After the existing match examples, add:

```markdown
#### Exhaustive Matching with Symbol Sets

When matching on a symbol set type, the compiler checks for exhaustiveness:

\```opal
type Color = :red | :green | :blue

def describe(c::Color) -> String
  match c
    case :red   then "warm"
    case :green then "cool"
    # COMPILE WARNING: non-exhaustive match, missing :blue
  end
end
\```
```

**Step 3: Verify** — Read the symbols section, the match section, and any core types table. Ensure "Symbol" is still listed as a core type and that the new examples use correct Opal syntax (double quotes for strings, `::` for type annotations, etc.).

**Step 4: Commit**

```
git add Opal.md
git commit -m "Add symbol sets with typed symbols and exhaustiveness checking"
```

---

### Task 3: Add Annotations Section to Metaprogramming

**Files:**
- Modify: `Opal.md:3500-3700` (Section 10 — Metaprogramming)

**Step 1: Add new subsection** after Section 10.2 Macros (after the macro rules around line 3654). Insert a new subsection:

```markdown
### 10.3 Annotations — Declarative Metadata

Annotations attach metadata to declarations. They are distinct from macros: macros transform code, annotations describe it.

| Syntax | Purpose | When | What it does |
|---|---|---|---|
| `@name args` | Macro invocation | Parse time | Transforms code (AST to AST) |
| `@[key: val, ...]` | Annotation | Never "runs" | Attaches metadata, queryable at runtime |

#### Annotation Syntax

\```opal
# Simple tag (presence means true)
@[deprecated]
def old_api()
  # ...
end

# Tag with values
@[deprecated, since: "2.0", replacement: "new_api"]
def old_api()
  # ...
end

# On classes
@[serializable, version: 3]
class User
  needs name::String
  needs email::String
end

# On fields
class Config
  @[env: "DATABASE_URL"]
  needs db_url::String

  @[env: "PORT", default: 8080]
  needs port::Int32
end

# Multiple annotations stack
@[deprecated, since: "1.5"]
@[experimental]
def risky_method()
  # ...
end
\```

#### Querying Annotations

\```opal
# On functions
annotations(old_api)
# => [{deprecated: true, since: "2.0", replacement: "new_api"}]

# On classes
User.annotations()
# => [{serializable: true, version: 3}]

# On fields
Config.field_annotations(:db_url)
# => [{env: "DATABASE_URL"}]

# Check for specific annotation
if :deprecated in annotations(old_api)
  print("This function is deprecated")
end
\```

#### Macros Reading Annotations

Macros can read annotations at parse time — annotations provide data, macros provide transformation:

\```opal
@[json_field, name: "user_name"]
needs name::String

# Macro reads field annotations during code generation
macro json_serializable(class_def)
  fields = class_def.needs_fields()
  for field in fields
    annots = field.annotations()
    json_name = if :json_field in annots
      annots[:json_field][:name]
    else
      field.name.to_string()
    end
    # ... use json_name in generated code
  end
end
\```

#### Built-in Annotations

| Annotation | Purpose |
|---|---|
| `@[deprecated]` | Mark as deprecated (compiler warning on use) |
| `@[deprecated, since: "X", use: "Y"]` | With migration info |
| `@[experimental]` | Mark as unstable API |
| `@[inline]` | Hint to inline this function |
| `@[todo, note: "..."]` | In-code TODO that tooling can collect |
| `@[test_only]` | Only available in test files (`.topl`) |

#### Rules

- `@[...]` attaches metadata as a dict of symbols to values.
- `@name` remains exclusively macro invocation — unchanged.
- Annotations are inert — no code transformation.
- Annotations are queryable via `annotations()` at runtime and by macros at parse time.
- Annotations stack (multiple `@[...]` on the same target).
- Annotations apply to the immediately following declaration (`def`, `class`, `needs`, etc.).
```

**Step 2: Renumber subsequent subsections.** The old 10.3 (AST Reflection) becomes 10.4, old 10.4 (Practical Macros) becomes 10.5, etc. Update all `### 10.X` headings accordingly.

**Step 3: Update the macro rules** (around line 3648-3654). After the line "`@` is reserved exclusively for macros", change to:

```
- `@name args` invokes a macro at parse time.
- `@[key: val]` attaches annotation metadata (see 10.3).
- `@` followed by an identifier is a macro; `@` followed by `[` is an annotation.
```

**Step 4: Update the "What Could Be Macros" section** (around line 3833-3843) — add `annotations()` to the introspection functions listed.

**Step 5: Update the summary table** at the end of Section 10 (around line 4017-4024). Add a row:

```
| `@[key: val]` | Attach metadata annotation |
```

**Step 6: Verify** — Read the full metaprogramming section. Ensure `@[...]` and `@name` are clearly distinguished. Check that renumbered subsections are sequential.

**Step 7: Commit**

```
git add Opal.md
git commit -m "Add annotations section and update metaprogramming subsection numbering"
```

---

### Task 4: Actor `receives` + Supervisor Cleanup

**Files:**
- Modify: `Opal.md:2805-3100` (Section 8 — Concurrency)

**Step 1: Add `receives` to actor examples.** Update the Counter actor example (around line 2820):

Before:
```opal
actor Counter
  def init()
    .count = 0
  end

  receive
    case :increment
```

After:
```opal
actor Counter
  receives :increment, :get_count, :reset

  def init()
    .count = 0
  end

  receive
    case :increment
```

**Step 2: Add `receives` to the Cache actor** (around line 2855):

Add `receives :get, :set, :delete` at the top of the actor body.

**Step 3: Add `receives` to the RateLimiter actor** (around line 3040):

Add `receives :check, :reset` at the top.

**Step 4: Add `receives` to any other actors** in the section (Worker around line 3000, DashboardSupervisor example actors, etc.).

**Step 5: Add a "Message Typing" subsection** after the basic actor examples. Before the "Structured Concurrency" section (around line 2875):

```markdown
#### Actor Message Typing

Actors can optionally declare their message interface with `receives`, enabling compile-time checking of `.send()` calls:

\```opal
actor Cache
  receives :get, :set, :delete

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

cache = Cache.new()
cache.send(:get, "user:1")     # OK — :get is in receives
cache.send(:gett, "user:1")    # COMPILE WARNING: :gett not in Cache.receives
\```

**Rules:**
- `receives :msg1, :msg2, ...` is optional — actors without it accept any symbol (backward compatible).
- When present, `.send()` calls are checked at compile time.
- `receives` uses symbol sets under the hood.
- Named symbol sets work too: `receives Status` where `type Status = :ok | :error`.
- The `receive` block must handle all declared messages (exhaustiveness check).
- Queryable: `Cache.receives()` returns the set of accepted messages.
```

**Step 6: Update supervisor syntax** — remove `within` keyword from all supervisor examples.

Change all occurrences of:
```
max_restarts 3 within 60
```
To:
```
max_restarts 3, 60
```

This affects approximately 3 supervisor definitions in Section 8.

**Step 7: Update the supervisor strategies table** (around line 2983) — no change needed to the table itself, but add a note that `strategy`, `max_restarts`, and `supervise` are contextual keywords (only keywords inside `supervisor` blocks).

**Step 8: Update the concurrency summary table** at the end of Section 8 (around line 3090). Add a row for `receives`:

```
| Declare actor interface | Message typing | `receives :msg1, :msg2` |
```

**Step 9: Verify** — Read the full concurrency section. Check that all actors with `receives` have matching `receive` case clauses. Check no `within` remains. Check all examples use valid Opal syntax.

**Step 10: Commit**

```
git add Opal.md
git commit -m "Add actor receives declaration; remove within keyword from supervisors"
```

---

### Task 5: Update Feature Docs — Concurrency, Type System, DI

**Files:**
- Modify: `docs/features/concurrency.md`
- Modify: `docs/features/type-system.md`
- Modify: `docs/features/dependency-injection-and-events.md`

**Step 1: Update concurrency.md actors.** Add `receives` declarations to all actor examples:

- Counter actor (lines 28-48): Add `receives :increment, :get_count, :reset`
- Cache actor (lines 60-76): Add `receives :get, :set, :delete`
- Worker actor (lines 273-294): Add `receives :do`
- RateLimiter actor (lines 305-323): Add `receives :check, :reset`

**Step 2: Update concurrency.md supervisors.** Change all `within` to comma syntax:

- AppSupervisor (line 236): `max_restarts 3 within 60` → `max_restarts 3, 60`
- DashboardSupervisor (line 350): `max_restarts 5 within 30` → `max_restarts 5, 30`

**Step 3: Add note to concurrency.md** about `strategy`, `max_restarts`, `supervise` being contextual keywords. Add after the supervisor section:

```markdown
**Note:** `strategy`, `max_restarts`, and `supervise` are contextual keywords — they are only reserved inside `supervisor` blocks and can be used as identifiers elsewhere.
```

**Step 4: Update type-system.md** — add symbol sets to the type system documentation. Find the core types section (around line 45 where `Symbol` is listed) and add:

```markdown
#### Symbol Sets

Symbol sets are type aliases over unions of symbol literals:

\```opal
type Status = :ok | :error | :pending
type Direction = :north | :south | :east | :west
\```

Symbol sets enable exhaustiveness checking in `match` and type-safe function signatures. See [Basics: Symbol Sets](#symbol-sets-typed-symbols) for full syntax.
```

**Step 5: Update dependency-injection-and-events.md** — add `receives` to actor examples:

- PaymentProcessor actor (lines 68-75): Add `receives :charge`
- OrderProcessor actor (lines 245-254): Add `receives :process`

**Step 6: Verify** — Read back each modified file. Check that `receives` declarations match `receive` case patterns. Check no `within` remains.

**Step 7: Commit**

```
git add docs/features/concurrency.md docs/features/type-system.md docs/features/dependency-injection-and-events.md
git commit -m "Update feature docs: add receives, remove within, add symbol sets"
```

---

### Task 6: Update Feature Docs — Metaprogramming, Testing

**Files:**
- Modify: `docs/features/metaprogramming.md`
- Modify: `docs/features/testing.md`

**Step 1: Update metaprogramming.md** — add a section distinguishing macros from annotations. After the macro examples section, add:

```markdown
### Macros vs Annotations

Opal has two `@` syntaxes with distinct purposes:

- `@name args` — **macro invocation** (transforms code at parse time)
- `@[key: val, ...]` — **annotation** (attaches metadata, never transforms code)

\```opal
# Macro — transforms the function definition
@memoize
def fibonacci(n::Int32) -> Int32
  if n <= 1 then n else fibonacci(n - 1) + fibonacci(n - 2) end
end

# Annotation — attaches metadata, no transformation
@[deprecated, since: "2.0"]
def old_fibonacci(n::Int32) -> Int32
  # ...
end

# Combined — annotation provides data, macro uses it
@[json_field, name: "user_name"]
needs name::String
\```

Macros can read annotations via `field.annotations()` during code generation. This separates the "what metadata exists" concern (annotations) from the "what code transformation to apply" concern (macros).
```

**Step 2: Update the macros-from-Julia comparison table** in metaprogramming.md. Add a row:

```
| (no equivalent) | `@[key: val]` annotation (metadata, not transformation) |
```

**Step 3: Update testing.md** — no structural changes needed (mock stubs use symbols correctly as-is). But add a brief example showing annotations on test helpers:

```opal
@[test_only]
def create_test_user(name::String) -> User
  User.new(name: name, email: f"{name}@test.com", age: 25)
end
```

This goes near the test utilities or helpers section if one exists, or at the end of the testing patterns section.

**Step 4: Verify** — Read back each file. Check that `@name` and `@[...]` are used correctly and consistently.

**Step 5: Commit**

```
git add docs/features/metaprogramming.md docs/features/testing.md
git commit -m "Update metaprogramming and testing docs with annotation examples"
```

---

### Task 7: Update CLAUDE.md, Philosophy & Stdlib Table

**Files:**
- Modify: `CLAUDE.md`
- Modify: `Opal.md:9-17` (Philosophy section)
- Modify: `Opal.md:4028-4050` (Stdlib table)

**Step 1: Update CLAUDE.md** design rules. Add a rule about annotations:

```markdown
- **`@name` for macros, `@[...]` for annotations**: `@memoize` invokes a macro, `@[deprecated]` attaches metadata
```

Also add a note about symbol sets:

```markdown
- **Symbol sets via type aliases**: `type Status = :ok | :error | :pending` for typed symbols
```

**Step 2: Update philosophy section** (line 13). The existing bullet about first-class concepts should include "annotations" if it doesn't already. Check and add if needed.

**Step 3: Update stdlib table** (around line 4028). Check if annotation-related functions (`annotations()`, `field_annotations()`) should be listed. If there's no obvious module for them, they're likely global functions or on the `Reflect` module. Add if there's a reflection entry.

**Step 4: Update the feature docs table** in CLAUDE.md if the metaprogramming doc description needs updating.

**Step 5: Verify** — Read CLAUDE.md and the relevant Opal.md sections.

**Step 6: Commit**

```
git add CLAUDE.md Opal.md
git commit -m "Update CLAUDE.md and Opal.md philosophy/stdlib for annotations and symbol sets"
```

---

### Task 8: Final Consistency Audit

**Files:**
- Read: All modified files

**Step 1: BNF consistency check.** Read the full BNF (Opal.md lines 37-279). Verify:
- [ ] `<annotation>` rule exists and is well-formed
- [ ] `<actor_body>` includes optional `receives`
- [ ] `<symbol_list>` rule exists
- [ ] `<supervisor_body>` uses comma instead of `within`
- [ ] `<function_def>`, `<class_def>`, `<needs_decl>` allow optional `<annotation>*` prefix
- [ ] No dangling references to removed rules

**Step 2: Cross-reference check.** Verify:
- [ ] Every `receives` declaration in actor examples matches the `receive` case patterns
- [ ] No `within` keyword remains anywhere in the codebase
- [ ] All `@[...]` examples use valid syntax (symbol keys, expression values)
- [ ] All `@name` examples are clearly macros (not annotations)
- [ ] Symbol set examples use `type Name = :a | :b | :c` syntax consistently
- [ ] The distinction between symbol sets and enums is clear everywhere it appears

**Step 3: Example spot-checks.** Read 10+ code examples across modified files and verify:
- [ ] Double quotes for strings (not single quotes)
- [ ] `::` for type annotations
- [ ] `.` prefix for instance variables
- [ ] `f"..."` for interpolation
- [ ] `###` for multiline comments
- [ ] `catch` not `on fail`
- [ ] `def init` not `def :init`

**Step 4: Grep for stale patterns:**
- Search for `within` in all .md files (should only appear in prose, not syntax)
- Search for `@\[` to verify all annotation examples are valid
- Search for `receives` to verify all usages are in actor contexts

**Step 5: Fix any issues found.**

**Step 6: Commit** (if fixes needed)

```
git add -A
git commit -m "Consistency fixes from final audit"
```
