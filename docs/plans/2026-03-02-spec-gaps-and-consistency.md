# Opal Specification Gaps & Consistency Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all inconsistencies in Opal.md, then design and integrate the missing/underspecified language features in priority order.

**Architecture:** This is a specification project — "implementation" means writing spec text, not code. Each phase either fixes existing content (exact edits) or follows the established brainstorm-then-integrate workflow: brainstorm design decisions with the user, write a feature doc to `docs/features/`, integrate into Opal.md (BNF + section + stdlib table + cross-references), and commit.

**Tech Stack:** Markdown, BNF grammar notation, Opal code examples

---

## Phase 1: Fix Inconsistencies

These are bugs in the existing spec. No design decisions needed — just reconcile contradictions.

### Task 1: Fix Pretotyping Section (13)

**Files:**
- Modify: `Opal.md:3343-3378` (section 13)

**Problem:** The Opal example uses `app.get "/" and return "Hello world!"!` — the `and` keyword isn't specified anywhere in the language, and `!` at end of lines contradicts its defined meaning (Result propagation operator, section 7.1). This also contradicts section 10.6 which shows the `@get` macro DSL approach as the idiomatic way to define routes.

**Step 1: Rewrite the Opal pretotyping example**

Replace the stale Flask-style example with one that uses the `@get`/`@post` macro DSL from section 10.6 (OpalWeb subdomain). The comparison should still show Python Flask vs Opal, but Opal should use its own macro system:

```opal
import OpalWeb

app = OpalWeb.App.new("app name")

@get "/" do
  "Hello world!"
end

app.run!
```

Remove the `and` keyword usage entirely. The `!` on `app.run!` is a method name convention (mutation/side-effect), not the Result `!` operator — this is fine but should be clear from context.

**Step 2: Verify consistency**

Check that the rewritten example only uses syntax defined elsewhere in the spec:
- `import` (section 6.4)
- `.new()` with named args (section 6.3)
- `@get` macro invocation (section 10.6)
- `do ... end` block (throughout)
- `.run!` method naming convention (section 6.3 — `!` suffix for mutations)

**Step 3: Commit**

```
git add Opal.md
git commit -m "Fix pretotyping section to use macro DSL syntax"
```

---

### Task 2: Fix Specifications Section (9.3)

**Files:**
- Modify: `Opal.md:2586-2632` (section 9.3)

**Problem:** Uses `@person in (Person)` and `@born_at in (String)` — a type-guard-as-decorator syntax that doesn't appear in the BNF, isn't used anywhere else, and conflicts with the `@name` macro invocation syntax. Also uses `import "patterns.Specification"` with a string path, which is inconsistent with `import Module` elsewhere.

**Step 1: Rewrite using standard type annotations**

Replace `@person in (Person)` with `person::Person` type annotations on parameters — the standard Opal way to constrain types. Replace the string import with module import. Keep the Specification pattern logic intact:

```opal
import Spec.Specification

class Person
  needs name::String
  needs age::Int32
  needs place_of_birth::String
end

class OverAgeSpec as Specification
  def is_satisfied_by(person::Person) -> Bool
    person.age >= 21
  end
end

class BornAtSpec as Specification
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

Key changes:
- `@person in (Person)` -> `person::Person` parameter annotation
- `@born_at in (String)` -> `needs born_at::String` (DI pattern)
- `def :init` -> `needs` (simpler, consistent with DI)
- `import "patterns.Specification"` -> `import Spec.Specification`
- `people.where(...)` -> `people.filter(...)` (consistent with section 4.5.1)
- Spec combination uses method chaining (`.not().and()`) instead of bare operators on specs

**Step 2: Verify consistency**

Check against: type annotations (6.2), `needs` (9.1), `import` (6.4), `filter` (4.5.1), `Spec` module (11).

**Step 3: Commit**

```
git add Opal.md
git commit -m "Fix specifications section to use standard type annotations and needs"
```

---

### Task 3: Fix Null Objects Section (7.3)

**Files:**
- Modify: `Opal.md:1954-1996` (section 7.3)
- Modify: `Opal.md` BNF section (add `null_object_def`)

**Problem:** Uses `as Nullable:Person` and `defaults {name: "anonymous", age: 0}` — neither is in the BNF. The `Nullable:Type` colon syntax is unique to this section and not explained. The `defaults` keyword is mentioned in CLAUDE.md but barely specified.

**Step 1: Add BNF rule for null object definitions**

Add to the BNF (section 3), near `class_def`:

```bnf
<null_object_def> ::= "class" IDENTIFIER "as" IDENTIFIER "defaults" <dict>
```

**Step 2: Rewrite the Null Objects section**

Clarify both forms. The full override form uses normal inheritance. The `defaults` shortcut creates a subclass with preset constructor values:

```opal
class Person
  needs name::String
  needs age::Int32

  def greet()
    print(f"Hi, I'm {.name}")
  end
end

# Full form — subclass with overridden behavior
class NullPerson < Person
  def :init()
    super(name: "anonymous", age: 0)
  end

  def greet()
    print("Hi, I don't want to say my name")
  end
end

# Shortcut — auto-generates a subclass with default values
class AnonymousPerson as Person defaults {name: "anonymous", age: 0}
# Equivalent to a subclass whose :init calls super with these defaults.
# All methods delegate to Person — only construction differs.
```

Key changes:
- Remove `Nullable:Person` syntax — use standard inheritance `< Person` for full override
- Keep `defaults` for the shortcut form — it's the unique value-add
- Use `needs` instead of manual `:init` for Person (consistent with rest of spec)

**Step 3: Verify consistency**

Check against: inheritance (6.3), `needs` (9.1), `defaults` keyword in CLAUDE.md, BNF.

**Step 4: Commit**

```
git add Opal.md
git commit -m "Fix null objects section with standard inheritance and defaults BNF"
```

---

### Task 4: Unify Guard Syntax (7.2)

**Files:**
- Modify: `Opal.md:1915-1952` (section 7.2)

**Problem:** Guards appear in three different syntactic forms across the spec:
1. `@guard_name` decorator on functions (7.2) — pre-condition
2. `where guard_name` on model fields (6.10) — field validation
3. `@name in (Type)` type guard on parameters (7.2) — type constraint

Form 3 is redundant with type annotations (`param::Type`) and conflicts with macro syntax. Forms 1 and 2 are different contexts (pre-conditions vs validation) but should use the same guard functions.

**Step 1: Rewrite Guards section**

Remove `@name in (Type)` entirely — use type annotations instead. Keep `@guard_name` for function pre-conditions and clarify how the same guard functions work in both `@` (decorator) and `where` (model field) contexts:

```opal
# Define a reusable guard
guard positive(value) fails :must_be_positive
  return value > 0
end

guard old_enough(age) fails :too_young
  return age >= 18
end

# As function pre-condition (decorator)
@positive
def sqrt(value::Float64) -> Float64
  value ** 0.5
end

@old_enough
def register_voter(name::String, age::Int32)
  print(f"{name} registered to vote")
end

# Same guards work in model field validation
model Registration
  needs name::String where |v| v.length > 0
  needs age::Int32 where old_enough
  needs amount::Float64 where positive
end
```

Add a rules block:
- `guard name(params) fails :symbol ... end` defines a reusable guard.
- `@guard_name` before a function = pre-condition. The guard receives the function's arguments.
- `where guard_name` on a model field = field validation. The guard receives the field value.
- Guards are the same functions in both contexts — one definition, two uses.
- For type constraints on parameters, use type annotations: `param::Type`.

**Step 2: Verify consistency**

Check against: model `where` (6.10), guard definition (7.2), decorator syntax (10.2 — macros also use `@`).

Note: `@` is used for both macros and guards. Distinguish by convention: guards are lowercase (`@positive`), macros can be anything but are typically verbs/actions (`@memoize`, `@json_serializable`). Both are resolved at parse time. If there's ambiguity, the guard definition takes precedence (guards are checked first, then macros). Document this resolution rule.

**Step 3: Commit**

```
git add Opal.md
git commit -m "Unify guard syntax and remove redundant type guard form"
```

---

### Task 5: Fix Package Manager / `with` Example (12)

**Files:**
- Modify: `Opal.md:3301-3339` (Package Manager subsection)

**Problem:** The nginx DSL example uses `with { ... }!` which has two issues: (1) the `with` block semantics aren't specified in the BNF, and (2) the `!` suffix on a block is ambiguous (Result propagation or method naming?). Also `import Roman@"https://..."` URL import syntax isn't specified.

**Step 1: Rewrite the package manager section**

Keep the CLI examples. For the DSL example, clarify that `with` takes a dict of config values and returns the configured object. Remove URL imports (not yet specified). Clean up `!` usage:

```opal
# Using with for DSL-style configuration blocks
import Nginx

my_site = Nginx.create with {
  user:              "www www",
  worker_processes:  5,
  error_log:         "logs/error.log",
  pid:               "logs/nginx.pid"
}

my_site.http with {
  index:        "index.html index.htm index.php",
  default_type: "application/octet-stream"
}

my_site.http.server with {
  listen:      80,
  server_name: "domain.com",
  access_log:  "logs/domain.access.log main"
}

my_site.serve!
```

Add BNF rule for `with` blocks:

```bnf
<with_expr> ::= <expression> "with" <dict>
```

**Step 2: Add `with` to BNF (section 3)**

Add `<with_expr>` to the `<expression>` production.

**Step 3: Commit**

```
git add Opal.md
git commit -m "Fix package manager examples and add with-block BNF"
```

---

## Phase 2: Import/Module System

**Status:** Needs brainstorming — multiple design decisions required.

### Design Decisions Needed

1. **Import syntax variations:** How many forms?
   - `import Module` (whole module)
   - `import Module.{a, b, c}` (selective)
   - `import Module as Alias` (aliased)
   - `import Module.SubModule` (nested)
   - Combination?

2. **File-to-module mapping:** One file = one module? Or module declaration independent of files? Nested modules = nested directories?

3. **Circular dependencies:** Allowed? Compile-time error? Lazy resolution?

4. **Re-exports:** Can a module re-export symbols from another module?

5. **Package vs module:** Is a "package" just a top-level module with a manifest? Or a separate concept?

6. **Relative imports:** `import .sibling` or always absolute?

7. **Selective import and aliasing interaction:** `import Module.{Foo as Bar, Baz}`?

### Task 6: Brainstorm Import/Module System

**Process:** Use the brainstorming skill workflow:
1. Present design questions above to user
2. Show 2-3 approaches for each decision with trade-offs
3. Get approval on each section
4. Write feature doc to `docs/features/imports-and-modules.md`

### Task 7: Integrate Import/Module System into Opal.md

**Files:**
- Modify: `Opal.md` section 6.4 (expand from ~35 lines to ~150 lines)
- Modify: `Opal.md` BNF (add `import_stmt`, `module_def` refinements)
- Modify: `Opal.md` section 11 stdlib table (if module organization changes)

**Integration checklist:**
- [ ] Add BNF rules for all import forms
- [ ] Expand section 6.4 with: basic import, selective import, aliased import, nested modules, file mapping, re-exports
- [ ] Update all existing examples that use `import` to be consistent with new spec
- [ ] Add cross-reference link to feature doc
- [ ] Commit

---

## Phase 3: Class Construction & Inheritance

**Status:** Needs brainstorming — the `needs`/`:init` relationship needs clarification.

### Design Decisions Needed

1. **`needs` vs `:init`:** When to use which? `needs` is for DI dependencies, `:init` is for construction logic? Can you have both? What's the order?

2. **Single vs multiple inheritance:** Currently shows single (`<`). Confirm this is intentional. If single, do protocols fill the multiple-inheritance gap?

3. **Abstract classes:** Does Opal have them? Or do protocols serve this purpose entirely?

4. **`super` semantics:** How does `super()` work? Only in `:init`? In any overridden method? What if the parent uses `needs`?

5. **Constructor inheritance:** Does a subclass inherit the parent's `needs`? Must it redeclare them?

6. **`needs` + inheritance interaction:** If `class Dog < Animal` and `Animal` has `needs logger::Logger`, does `Dog.new()` require `logger:`?

### Task 8: Brainstorm Class Construction & Inheritance

**Process:** Same brainstorming workflow. This may be one feature doc or two (`docs/features/classes-and-inheritance.md`).

### Task 9: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` section 6.3 (expand from ~55 lines to ~150 lines)
- Modify: `Opal.md` BNF (refine `class_def`, add `abstract_class_def` if needed)
- Modify: `Opal.md` section 9.1 (clarify `needs` interaction with inheritance)

**Integration checklist:**
- [ ] Expand section 6.3 with: construction order, `needs` vs `:init`, `super`, abstract classes (or explain why protocols replace them), inheritance rules
- [ ] Update BNF if new constructs added
- [ ] Ensure 9.1 (DI) cross-references the class construction rules
- [ ] Add cross-reference link to feature doc
- [ ] Commit

---

## Phase 4: Testing Framework

**Status:** Needs brainstorming — signature feature for "batteries included" philosophy.

### Design Decisions Needed

1. **Test file convention:** `.topl` files are mentioned. How are they discovered? Convention-based (mirror `src/` with `tests/`)? Or explicit?

2. **Test structure:** Use macros (`@test`, `@describe` from section 10) or a Test module API (`Test.describe`, `Test.it` from section 11)? Or both (macros are sugar for the module API)?

3. **Assertions:** What built-in assertions? `assert_eq`, `assert_ne`, `assert_true`, `assert_raises`, `assert_match`?

4. **Lifecycle:** `before_each`, `after_each`, `before_all`, `after_all`? Or simpler?

5. **Mocking:** How does `Mock` module work? Protocol-based mocking (swap implementations via `needs`)? Or method-level stubbing?

6. **Fixtures:** Built-in fixture system? Or just use `before_each`?

7. **Test runner:** `opal test` CLI? What output format? Filtering by name/tag?

### Task 10: Brainstorm Testing Framework

**Process:** Same brainstorming workflow. Write to `docs/features/testing.md`.

### Task 11: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` — add new section (likely 12.x or a new top-level section before Tooling)
- Modify: `Opal.md` section 11 (expand Test, Mock entries in stdlib table)
- Modify: `Opal.md` section 12 (add `opal test` CLI)

**Integration checklist:**
- [ ] New section: test file convention, test structure, assertions, lifecycle, mocking
- [ ] Reconcile with section 10.4 (test framework macro example)
- [ ] Update stdlib table with specific Test/Mock APIs
- [ ] Add `opal test` to tooling section
- [ ] Add cross-reference link to feature doc
- [ ] Commit

---

## Phase 5: Pattern Matching Depth

**Status:** Needs brainstorming — current section is thin.

### Design Decisions Needed

1. **Nested patterns:** `case (x, (a, b))` — already shown for destructuring but not in match. Confirm it works identically.

2. **Or-patterns:** `case 1 | 2 | 3` — can you match multiple values in one case arm?

3. **`as` bindings in patterns:** `case Shape.Circle(r) as shape` — bind the whole matched value while also destructuring?

4. **Custom class patterns:** Beyond enums, can `match` destructure regular classes? `case Person(name, age)` or only via protocol?

5. **Literal patterns:** Strings, symbols, booleans, null — all matchable?

6. **Nested enum patterns:** `case Result.Ok(Option.Some(value))` — arbitrarily nested?

7. **List patterns:** `case [1, 2, _]` or `case [head | tail]` — same as destructuring?

### Task 12: Brainstorm Pattern Matching

**Process:** Same workflow. This may not need a separate feature doc if the decisions are straightforward — could be a direct expansion of section 5.3.

### Task 13: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` section 5.3 (expand from ~35 lines to ~100 lines)
- Modify: `Opal.md` BNF (expand `<case_clause>` and `<pattern>` productions)

**Integration checklist:**
- [ ] Expand `<pattern>` BNF to cover all forms: literals, types, destructuring, nested, or-patterns, as-bindings, guards
- [ ] Add examples for each pattern form
- [ ] Cross-reference with: enum exhaustiveness (6.9), destructuring (4.7), Result matching (7.1)
- [ ] Commit

---

## Phase 6: Closures & Visibility

Two smaller topics that can be brainstormed together.

### Design Decisions — Closures

1. **Capture semantics:** By value or by reference? Mutable capture?
2. **Closure types in type system:** Can you annotate a closure type? `|Int32| -> String`?
3. **Move semantics:** Any concept of "move into closure" or always shared?

### Design Decisions — Visibility

1. **Module-level visibility:** Can functions/classes in a module be private (not exported)?
2. **Field visibility:** Are `needs` fields public by default? Can they be private?
3. **Visibility + protocols:** Does implementing a protocol force methods to be public?
4. **Visibility + retroactive conformance:** Can retroactive conformance add public methods?

### Task 14: Brainstorm Closures & Visibility

**Process:** Same workflow. May produce one or two feature docs, or just direct Opal.md expansion if decisions are simple.

### Task 15: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` section 6.1 (add capture semantics, ~30 extra lines)
- Modify: `Opal.md` section 6.5 (expand from ~30 lines to ~80 lines)

**Integration checklist:**
- [ ] Add closure capture rules to 6.1
- [ ] Add closure type syntax to type system (6.2) if decided
- [ ] Expand 6.5 with module visibility, field visibility, interaction with protocols
- [ ] Update BNF if new visibility keywords added
- [ ] Commit

---

## Phase 7: Standard Library, Strings, Numerics

Three related topics about core data types and their APIs.

### Design Decisions — String Operations

1. **Encoding:** UTF-8 internally? Or abstract over encoding?
2. **Core methods:** What's the minimum API? `split`, `join`, `trim`, `replace`, `starts_with?`, `ends_with?`, `contains?`, `to_upper`, `to_lower`, `slice`?
3. **String vs Char:** Indexing a string returns a Char? Or a single-char String?

### Design Decisions — Numeric Semantics

1. **Overflow:** Wrap? Raise? Promote to bigger type?
2. **Division:** `5 / 2` = `2` (integer) or `2.5` (float)? Separate operators?
3. **BigInt/BigDecimal:** In stdlib? Auto-promoted?

### Design Decisions — Standard Library API

1. **Depth of spec:** Do we spec every method signature? Or just the module purpose + key methods?
2. **IO API:** `print`, `println`, `read_line`, `read_all`?
3. **File API:** `read`, `write`, `exists?`, `delete`, `list_dir`?
4. **Collection methods:** Canonical list: `map`, `filter`, `reduce`, `each`, `find`, `any?`, `all?`, `sort`, `reverse`, `flatten`, `zip`, `take`, `drop`, `group_by`?

### Task 16: Brainstorm String, Numeric, and Stdlib APIs

**Process:** Same workflow. This could produce `docs/features/standard-library.md` or direct expansion.

### Task 17: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` section 4.3.5 (add string methods)
- Modify: `Opal.md` section 4.3.3 (add numeric semantics)
- Modify: `Opal.md` section 4.5 (expand collection methods)
- Modify: `Opal.md` section 11 (expand stdlib from table to actual API descriptions)

**Integration checklist:**
- [ ] Add string method examples after string literal section
- [ ] Add numeric overflow/division rules
- [ ] Expand collection section with method signatures
- [ ] Expand stdlib section with at least key methods per module
- [ ] Commit

---

## Phase 8: `with` Keyword Semantics & FFI

### Design Decisions — `with` Blocks

1. **Semantics:** Does `with` pass a dict to a method? Or does it create a scope?
2. **Nesting:** `a with { b: c with { d: e } }` — how does this work?
3. **Return value:** Does `expr with {...}` return the configured object?

### Design Decisions — FFI

1. **Target runtime:** What will Opal run on? This determines FFI options.
2. **FFI syntax:** Keywords? Or stdlib function?
3. **Scope:** Is this even worth speccing now, or leave as TBD?

### Task 18: Brainstorm `with` and FFI

**Process:** `with` is likely a quick decision. FFI may remain TBD depending on runtime decisions.

### Task 19: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` BNF (formalize `with_expr` if not done in Task 5)
- Modify: `Opal.md` section 2 Facts table (update FFI answer if decided)

---

## Phase 9: Packaging & Tooling

### Design Decisions

1. **Package manifest format:** TOML (like Cargo/Poetry)? Or Opal-native?
2. **Lock file:** Yes? Format?
3. **Version resolution:** Semantic versioning? How are conflicts resolved?
4. **`opal fmt`:** Built-in formatter? What style?
5. **`opal test` / `opal bench`:** CLI integration

### Task 20: Brainstorm Packaging & Tooling

### Task 21: Integrate into Opal.md

**Files:**
- Modify: `Opal.md` section 12 (expand from ~70 lines to ~150+ lines)

---

## Execution Order Summary

| Phase | Tasks | Type | Estimated Effort |
|---|---|---|---|
| 1: Fix Inconsistencies | 1-5 | Direct edits | Small — exact changes known |
| 2: Import/Module System | 6-7 | Brainstorm + integrate | Medium — foundational |
| 3: Classes & Inheritance | 8-9 | Brainstorm + integrate | Medium — clarifying existing |
| 4: Testing Framework | 10-11 | Brainstorm + integrate | Medium — new section |
| 5: Pattern Matching | 12-13 | Brainstorm + integrate | Small-Medium |
| 6: Closures & Visibility | 14-15 | Brainstorm + integrate | Small |
| 7: Stdlib/Strings/Numerics | 16-17 | Brainstorm + integrate | Medium-Large |
| 8: `with` & FFI | 18-19 | Brainstorm + integrate | Small |
| 9: Packaging & Tooling | 20-21 | Brainstorm + integrate | Small-Medium |

**Dependencies:**
- Phase 1 has no dependencies — do first
- Phase 2 (imports) should come before Phase 4 (testing) since test files depend on the module system
- Phase 3 (classes) should come before Phase 6 (visibility) since visibility is about classes
- All other phases are independent

**After each brainstorm phase:** Update CLAUDE.md feature table if a new feature doc is created.
