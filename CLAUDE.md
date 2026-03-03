# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Opal (Opinionated Programming Algorithmic Language) is a programming language in the design/specification phase. There is no compiler or interpreter yet — the project is a language specification document (`Opal.md`) and supporting design documents.

## Repository Structure

- **`Opal.md`** — The complete language specification (~3000 lines). This is the primary artifact. Organized into 13 sections from foundational to generalist: Philosophy, Facts, BNF Grammar, Basics, Control Flow, Functions & Types, Error Handling, Concurrency, Software Engineering Patterns, Metaprogramming, Standard Library, Tooling, Pretotyping, plus appendices.
- **`docs/`** — Numbered directories (`01-basics/` through `10-examples/` plus `appendix/`) containing focused topic documents with design rationale, trade-offs, and extended examples. Each directory maps to a section of the language spec.

## Key Language Design Rules

These rules are enforced across all examples and specifications. Violating them in code examples is a bug:

- **Both quote styles for strings**: `"hello"` and `'hello'` are identical. Use whichever avoids escaping.
- **String prefixes use either quote style**: `f"hi {name}"`, `f'hi {name}'`, `r"raw\n"`, `t"template {x}"`
- **`with` keyword is ONLY for DSL config blocks** (nginx-style). Object creation uses `.new()` with named args. String interpolation uses f-strings.
- **`defaults` keyword** (not `with`) for null object variant creation
- **`:` for type annotations**: `name: String`
- **`[T]` for generics**: `List[T]`, `Dict[K, V]`, `Option[T]`
- **`from X import Y` for selective imports**: `from Math import sqrt, PI`
- **`Fn(Type) -> Type` for function types**: `Fn(Int32) -> String`, not pipe-delimited syntax
- **Two closure forms only**: `|params| expr` for inline, `do |params| ... end` for multi-line. No `fn` keyword form.
- **`.` prefix for instance variables**: `.name`, `.age`
- **`()` for tuples, `{}` for dicts**: Empty tuple `()`, empty dict `{:}`
- **`###` for multiline comments** (not `/* */` or `""" """`)
- **`@name` for macros, `@[...]` for annotations**: `@memoize` invokes a macro, `@[deprecated]` attaches metadata
- **Symbol sets via type aliases**: `type Status = :ok | :error | :pending` for typed symbols

## Design Process

New language features follow this workflow:
1. Brainstorm with clarifying questions, approach comparison, and design approval
2. Write feature document to the appropriate `docs/<NN-section>/` directory
3. Integrate into `Opal.md` (update BNF, add section, update stdlib table) and link to the feature doc

When updating Opal.md, always check that changes are consistent across: BNF grammar (section 3), the relevant syntax section, the stdlib table (section 11), and the philosophy line (section 1) if a new first-class concept is added.

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
