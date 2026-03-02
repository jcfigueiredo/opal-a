# Spec Restructuring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Break the 4,661-line `Opal.md` into a ~600-800 line hub document plus ~30 focused topic docs organized in numbered directories, merging existing `docs/features/` content into the new structure.

**Architecture:** Each Opal.md section gets extracted into its own file under `docs/NN-topic/`. Existing feature docs are merged (spec content from Opal.md + rationale from feature docs → single authoritative file). The `self-hosting-foundations.md` cross-cutting doc is dissolved into 5 target files. Opal.md is rewritten as a hub with summaries and links.

**Tech Stack:** Markdown files only. No code, no tests.

---

## Context

**Source files:**
- `Opal.md` — 4,661 lines, 13 sections + appendix
- `docs/features/` — 11 files, ~4,200 lines total
- `docs/features/self-hosting-foundations.md` — 498 lines covering 5 cross-cutting topics

**Target structure:** See `docs/plans/2026-03-02-spec-restructuring-design.md` for the full directory layout.

**Merge template** for each topic doc:
```markdown
# Topic Name

---

## Overview
2-3 sentences.

---

## 1. [Spec content from Opal.md — the "what"]

## 2. Design Rationale [from feature docs — the "why"]

## 3. Extended Examples [best examples from both]

## Summary [quick-reference table]
```

**Key rule:** When both sources have the same code example, keep the more complete one. Drop pure duplicates. Preserve "New Keywords" summary tables.

---

### Task 1: Create directory structure

**Files:**
- Create: `docs/01-basics/` (directory)
- Create: `docs/02-control-flow/` (directory)
- Create: `docs/03-functions-and-types/` (directory)
- Create: `docs/04-error-handling/` (directory)
- Create: `docs/05-concurrency/` (directory)
- Create: `docs/06-patterns/` (directory)
- Create: `docs/07-metaprogramming/` (directory)
- Create: `docs/08-stdlib/` (directory)
- Create: `docs/09-tooling/` (directory)
- Create: `docs/10-examples/` (directory)
- Create: `docs/appendix/` (directory)

**Step 1: Create all directories**

```bash
mkdir -p docs/{01-basics,02-control-flow,03-functions-and-types,04-error-handling,05-concurrency,06-patterns,07-metaprogramming,08-stdlib,09-tooling,10-examples,appendix}
```

**Step 2: Commit**

```bash
git add docs/
git commit -m "Add directory structure for spec restructuring"
```

---

### Task 2: Extract 01-basics (simple files)

**Source:** `Opal.md` lines 291-673
**Files:**
- Create: `docs/01-basics/comments.md`
- Create: `docs/01-basics/variables-and-assignment.md`
- Create: `docs/01-basics/literals.md`

**Step 1: Create `docs/01-basics/comments.md`**

Extract from `Opal.md` section 4.1 (lines 291-304). This is a simple section, no feature doc to merge.

Content: Opal.md section 4.1 verbatim (single-line `#`, multiline `###`, inline comments). Add an Overview header.

**Step 2: Create `docs/01-basics/variables-and-assignment.md`**

Extract from `Opal.md` section 4.2 (lines 306-356). No feature doc to merge.

Content: Variables, `let` bindings, parallel assignment, naming conventions. Add an Overview header.

**Step 3: Create `docs/01-basics/literals.md`**

Extract from `Opal.md` section 4.3 (lines 357-673). No feature doc to merge.

Content: All literal types — null, booleans, numbers (with numeric semantics), strings (all quote styles, prefixes, escape sequences, methods), symbols (including symbol sets subsection). Add an Overview header.

This is the largest basics file (~317 lines) but it's all one topic — "what values look like in Opal."

**Step 4: Commit**

```bash
git add docs/01-basics/
git commit -m "Extract basics: comments, variables, literals from Opal.md"
```

---

### Task 3: Extract 01-basics (merged files)

**Source:** `Opal.md` lines 675-1079 + `docs/features/self-hosting-foundations.md`
**Files:**
- Create: `docs/01-basics/operators.md`
- Create: `docs/01-basics/collections.md`
- Create: `docs/01-basics/destructuring.md`
- Read: `docs/features/self-hosting-foundations.md` (for operator overloading and destructuring sections)

**Step 1: Read `docs/features/self-hosting-foundations.md`**

Identify the operator overloading section and the destructuring section. These merge into the new files.

**Step 2: Create `docs/01-basics/operators.md`**

Merge:
- `Opal.md` section 4.4 (lines 675-856) — arithmetic, comparison, logical, membership, assignment, operator overloading, pipe operator, null-safe chaining
- `self-hosting-foundations.md` operator overloading design rationale section

Structure:
1. Overview
2. Operator reference (arithmetic, comparison, logical, membership, assignment)
3. Operator overloading (spec from Opal.md + rationale from self-hosting-foundations)
4. Pipe operator
5. Null-safe chaining and null coalescing

**Step 3: Create `docs/01-basics/collections.md`**

Extract from `Opal.md` sections 4.5 + 4.6 (lines 857-1010). No feature doc to merge.

Content: Lists, tuples, dicts, ranges, regex, collection methods, comprehensions.

**Step 4: Create `docs/01-basics/destructuring.md`**

Merge:
- `Opal.md` section 4.7 (lines 1011-1079) — destructuring syntax
- `self-hosting-foundations.md` destructuring design rationale section

Structure:
1. Overview
2. Destructuring syntax (tuples, dicts, lists, function params, for loops, closures)
3. Design rationale (from self-hosting-foundations)

**Step 5: Commit**

```bash
git add docs/01-basics/
git commit -m "Extract basics: operators, collections, destructuring (merged with self-hosting-foundations)"
```

---

### Task 4: Extract 02-control-flow

**Source:** `Opal.md` lines 1081-1339
**Files:**
- Create: `docs/02-control-flow/conditionals.md`
- Create: `docs/02-control-flow/loops-and-iteration.md`
- Create: `docs/02-control-flow/pattern-matching.md`

**Step 1: Create `docs/02-control-flow/conditionals.md`**

Extract from `Opal.md` section 5.1 (lines 1083-1109). No feature doc.

Content: if/elsif/else, suffix form, ternary-style inline.

**Step 2: Create `docs/02-control-flow/loops-and-iteration.md`**

Extract from `Opal.md` section 5.2 (lines 1111-1140). No feature doc.

Content: while, for-in, with_index, break, next.

**Step 3: Create `docs/02-control-flow/pattern-matching.md`**

Extract from `Opal.md` section 5.3 (lines 1141-1339). No feature doc.

Content: All pattern forms (literals, ranges, variables, types, or-patterns, tuple/dict/list destructuring, enum patterns, nesting, guards, as-bindings), pattern summary table, exhaustive matching with symbol sets.

This is the largest control-flow file (~199 lines).

**Step 4: Commit**

```bash
git add docs/02-control-flow/
git commit -m "Extract control flow: conditionals, loops, pattern matching"
```

---

### Task 5: Extract 03-functions-and-types (part 1 — simple + type-system merge)

**Source:** `Opal.md` lines 1341-1995 + `docs/features/type-system.md` + `docs/features/classes-and-inheritance.md` + `docs/features/imports-and-modules.md`
**Files:**
- Create: `docs/03-functions-and-types/functions-and-closures.md`
- Create: `docs/03-functions-and-types/type-system.md`
- Create: `docs/03-functions-and-types/classes-and-inheritance.md`
- Create: `docs/03-functions-and-types/modules-and-imports.md`
- Create: `docs/03-functions-and-types/visibility.md`

**Step 1: Create `docs/03-functions-and-types/functions-and-closures.md`**

Extract from `Opal.md` section 6.1 (lines 1343-1491). No feature doc to merge.

Content: Function definitions, closures/lambdas, capture semantics, closure types, trailing blocks, `fn` keyword.

**Step 2: Create `docs/03-functions-and-types/type-system.md`**

Merge:
- `Opal.md` section 6.2 (lines 1492-1703) — core rules, generics, constraints, unions, type aliases, runtime introspection
- `docs/features/type-system.md` (444 lines) — same topics with design rationale, symbol sets section, retroactive conformance

Dedup carefully: both cover generics, constraints, unions, aliases. Keep Opal.md syntax as the spec section, feature doc reasoning as the rationale section. The feature doc's retroactive conformance and symbol sets sections may have more detail — keep those.

**Step 3: Create `docs/03-functions-and-types/classes-and-inheritance.md`**

Merge:
- `Opal.md` section 6.3 (lines 1704-1871) — classes, init, constructor shorthand, inheritance, construction order, inherited needs, super
- `docs/features/classes-and-inheritance.md` (230 lines) — same topics with rationale

**Step 4: Create `docs/03-functions-and-types/modules-and-imports.md`**

Merge:
- `Opal.md` section 6.4 (lines 1872-1995) — file-module mapping, import syntax, re-exports, packages
- `docs/features/imports-and-modules.md` (200 lines) — same topics with rationale

**Step 5: Create `docs/03-functions-and-types/visibility.md`**

Extract from `Opal.md` section 6.5 (lines 1996-2072). No feature doc.

Content: public/private/protected, class methods, module definitions, visibility summary table.

**Step 6: Commit**

```bash
git add docs/03-functions-and-types/
git commit -m "Extract functions-and-types part 1: functions, types, classes, modules, visibility"
```

---

### Task 6: Extract 03-functions-and-types (part 2 — protocols through FFI)

**Source:** `Opal.md` lines 2073-2644 + `docs/features/self-hosting-foundations.md` + `docs/features/enums-and-algebraic-types.md` + `docs/features/validation-and-settings.md`
**Files:**
- Create: `docs/03-functions-and-types/protocols.md`
- Create: `docs/03-functions-and-types/multiple-dispatch.md`
- Create: `docs/03-functions-and-types/iterators.md`
- Create: `docs/03-functions-and-types/enums.md`
- Create: `docs/03-functions-and-types/models-and-settings.md`
- Create: `docs/03-functions-and-types/ffi.md`
- Read: `docs/features/self-hosting-foundations.md` (for protocol defaults and iterator sections)
- Read: `docs/features/enums-and-algebraic-types.md`
- Read: `docs/features/validation-and-settings.md`

**Step 1: Create `docs/03-functions-and-types/protocols.md`**

Merge:
- `Opal.md` section 6.6 (lines 2073-2247) — protocol syntax, defaults, nominal typing, retroactive conformance, generic protocols
- `self-hosting-foundations.md` protocol defaults section — design rationale

**Step 2: Create `docs/03-functions-and-types/multiple-dispatch.md`**

Extract from `Opal.md` section 6.7 (lines 2248-2292). No feature doc.

Content: Dispatch by type, arity, preconditions. Resolution order.

**Step 3: Create `docs/03-functions-and-types/iterators.md`**

Merge:
- `Opal.md` section 6.8 (lines 2293-2373) — Iterable/Iterator protocols, custom collections, lazy sequences
- `self-hosting-foundations.md` iterator section — design rationale

**Step 4: Create `docs/03-functions-and-types/enums.md`**

Merge:
- `Opal.md` section 6.9 (lines 2374-2521) — enum syntax, exhaustive matching, methods, generic enums
- `docs/features/enums-and-algebraic-types.md` (334 lines) — same topics with rationale

**Step 5: Create `docs/03-functions-and-types/models-and-settings.md`**

Merge:
- `Opal.md` section 6.10 (lines 2522-2620) — model keyword, field validation, serialization
- `Opal.md` section 9.4 (lines 3571-3628) — settings model, loading, source priority
- `docs/features/validation-and-settings.md` (329 lines) — same topics with rationale

This is a three-way merge: two Opal.md sections + one feature doc → one file.

**Step 6: Create `docs/03-functions-and-types/ffi.md`**

Extract from `Opal.md` section 6.11 (lines 2621-2644). No feature doc.

Content: extern syntax, rules, placeholder status.

**Step 7: Commit**

```bash
git add docs/03-functions-and-types/
git commit -m "Extract functions-and-types part 2: protocols, dispatch, iterators, enums, models, FFI"
```

---

### Task 7: Extract 04-error-handling

**Source:** `Opal.md` lines 2646-2887 + `docs/features/error-handling.md` + `docs/features/self-hosting-foundations.md`
**Files:**
- Create: `docs/04-error-handling/error-handling.md`
- Create: `docs/04-error-handling/preconditions.md`
- Create: `docs/04-error-handling/null-objects.md`
- Read: `docs/features/error-handling.md` (256 lines)
- Read: `docs/features/self-hosting-foundations.md` (custom error types section)

**Step 1: Create `docs/04-error-handling/error-handling.md`**

Merge:
- `Opal.md` section 7.1 (lines 2648-2782) — exceptions, Result types, `!` operator, bridging
- `docs/features/error-handling.md` (256 lines) — two-track model rationale, decision matrix
- `self-hosting-foundations.md` custom error types section — error class design rationale

**Step 2: Create `docs/04-error-handling/preconditions.md`**

Extract from `Opal.md` section 7.2 (lines 2783-2840). No feature doc.

Content: `requires` keyword, reusable validators shared with model `where`.

**Step 3: Create `docs/04-error-handling/null-objects.md`**

Extract from `Opal.md` section 7.3 (lines 2841-2887). No feature doc.

Content: Null object pattern, `defaults` shorthand, usage example.

**Step 4: Commit**

```bash
git add docs/04-error-handling/
git commit -m "Extract error handling: errors, preconditions, null objects"
```

---

### Task 8: Extract 05-concurrency

**Source:** `Opal.md` lines 2889-3223 + `docs/features/concurrency.md`
**Files:**
- Create: `docs/05-concurrency/concurrency.md`
- Read: `docs/features/concurrency.md` (378 lines)

**Step 1: Create `docs/05-concurrency/concurrency.md`**

Merge:
- `Opal.md` section 8 (lines 2889-3223) — all 5 subsections: actors, parallel, async, supervisors, complete example
- `docs/features/concurrency.md` (378 lines) — design principles, four-layer rationale, detailed trade-offs

Both sources are comprehensive. The Opal.md version has the `receives` and message typing additions from the recent session. The feature doc has more design rationale. Merge them.

**Step 2: Commit**

```bash
git add docs/05-concurrency/
git commit -m "Extract concurrency (merged with feature doc)"
```

---

### Task 9: Extract 06-patterns + 07-metaprogramming

**Source:** `Opal.md` lines 3226-4270 + `docs/features/dependency-injection-and-events.md` + `docs/features/metaprogramming.md`
**Files:**
- Create: `docs/06-patterns/dependency-injection.md`
- Create: `docs/06-patterns/specifications.md`
- Create: `docs/07-metaprogramming/metaprogramming.md`
- Read: `docs/features/dependency-injection-and-events.md` (414 lines)
- Read: `docs/features/metaprogramming.md` (571 lines)

**Step 1: Create `docs/06-patterns/dependency-injection.md`**

Merge:
- `Opal.md` sections 9.1 + 9.2 (lines 3228-3528) — `needs`, Container, events (`event`/`emit`/`on`), complete DDD example
- `docs/features/dependency-injection-and-events.md` (414 lines) — design rationale, DDD patterns

**Step 2: Create `docs/06-patterns/specifications.md`**

Extract from `Opal.md` section 9.3 (lines 3530-3569). No feature doc.

Content: Specification pattern, composable business rules.

**Step 3: Create `docs/07-metaprogramming/metaprogramming.md`**

Merge:
- `Opal.md` section 10 (lines 3630-4270) — all 7 subsections: quoting, macros, annotations, AST, examples, self-hosting, subdomains
- `docs/features/metaprogramming.md` (571 lines) — same topics with design rationale

Both are comprehensive. Heavy dedup needed — check each subsection for which version is more complete.

**Step 4: Commit**

```bash
git add docs/06-patterns/ docs/07-metaprogramming/
git commit -m "Extract patterns and metaprogramming (merged with feature docs)"
```

---

### Task 10: Extract 08-stdlib, 09-tooling, 10-examples, appendix

**Source:** `Opal.md` lines 4272-4662 + `docs/features/testing.md`
**Files:**
- Create: `docs/08-stdlib/stdlib.md`
- Create: `docs/09-tooling/tooling.md`
- Create: `docs/10-examples/pretotyping.md`
- Create: `docs/appendix/appendix.md`
- Read: `docs/features/testing.md` (340 lines)

**Step 1: Create `docs/08-stdlib/stdlib.md`**

Extract from `Opal.md` section 11 (lines 4272-4321).

Content: Standard library module table + usage examples.

**Step 2: Create `docs/09-tooling/tooling.md`**

Merge:
- `Opal.md` section 12 (lines 4323-4555) — running, testing, scaffolding, docs, linter, formatter, package manager, CLI summary
- `docs/features/testing.md` (340 lines) — test structure, assertions, lifecycle hooks, mocking, test runner rationale

The testing content in Opal.md section 12 and `testing.md` overlap significantly. Merge testing into its own section within tooling, or as a major subsection.

**Step 3: Create `docs/10-examples/pretotyping.md`**

Extract from `Opal.md` section 13 (lines 4557-4590).

Content: Flask vs Opal comparison.

**Step 4: Create `docs/appendix/appendix.md`**

Extract from `Opal.md` appendix (lines 4591-4662).

Content: Links, topics, tutorials, references, ideas.

**Step 5: Commit**

```bash
git add docs/08-stdlib/ docs/09-tooling/ docs/10-examples/ docs/appendix/
git commit -m "Extract stdlib, tooling, examples, appendix"
```

---

### Task 11: Rewrite Opal.md as hub document

**Files:**
- Modify: `Opal.md` (rewrite from 4,661 lines to ~600-800 lines)

**Step 1: Read all newly created docs to verify they exist and get their exact paths**

Glob for `docs/0*/**/*.md` and `docs/appendix/*.md` to confirm everything is in place.

**Step 2: Rewrite Opal.md**

The new Opal.md keeps three sections verbatim:
- **Section 1: Design Philosophy** (lines 9-18) — keep as-is
- **Section 2: Facts & Semantics** (lines 20-35) — keep as-is
- **Section 3: Formal Grammar (BNF)** (lines 37-285) — keep as-is

Everything else becomes summaries. For each former section:
- 2-5 line description
- One short code example (the simplest/most representative one)
- Link: `> See [Topic Name](docs/NN-topic/file.md) for the full specification.`

The hub should follow this structure:

```markdown
# Opal — Opinionated Programming Algorithmic Language

[intro paragraph]

---

## 1. Design Philosophy
[kept verbatim]

---

## 2. Facts & Semantics
[kept verbatim]

---

## 3. Formal Grammar (BNF Excerpt)
[kept verbatim]

---

## 4. Basics
### 4.1 Comments — [2 lines + link]
### 4.2 Variables — [3 lines + example + link]
### 4.3 Literals — [3 lines + example + link]
### 4.4 Operators — [3 lines + example + link]
### 4.5 Collections — [3 lines + example + link]
### 4.6 Destructuring — [2 lines + example + link]

## 5. Control Flow
### 5.1 Conditionals — [2 lines + link]
### 5.2 Loops — [2 lines + link]
### 5.3 Pattern Matching — [3 lines + example + link]

## 6. Functions & Types
[11 subsections, each 2-5 lines + link]

## 7. Error Handling & Safety
[3 subsections, each 2-5 lines + link]

## 8. Concurrency
[brief description + link]

## 9. Software Engineering Patterns
[3 subsections + links]

## 10. Metaprogramming
[brief description + link]

## 11. Standard Library
[table + link]

## 12. Tooling
[CLI table + link]

## 13. Pretotyping
[brief + link]

## Appendix
[link]
```

**Step 3: Verify the hub is 600-800 lines**

```bash
wc -l Opal.md
```

If over 800, trim examples. If under 600, the summaries may be too terse.

**Step 4: Commit**

```bash
git add Opal.md
git commit -m "Rewrite Opal.md as hub document (~700 lines)"
```

---

### Task 12: Update CLAUDE.md, delete docs/features/, final audit

**Files:**
- Modify: `CLAUDE.md`
- Delete: `docs/features/` (entire directory — 11 files)

**Step 1: Update CLAUDE.md**

Update the "Feature Documents" table to list the new structure:

```markdown
## Documentation Structure

| Directory | Covers |
|---|---|
| `docs/01-basics/` | Comments, variables, literals, operators, collections, destructuring |
| `docs/02-control-flow/` | Conditionals, loops, pattern matching |
| `docs/03-functions-and-types/` | Functions, type system, classes, modules, visibility, protocols, dispatch, iterators, enums, models/settings, FFI |
| `docs/04-error-handling/` | Exceptions, Result types, preconditions, null objects |
| `docs/05-concurrency/` | Actors, parallel, async/futures, supervisors |
| `docs/06-patterns/` | Dependency injection, events, specifications |
| `docs/07-metaprogramming/` | Quoting, macros, annotations, AST, subdomains |
| `docs/08-stdlib/` | Standard library reference |
| `docs/09-tooling/` | Testing, runner, formatter, linter, package manager |
| `docs/10-examples/` | Pretotyping / comparison examples |
| `docs/appendix/` | Links, references, ideas |
```

Update the "Design Process" section to reference the new paths instead of `docs/features/<topic>.md`.

**Step 2: Verify no broken links**

Search for any remaining references to `docs/features/` in the entire repo:

```bash
grep -r "docs/features/" --include="*.md" .
```

All should be gone. If any remain (in plan docs, etc.), update or note them.

**Step 3: Verify all old feature doc content is merged**

Check that every file in `docs/features/` has been consumed:
- `type-system.md` → merged into `docs/03-functions-and-types/type-system.md`
- `classes-and-inheritance.md` → merged into `docs/03-functions-and-types/classes-and-inheritance.md`
- `imports-and-modules.md` → merged into `docs/03-functions-and-types/modules-and-imports.md`
- `self-hosting-foundations.md` → dissolved into operators.md, destructuring.md, protocols.md, iterators.md, error-handling.md
- `enums-and-algebraic-types.md` → merged into `docs/03-functions-and-types/enums.md`
- `validation-and-settings.md` → merged into `docs/03-functions-and-types/models-and-settings.md`
- `error-handling.md` → merged into `docs/04-error-handling/error-handling.md`
- `concurrency.md` → merged into `docs/05-concurrency/concurrency.md`
- `dependency-injection-and-events.md` → merged into `docs/06-patterns/dependency-injection.md`
- `metaprogramming.md` → merged into `docs/07-metaprogramming/metaprogramming.md`
- `testing.md` → merged into `docs/09-tooling/tooling.md`

**Step 4: Delete docs/features/**

```bash
rm -rf docs/features/
```

**Step 5: Verify new doc count and total line count**

```bash
find docs/ -name "*.md" -not -path "docs/plans/*" | wc -l
# Expected: ~30 files

find docs/ -name "*.md" -not -path "docs/plans/*" -exec wc -l {} + | tail -1
# Expected: ~5,000-6,000 total lines (merged content from 4,661 + 4,200 minus deduplication)
```

**Step 6: Commit**

```bash
git add -A
git commit -m "Complete restructuring: update CLAUDE.md, remove docs/features/"
```

---

## Dependency Graph

```
Task 1 (dirs)
  └─> Tasks 2-10 (all extraction tasks, can run sequentially)
        └─> Task 11 (hub rewrite — needs all docs to exist)
              └─> Task 12 (cleanup — needs hub to be done)
```

Tasks 2-10 are sequentially ordered but independent in content. Each reads from the original Opal.md (not yet modified) and creates new files.

Task 11 must run after all extraction tasks because it needs to link to the new files.

Task 12 must run last because it deletes the old structure.
