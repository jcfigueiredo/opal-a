# Opal Spec Restructuring Design

## Goal

Break down the monolithic `Opal.md` (4,661 lines) into a hub document (~600-800 lines) plus focused topic docs organized in numbered directories under `docs/`. Merge existing `docs/features/` files into the new structure so each topic has a single authoritative source.

## Decisions

### Opal.md Becomes a Hub

Opal.md keeps three sections verbatim:
- **Design Philosophy** (~10 lines) — the soul of the language
- **Facts & Semantics** (~16 lines) — quick reference table
- **Formal Grammar (BNF)** (~249 lines) — the single canonical grammar

Everything else becomes **2-5 line summaries** with one short code example and a link to the full doc. Target: ~600-800 lines total.

### Directory Structure

```
Opal.md                          (hub document)
docs/
  01-basics/
    comments.md
    variables-and-assignment.md
    literals.md                  (numbers, strings, symbols, booleans, null)
    operators.md                 (merged with self-hosting-foundations operator section)
    collections.md               (lists, tuples, dicts, ranges, regex, comprehensions)
    destructuring.md             (merged with self-hosting-foundations destructuring section)
  02-control-flow/
    conditionals.md
    loops-and-iteration.md
    pattern-matching.md
  03-functions-and-types/
    functions-and-closures.md
    type-system.md               (merged: Opal.md 6.2 + existing type-system.md)
    classes-and-inheritance.md    (merged: Opal.md 6.3 + existing classes-and-inheritance.md)
    modules-and-imports.md       (merged: Opal.md 6.4 + existing imports-and-modules.md)
    visibility.md
    protocols.md                 (merged with self-hosting-foundations protocol defaults)
    multiple-dispatch.md
    iterators.md                 (merged with self-hosting-foundations iterator section)
    enums.md                     (merged: Opal.md 6.9 + existing enums-and-algebraic-types.md)
    models-and-settings.md       (merged: Opal.md 6.10 + 9.4 + existing validation-and-settings.md)
    ffi.md
  04-error-handling/
    error-handling.md            (merged: Opal.md 7.1 + existing error-handling.md)
    preconditions.md
    null-objects.md
  05-concurrency/
    concurrency.md               (merged: Opal.md 8 + existing concurrency.md)
  06-patterns/
    dependency-injection.md      (merged: Opal.md 9.1-9.2 + existing DI-and-events.md)
    specifications.md
  07-metaprogramming/
    metaprogramming.md           (merged: Opal.md 10 + existing metaprogramming.md)
  08-stdlib/
    stdlib.md
  09-tooling/
    tooling.md                   (merged: Opal.md 12 + existing testing.md)
  10-examples/
    pretotyping.md
  appendix/
    appendix.md
  plans/                         (unchanged)
```

### Merged Doc Structure

Each merged topic doc follows this template:

```markdown
# Topic Name

---

## Overview
2-3 sentences. What it is, why it exists.

---

## 1. Core Syntax / Rules
Specification content (from Opal.md). The "what."

## 2. Design Rationale
Design reasoning (from feature docs). The "why."

## 3. Extended Examples
Complex, real-world examples from both sources.

## Summary
Quick-reference table.
```

### Merge Rules

- When both sources have the same code example, keep the better/more complete one.
- Syntax rules (Opal.md) and design rationale (feature docs) sit in separate sections within the same file.
- Duplicate content is dropped — no redundancy.
- "New Keywords" summary tables preserved at the bottom.

### self-hosting-foundations.md Dissolution

This cross-cutting doc covers 5 topics. Each merges into the relevant section file:
- Operator overloading → `docs/01-basics/operators.md`
- Destructuring → `docs/01-basics/destructuring.md`
- Protocol defaults → `docs/03-functions-and-types/protocols.md`
- Iterator protocol → `docs/03-functions-and-types/iterators.md`
- Custom error types → `docs/04-error-handling/error-handling.md`

The file is then deleted.

### Migration Safety

- All internal links updated (Opal.md → new paths, cross-doc references).
- CLAUDE.md updated with new structure.
- Each major step is a separate commit.
- No content loss — every line ends up somewhere, only explicit deduplication.
- `docs/features/` deleted only after all content verified merged.
- `docs/plans/` untouched.
- Final audit: grep for broken links, verify completeness.
