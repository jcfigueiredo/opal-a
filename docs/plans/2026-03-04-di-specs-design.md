# DI Spec Tests — Design

## Goal

Write spec tests for all dependency injection features from `docs/06-patterns/dependency-injection.md`, then implement each feature in the interpreter to make them pass. Spec-first (TDD) approach.

## Scope — 6 spec files

All specs live in `tests/spec/06-patterns/` using the collab platform domain.

| File | Feature | Key behavior |
|---|---|---|
| `needs_defaults.opl` | `needs` with default values | Optional deps, override at `.new()` |
| `needs_modules.opl` | `needs` on modules | Module-level DI, `.dep` access in methods |
| `needs_actors.opl` | `needs` on actors | Actor DI, deps in `receive` handlers |
| `needs_protocol.opl` | Protocol-typed `needs` | Reject non-conforming values at construction |
| `container.opl` | `Container` class | `register`/`resolve`, missing registration error |
| `domain_events.opl` | `event`/`emit`/`on` | Event declaration, emission, handler dispatch |

## Implementation order

1. `needs_defaults` — add `default` field to `NeedsDecl`, eval fallback
2. `needs_modules` — add `needs` to `ModuleDef` AST, eval module construction
3. `needs_actors` — add `needs` to `ActorDef` AST, eval actor construction
4. `needs_protocol` — runtime protocol conformance check at `.new()`
5. `container` — stdlib `Container` class with `register`/`resolve`
6. `domain_events` — new `event`/`emit`/`on` keywords (lexer + parser + eval)

## Interpreter changes required

- **Parser**: `NeedsDecl.default`, `ModuleDef.needs`, `ActorDef.needs`, `event`/`emit`/`on` statements
- **Lexer**: `event`, `emit`, `on` tokens
- **Eval**: default needs resolution, module/actor needs injection, protocol validation at construction, Container stdlib, event dispatch
