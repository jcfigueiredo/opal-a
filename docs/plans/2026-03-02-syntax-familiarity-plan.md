# Syntax Familiarity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Apply 5 syntax changes across the entire Opal spec to align with Python/Ruby/Rust conventions.

**Architecture:** Each task is one syntax change applied globally across all 33 spec files (Opal.md hub + 31 docs + CLAUDE.md). Changes are applied in dependency order: generics first (changes type syntax in definitions), then type annotations (changes the separator), then imports, then function types (depends on new generics syntax for `Fn`), then closures last.

**Tech Stack:** Markdown files only. No code, no tests.

---

## Context

**Design doc:** `docs/plans/2026-03-02-syntax-familiarity-design.md`

**Files to modify:** 33 total — `Opal.md`, `CLAUDE.md`, and all 31 files under `docs/01-basics/` through `docs/appendix/` (excluding `docs/plans/`).

**The 5 changes in order:**

1. Generics: `List(T)` → `List[T]` — affects BNF, type definitions, usage everywhere
2. Type annotations: `::` → `:` — affects every typed parameter, variable, and `needs` field
3. Imports: `import X.{a, b}` → `from X import a, b` — affects BNF and all import examples
4. Function types: `|Type| -> Type` → `Fn(Type) -> Type` — affects type annotations for closures
5. Closures: drop `fn` keyword form — affects BNF and closure examples

**Critical caution for Change 2 (:: → :):** The `::` replacement must be surgical. NOT all `::` in the files are type annotations. Watch for:
- `Rust's ::` in prose text (e.g., "Rust uses `::` for path resolution")
- `::` inside code that's showing Rust/Haskell comparisons
- The BNF grammar rule names like `<type_expr>` that reference `"::"` as a literal token

The BNF itself must change `"::"` to `":"` in the `<param>` and `<needs_decl>` rules.

---

### Task 1: Generics — `(T)` → `[T]` everywhere

**Files:** All 33 spec files (25+ contain generics)

**What to change:**

In the BNF grammar (Opal.md):
- `<type_params>` rule: change `"(" <type_params> ")"` to `"[" <type_params> "]"`
- All BNF rules that reference generic params: `<class_def>`, `<enum_def>`, `<model_def>`, `<settings_def>`, `<type_alias>`
- `<type_expr>` rule: change `TYPE "(" <type_args> ")"` to `TYPE "[" <type_args> "]"`

In code examples throughout ALL files:
- `List(T)` → `List[T]`
- `Dict(K, V)` → `Dict[K, V]`
- `Stack(T)` → `Stack[T]`
- `Option(T)` → `Option[T]`
- `Result(T, E)` → `Result[T, E]`
- `Iterator(T)` → `Iterator[T]`
- `Collection(T)` → `Collection[T]`
- `Set(T implements Hashable)` → `Set[T implements Hashable]`
- `Cache(K, V)` → `Cache[K, V]`
- `Pair(A, B)` → `Pair[A, B]`
- `SortedPair(T implements Comparable)` → `SortedPair[T implements Comparable]`
- Any other `ClassName(TypeParam)` patterns in type positions

**Be careful NOT to change:**
- Constructor calls: `Stack.new(items: [])` — these stay with parens
- Constructor shorthand: `User(name: "claudio")` — these stay with parens
- Function calls: `Option.Some(value: 42)` — these stay with parens
- Enum variant construction: `Shape.Circle(radius: 5.0)` — these stay with parens
- The `Fn(Type)` function type syntax (that comes in Task 4)

**How to distinguish:** Generic type parameters follow a PascalCase type name and contain other PascalCase type names or single uppercase letters: `List(T)`, `Dict(String, Int32)`, `Result(T, E)`. Constructor calls contain snake_case value arguments: `User(name: "x")`, `Shape.Circle(5.0)`.

**Commit:** `"Change generics syntax: parentheses to square brackets"`

---

### Task 2: Type Annotations — `::` → `:` everywhere

**Files:** All 33 spec files (25 contain `::`)

**What to change in BNF (Opal.md):**
- `<param>` rule: change `IDENTIFIER "::" TYPE` to `IDENTIFIER ":" TYPE`
- `<needs_decl>` rule: change `"needs" IDENTIFIER "::" TYPE` to `"needs" IDENTIFIER ":" TYPE`
- `<where_field>` rule: change `"needs" IDENTIFIER "::" TYPE` to `"needs" IDENTIFIER ":" TYPE`
- Any other BNF rules with `"::"` for type annotation

**What to change in code examples:**
- `name::String` → `name: String`
- `a::Int32` → `a: Int32`
- `needs db::Database` → `needs db: Database`
- `items::List[T]` → `items: List[T]` (note: already using `[T]` from Task 1)
- `result::String | Int32` → `result: String | Int32`
- `case n::Int32` → `case n: Int32` (in pattern matching)

**Be careful NOT to change:**
- Prose text that mentions `::` as a syntax element (update those to describe the new `:` syntax)
- The `"::"` literal in BNF — change it TO `":"` (this IS a change, not a skip)
- Named arguments at call sites already use `:` — don't double up

**Special attention to pattern matching:** `case n::Int32` becomes `case n: Int32`. This looks like a named argument but in match context it's a type pattern. The parser distinguishes by position (inside `case` = type pattern). Note this in prose if not already explained.

**Commit:** `"Change type annotations: double colon to single colon"`

---

### Task 3: Imports — add `from/import` form

**Files:** Opal.md (BNF + hub), `docs/03-functions-and-types/modules-and-imports.md`, `docs/07-metaprogramming/metaprogramming.md`, and any other files with import examples

**What to change in BNF (Opal.md):**

Replace the current import rules:
```bnf
<import_stmt>   ::= "import" <module_path>
                   | "import" <module_path> "as" IDENTIFIER
                   | "import" <module_path> ".{" <import_list> "}"
```

With:
```bnf
<import_stmt>   ::= "import" <module_path>
                   | "import" <module_path> "as" IDENTIFIER
                   | "from" <module_path> "import" <import_list>
                   | "from" <module_path> "import" "(" <import_list> ")"
```

Update the export rule:
```bnf
<export_stmt>   ::= "export" <import_list> "from" <module_path>
```

**What to change in code examples:**
- `import Math.{abs, max}` → `from Math import abs, max`
- `import Math.{abs, max as maximum}` → `from Math import abs, max as maximum`
- `import OpalWeb.{get, post}` → `from OpalWeb import get, post`
- `export Router.{get, post, put, delete}` → `export get, post, put, delete from Router`
- `export Middleware.{use}` → `export use from Middleware`

**Unchanged:**
- `import Math` — stays
- `import Math.Vector` — stays
- `import Math as M` — stays
- `import Math.Vector as Vec` — stays

**Add multi-line import example** somewhere in modules-and-imports.md:
```opal
from Math import (
  sin, cos, tan,
  sqrt, abs, max,
  PI, E
)
```

**Commit:** `"Change imports: brace-style to from/import syntax"`

---

### Task 4: Function Types — `|Type| -> Type` → `Fn(Type) -> Type`

**Files:** Opal.md (BNF), `docs/03-functions-and-types/functions-and-closures.md`, `docs/03-functions-and-types/type-system.md`, and any file with function type annotations

**What to change in BNF (Opal.md):**

In `<type_expr>`, change:
```bnf
| "|" <type_list> "|" "->" <type_expr>
```
To:
```bnf
| "Fn" "(" <type_list> ")" "->" <type_expr>
```

**What to change in code examples:**
- `|Int32| -> Int32` → `Fn(Int32) -> Int32`
- `|Request, Response| -> Null` → `Fn(Request, Response) -> Null`
- `|Int32| -> Bool` → `Fn(Int32) -> Bool`
- `transform: |Int32| -> Int32` → `transform: Fn(Int32) -> Int32` (with single colon from Task 2)
- `type Handler = |Request, Response| -> Null` → `type Handler = Fn(Request, Response) -> Null`

**Do NOT change:**
- Closure *syntax*: `|x| x * 2` stays — pipes are for closure params, not type annotations
- Union types: `Int32 | String` stays — pipe means union

**Commit:** `"Change function types: pipe-delimited to Fn() syntax"`

---

### Task 5: Closures — drop `fn` keyword form

**Files:** Opal.md (BNF), `docs/03-functions-and-types/functions-and-closures.md`

**What to change in BNF (Opal.md):**

Remove these two `fn` lambda forms from `<lambda>`:
```bnf
| "fn" "(" <params> ")" <expression> "end"
| "fn" "(" <params> ")" NEWLINE <block> "end"
```

Keep the remaining forms:
```bnf
<lambda>        ::= "|" <params> "|" <expression>
                   | "|" <params> "|" NEWLINE <block> "end"
                   | "do" <expression> "end"
                   | "do" NEWLINE <block> "end"
                   | "do" "|" <params> "|" NEWLINE <block> "end"
```

**What to change in code examples:**

In `functions-and-closures.md`, remove the "Named Closures with `fn`" section entirely. Update the "when to use which" guidance:
- `|params| expr` — inline closures passed directly to functions
- `do |params| ... end` — multi-line closures, stored function values, trailing blocks
- `do ... end` — no-arg closures

Replace any `fn(...)` examples with `do |...| ... end`:
```opal
# Before
handler = fn(request, response)
  user = authenticate(request)
  response.json(data)
end

# After
handler = do |request, response|
  user = authenticate(request)
  response.json(data)
end
```

**Check all other files** for any `fn(` usage in code examples and replace.

**Commit:** `"Drop fn closure form: keep pipes + do...end only"`

---

### Task 6: Update CLAUDE.md and summary tables

**Files:** `CLAUDE.md`, and any summary/keyword tables across docs

**What to change in CLAUDE.md:**
- Update `**:: for type annotations**` rule to `**: for type annotations**: name: String`
- Update any mentions of `::` syntax
- Update any mentions of generics syntax
- Update any mentions of import syntax
- Add note about `Fn` type for function annotations
- Remove `fn` from closure forms list if mentioned

**What to change in doc summary tables:**
- Any "New Keywords" tables that mention `::`, `fn`, or old import syntax
- The type system summary table
- The functions-and-closures summary (if it has a "when to use which" for three forms)

**Commit:** `"Update CLAUDE.md and summary tables for new syntax"`

---

### Task 7: Final audit

**Files:** All 33 spec files

**Step 1:** Search for any remaining `::` in code blocks that should be `:`:
```bash
grep -n '::' Opal.md docs/**/*.md | grep -v 'docs/plans/' | grep -v '# =>' | head -40
```
Some `::` may legitimately remain in prose explaining the old syntax or in the BNF `"::"` token removal. But no code examples should have `::` for type annotations.

**Step 2:** Search for any remaining `(T)` generics that should be `[T]`:
```bash
grep -n '[A-Z][a-z]*([A-Z][a-z]*\|[A-Z],\|[A-Z])' docs/**/*.md Opal.md | grep -v 'docs/plans/' | head -20
```

**Step 3:** Search for any remaining `.{` import syntax:
```bash
grep -n '\.{' docs/**/*.md Opal.md | grep -v 'docs/plans/'
```

**Step 4:** Search for any remaining `fn(` closure syntax:
```bash
grep -n ' fn(' docs/**/*.md Opal.md | grep -v 'docs/plans/'
```

**Step 5:** Search for any remaining pipe-delimited function types:
```bash
grep -n '|[A-Z].*| ->' docs/**/*.md Opal.md | grep -v 'docs/plans/'
```

Fix any remaining instances found.

**Commit:** `"Audit and fix remaining old syntax instances"`

---

## Dependency Graph

```
Task 1 (generics [T])
  └─> Task 2 (type : instead of ::)
        └─> Task 3 (from/import)
              └─> Task 4 (Fn() types)
                    └─> Task 5 (drop fn closures)
                          └─> Task 6 (CLAUDE.md + tables)
                                └─> Task 7 (audit)
```

Tasks are strictly sequential — each builds on the previous syntax state.
