# Separate ControlFlow from EvalError — Design Spec

**Date:** 2026-03-16
**Status:** Draft
**Scope:** Restructure `EvalError` to separate control flow signals from actual errors using a nested enum

## Problem

`EvalError` conflates two distinct concerns in a single flat enum:

```rust
pub enum EvalError {
    Panic(PanicKind, String),  // uncatchable error
    Fail(Value),               // catchable error (auto-throw)
    Return(Value),             // function return signal
    Reply(Value),              // actor reply signal
    Break,                     // loop break signal
    Next,                      // loop continue signal
}
```

**Consequences:**

1. **Silent propagation bugs:** If a new control flow variant is added (e.g., `Yield` for generators) and a `match` site doesn't handle it, it propagates up the stack as an error. The compiler won't warn because the wildcard `Err(e) => return Err(e)` (via `?`) catches everything.
2. **Unclear intent at match sites:** When reading `Err(EvalError::Return(v)) =>`, it's not obvious that this is expected control flow, not an error condition. The reader must know the interpreter's conventions.
3. **No type-level distinction:** Functions that should only produce errors (e.g., `eval_binary_op`) use the same return type as functions that legitimately produce control flow (e.g., `eval_stmt`). The type system doesn't help enforce this.

## Design

### Nested Enum Approach

Introduce `FlowSignal` as a separate enum, nested inside `EvalError` via a `Flow` variant:

```rust
#[derive(Debug)]
pub enum FlowSignal {
    Return(Value),
    Break,
    Next,
    Reply(Value),
}

impl std::fmt::Display for FlowSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowSignal::Return(_) => write!(f, "return"),
            FlowSignal::Break => write!(f, "break"),
            FlowSignal::Next => write!(f, "next"),
            FlowSignal::Reply(_) => write!(f, "reply"),
        }
    }
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("{0}: {1}")]
    Panic(PanicKind, String),
    #[error("{0}")]
    Fail(Value),
    #[error("{0}")]
    Flow(FlowSignal),
}
```

`FlowSignal` is `pub` (for pattern matching inside `eval.rs` and future internal consumers) but is **not** re-exported from `lib.rs`. External callers (CLI, LSP) only match on `Panic` and `Fail` — they never see control flow signals.

### Convenience Constructors

To keep emit sites concise, add shorthand methods on `EvalError`:

```rust
impl EvalError {
    fn ret(v: Value) -> Self { Self::Flow(FlowSignal::Return(v)) }
    fn brk() -> Self { Self::Flow(FlowSignal::Break) }
    fn nxt() -> Self { Self::Flow(FlowSignal::Next) }
    fn reply(v: Value) -> Self { Self::Flow(FlowSignal::Reply(v)) }
}
```

Emit sites use `Err(EvalError::ret(v))` instead of `Err(EvalError::Flow(FlowSignal::Return(v)))`.

### Call Site Changes

**Emit sites (4 unique patterns across ~6 locations):**

| Before | After |
|---|---|
| `Err(EvalError::Return(v))` | `Err(EvalError::ret(v))` |
| `Err(EvalError::Break)` | `Err(EvalError::brk())` |
| `Err(EvalError::Next)` | `Err(EvalError::nxt())` |
| `Err(EvalError::Reply(v))` | `Err(EvalError::reply(v))` |

**Catch sites (~16 locations):**

| Before | After |
|---|---|
| `Err(EvalError::Return(v)) =>` | `Err(EvalError::Flow(FlowSignal::Return(v))) =>` |
| `Err(EvalError::Break) =>` | `Err(EvalError::Flow(FlowSignal::Break)) =>` |
| `Err(EvalError::Next) =>` | `Err(EvalError::Flow(FlowSignal::Next)) =>` |
| `Err(EvalError::Reply(v)) =>` | `Err(EvalError::Flow(FlowSignal::Reply(v))) =>` |

**Unchanged (~185 sites):**
- All `EvalError::Panic(...)` emit/catch sites — untouched
- All `EvalError::Fail(...)` emit/catch sites — untouched
- All `?` operator usage — still works (same `Result<Value, EvalError>` type)

### What Doesn't Change

- Function signatures — still `Result<Value, EvalError>` everywhere
- The `?` operator — works identically
- External API — `EvalError` is still the public error type from `lib.rs`; `FlowSignal` is `pub` but not re-exported
- `Display` output — preserved via explicit `Display` impl on `FlowSignal` (e.g., `Return(v)` still displays as `"return"`)
- Test code — same assertions, match arms just get longer at catch sites

### Why Nested Enum Over Full Separation

Full separation (two distinct types with a union wrapper) would require changing every function signature and `?` usage. The nested approach gets the same type-level distinction with zero signature changes. The trade-off is that functions which should never produce control flow can still *technically* return `EvalError::Flow(...)`, but this is a convention issue, not a correctness issue — and it can be enforced later via clippy lints or module boundaries when `eval.rs` is decomposed.

## Testing

Pure mechanical refactor — zero semantic change. All existing tests cover the affected paths. Verify with `cargo test --all` + `./tests/run_spec.sh` + `cargo clippy -- -D warnings`.

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Approach | Nested enum (`Flow(FlowSignal)`) | No signature changes, `?` still works, gets exhaustive matching |
| `Reply` placement | In `FlowSignal` | It's control flow today; BEAM migration will rewrite actor semantics from scratch |
| Convenience constructors | Yes (`ret`, `brk`, `nxt`, `reply`) | Keeps emit sites concise; avoids `Flow(FlowSignal::X)` verbosity |
| Scope | `eval.rs` only | All ~22 emit/catch sites are in one file |
| `FlowSignal` visibility | `pub` but not re-exported | External callers never match on control flow; re-export when needed |
| `Display` preservation | Explicit `Display` impl on `FlowSignal` | Prevents `"return"` → `"control flow: Return(...)"` regression |
