# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Opal (Opinionated Programming Algorithmic Language) is a programming language in the design/specification phase. There is no compiler or interpreter yet — the project is a language specification document (`Opal.md`) and supporting design documents.

## Repository Structure

- **`Opal.md`** — The complete language specification (~3000 lines). This is the primary artifact. Organized into 13 sections from foundational to generalist: Philosophy, Facts, BNF Grammar, Basics, Control Flow, Functions & Types, Error Handling, Concurrency, Software Engineering Patterns, Metaprogramming, Standard Library, Tooling, Pretotyping, plus appendices.
- **`docs/features/`** — Deep-dive feature documentation with design rationale, trade-offs, and extended examples. Linked from Opal.md at the relevant sections.

## Key Language Design Rules

These rules are enforced across all examples and specifications. Violating them in code examples is a bug:

- **Single quotes for chars only**: `'a'`, `'ñ'` — never for multi-character strings
- **Double quotes for strings**: `"hello"`, `f"hi {name}"`, `r"raw\n"`, `t"template {x}"`
- **`with` keyword is ONLY for DSL config blocks** (nginx-style). Object creation uses `.new()` with named args. String interpolation uses f-strings.
- **`defaults` keyword** (not `with`) for null object variant creation
- **`::` for type annotations**: `name::String`
- **`.` prefix for instance variables**: `.name`, `.age`
- **`()` for tuples, `{}` for dicts**: Empty tuple `()`, empty dict `{:}`
- **`###` for multiline comments** (not `/* */` or `""" """`)

## Design Process

New language features follow this workflow:
1. Brainstorm with clarifying questions, approach comparison, and design approval
2. Write feature document to `docs/features/<topic>.md`
3. Integrate into `Opal.md` (update BNF, add section, update stdlib table) and link to the feature doc

When updating Opal.md, always check that changes are consistent across: BNF grammar (section 3), the relevant syntax section, the stdlib table (section 11), and the philosophy line (section 1) if a new first-class concept is added.

## Feature Documents

| File | Covers |
|---|---|
| `docs/features/type-system.md` | Generics, constraints, union types, aliases, nominal typing, retroactive conformance |
| `docs/features/concurrency.md` | Four-layer model: actors, parallel, async/futures, supervisors |
| `docs/features/dependency-injection-and-events.md` | `needs` keyword, `event`/`emit`/`on`, optional Container |
| `docs/features/metaprogramming.md` | Julia-adapted quoting, hygienic macros, AST, subdomains |
| `docs/features/self-hosting-foundations.md` | Operator overloading, iterators, custom errors, destructuring, protocol defaults |
| `docs/features/enums-and-algebraic-types.md` | Enums, data-carrying variants, exhaustive matching, generic enums (Option, Result) |
| `docs/features/error-handling.md` | Two-track model: exceptions vs Result types, `!` operator, bridging |
| `docs/features/validation-and-settings.md` | `model` keyword, field validation with `where`, serialization, Settings loading |
| `docs/features/imports-and-modules.md` | Hybrid file-module mapping, import forms, re-exports, circular deps, packages |
| `docs/features/classes-and-inheritance.md` | Construction model (needs + init), single inheritance, super, inherited needs |
| `docs/features/testing.md` | Test structure, assertions, lifecycle hooks, mocking, test runner |
