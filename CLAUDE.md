# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Opal (Opinionated Programming Algorithmic Language) is a programming language in the design/specification phase. There is no compiler or interpreter yet — the project is a language specification document (`Opal.md`) and supporting design documents.

## Repository Structure

- **`Opal.md`** — The complete language specification (~2800 lines). This is the primary artifact. Organized into 13 sections from foundational to generalist: Philosophy, Facts, BNF Grammar, Basics, Control Flow, Functions & Types, Error Handling, Concurrency, Software Engineering Patterns, Metaprogramming, Standard Library, Tooling, Pretotyping, plus appendices.
- **`docs/plans/`** — Approved design documents for major features, each created through a brainstorming process before being integrated into Opal.md.

## Key Language Design Rules

These rules are enforced across all examples and specifications. Violating them in code examples is a bug:

- **Single quotes for chars only**: `'a'`, `'ñ'` — never for multi-character strings
- **Double quotes for strings**: `"hello"`, `f"hi {name}"`, `r"raw\n"`, `t"template {x}"`
- **`with` keyword is ONLY for DSL config blocks** (nginx-style). Object creation uses `.new()` with named args. String interpolation uses f-strings.
- **`defaults` keyword** (not `with`) for null object variant creation
- **`::` for type annotations**: `name::String`
- **`.` prefix for instance variables**: `.name`, `.age`
- **`()` for tuples, `{}` for dicts**: Empty tuple `()`, empty dict `{:}`
- **`#{ }#` for multiline comments** (not `/* */` or `""" """`)

## Design Process

New language features follow this workflow:
1. Brainstorm with clarifying questions, approach comparison, and design approval
2. Write design document to `docs/plans/YYYY-MM-DD-<topic>-design.md`
3. Integrate approved design into `Opal.md` (update BNF, add section, update stdlib table)

When updating Opal.md, always check that changes are consistent across: BNF grammar (section 3), the relevant syntax section, the stdlib table (section 11), and the philosophy line (section 1) if a new first-class concept is added.

## Existing Design Documents

| File | Covers |
|---|---|
| `2026-03-01-async-concurrency-design.md` | Four-layer model: actors, parallel, async/futures, supervisors |
| `2026-03-01-di-and-events-design.md` | `needs` keyword, `event`/`emit`/`on`, optional Container |
| `2026-03-01-metaprogramming-design.md` | Julia-adapted quoting, hygienic macros, AST, subdomains |
| `2026-03-01-self-hosting-foundations-design.md` | Operator overloading, iterators, custom errors, destructuring, protocol defaults |
