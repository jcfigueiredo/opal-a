# Opal for the AI Era

## Why this language, why now

Most languages were designed before AI assistants became part of daily development. Opal is designed for this new workflow:

- Humans and AI both write code.
- Context windows and token budgets matter.
- Predictable, explicit syntax improves generation quality.
- Built-in architecture patterns reduce framework glue.

Opal is an opinionated language that combines:

- Ruby-like readability
- actor-based concurrency
- gradual typing
- macro-powered extensibility
- software architecture patterns as first-class language features

The goal is not novelty. The goal is to make real software easier to build, review, and evolve with AI in the loop.

---

## What is special about Opal

### 1. AI-friendly surface area

Opal aims for one explicit way to do common tasks. That helps both humans and models avoid branching syntax and accidental style drift.

- fewer equivalent forms for the same intent
- consistent block and control-flow patterns
- readable defaults that reduce prompt length and review overhead

### 2. Architecture is built in, not bolted on

Instead of forcing teams to assemble core architecture from frameworks, Opal includes language-level patterns:

- `needs` for dependency injection
- domain events with `event`, `emit`, `on`
- specifications for composable business rules
- models/settings with validation

This gives AI generated code stronger structure by default.

### 3. Concurrency model with clear lanes

Opal separates concerns cleanly:

- actors for stateful concurrent entities
- `parallel` for structured fan-out/fan-in
- `async`/`await` for fine-grained futures
- supervisors for fault-tolerant trees

That clarity is easier to prompt, generate, and reason about than mixed async models.

### 4. Metaprogramming you can inspect

Macros and AST tooling allow extension without compiler patching. Crucially, expansions are inspectable (`macroexpand`), which keeps generated code auditable.

### 5. Practical path to production runtime semantics

The project strategy is explicit: stabilize semantics in the interpreter, then target BEAM where actor/fault-tolerance semantics are native.

---

## Why this can be an alternative in the AI age

If AI is writing a lot of your first draft, the language should minimize ambiguity and maximize intent density.

Opal is aiming for exactly that:

- concise code with explicit meaning
- predictable syntax for model reliability
- built-in primitives for architecture and operations
- fewer external layers to explain in prompts

In short: fewer tokens spent on glue, more tokens spent on domain logic.

---

## Rock-solid daily core: 30 features to make excellent by default

This is the practical checklist for day-to-day developer confidence.

### Language core

1. Fast, precise parse errors with line/column.
2. Stable expression precedence and operator behavior.
3. Functions with defaults and closures (`|...|`, `do ... end`).
4. Pattern matching with useful guards.
5. Collection literals and idiomatic methods (`map`, `filter`, `reduce`).
6. Destructuring for tuples/dicts/lists.
7. Classes with `needs`, `init`, and method visibility.
8. Modules/imports with predictable resolution.

### Type and correctness ergonomics

9. Gradual type boundaries that catch bad inputs early.
10. `is`/`is not` and `typeof` for runtime clarity.
11. Type aliases, including symbol-set aliases.
12. Enums with data-carrying variants.
13. Auto-throw errors with `?` suppress-throw for explicit handling.
14. Error falsiness enabling `f()? or default` patterns.
15. Preconditions (`requires`) with clear failures.
16. Try/catch/ensure with reliable binding semantics.

### Concurrency and resilience

17. Actor messaging semantics that are deterministic and debuggable.
18. `parallel` cancellation behavior that is predictable.
19. `async`/`await` with explicit blocking points.
20. Supervisor strategies and restart limits.

### Built-in architecture patterns

21. `needs` dependency injection usable in classes/modules/actors.
22. Container registration/resolve with clear runtime errors.
23. Event declaration + async dispatch + handlers.
24. Specification combinators (`and`, `or`, `not`).
25. Model/settings validation and immutable defaults.

### Tooling and developer loop

26. REPL and `opal run` reliability for fast iteration.
27. Spec runner and testing ergonomics (`.topl`, assertions, hooks).
28. LSP diagnostics, go-to-definition, and symbols.
29. Tree-sitter/TextMate highlighting that matches parser reality.
30. Formatter/lint/docs workflow that keeps teams consistent.

---

## AI collaboration patterns Opal should encourage

- Generate from small, strict templates (fewer free-form variants).
- Prefer protocol-first designs so mocks and substitutions are automatic.
- Use `match` and Result flows to reduce hidden control paths.
- Keep macro usage inspectable and test macro expansions.
- Make event-driven boundaries explicit so generated code stays decoupled.

---

## Positioning statement

Opal is an opinionated, AI-friendly language for teams that want readable code, built-in architecture patterns, and actor-grade concurrency without heavy framework sprawl.

It is designed so humans and AI can produce code that is concise, explicit, and production-oriented from day one.
