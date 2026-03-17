# Extract String and Dict Method Dispatch — Design Spec

**Date:** 2026-03-16
**Status:** Draft
**Scope:** Move String and Dict method dispatch from `call_method` into separate files (phase 1 of eval.rs decomposition)

## Problem

`call_method` in `eval.rs` is ~1,400 lines — a single function dispatching methods for 10+ value types. It's unnavigable, untestable in isolation, and every edit risks touching unrelated type logic. The String arm (180 lines, 18 methods) and Dict arm (103 lines, 7 methods) are self-contained value transforms that don't need mutable interpreter state, making them ideal first extraction candidates.

## Design

### Approach: Split `impl` blocks across files

Rust allows multiple `impl` blocks for the same type in different files within a crate. Each extracted file contains `impl<W: Write> Interpreter<W> { ... }` with a single dispatch method. No file renames, no callback gymnastics, full `self` access.

### New Files

**`crates/opal-interp/src/string_methods.rs`**

```rust
use std::io::Write;
use opal_runtime::Value;
use crate::eval::{EvalError, Interpreter, PanicKind};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_string_method(
        &mut self,
        s: &str,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> { ... }
}
```

Contains the 18 string method arms: `length`, `split`, `trim`, `contains`, `replace`, `starts_with`, `ends_with`, `to_upper`, `to_lower`, `chars`, `to_int`, `to_float`, `reverse`, `upcase`, `downcase`, `slice`, `index`, `empty?`.

**`crates/opal-interp/src/dict_methods.rs`**

```rust
use std::io::Write;
use opal_runtime::Value;
use crate::eval::{EvalError, Interpreter, PanicKind};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_dict_method(
        &mut self,
        entries: &[(String, Value)],
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> { ... }
}
```

Contains the 7 dict method arms: `length`, `get`, `keys`, `values`, `set`, `has_key`, `merge`.

### Modified Files

**`crates/opal-interp/src/lib.rs`**

Add module declarations:
```rust
mod string_methods;
mod dict_methods;
```

**`crates/opal-interp/src/eval.rs`**

Replace the String and Dict arms in `call_method` with delegation:

```rust
(Value::String(s), _) => self.call_string_method(s, method, args),
(Value::Dict(entries), _) => self.call_dict_method(entries, method, args),
```

### Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| `&mut self` on both methods | Yes | Forward-compatible for future closure-taking methods (e.g., `dict.map`). Uniform signature. |
| `pub(crate)` visibility | Yes | Called from `eval.rs` within the same crate; not part of the public API |
| Separate files (not `eval/` dir) | Sibling files in `src/` | No file renames, minimal diff, validated pattern for future extractions |
| Scope | String + Dict only | Pure value transforms, low risk. Validates the pattern before tackling List/Instance/Class. |

### What Doesn't Change

- All other `call_method` arms — untouched
- `call_method` still exists in `eval.rs` — just delegates two arms
- External API — nothing changes in `lib.rs` re-exports
- Error types — same `EvalError::Panic(PanicKind::TypeError/RuntimeError, ...)` patterns
- `Interpreter` struct definition — stays in `eval.rs`

### Future Work

After this extraction is validated, the next phases (captured in `docs/future-improvements.md`):
- Phase 2: Extract List methods (~455 lines, needs `call_closure` access)
- Phase 3: Extract Instance dispatch (~249 lines, most complex arm)
- Phase 4: Extract Class/Actor/Enum dispatch (~330 lines combined)

### Testing

Pure extraction — zero semantic change. All existing tests cover string and dict methods:
- String: `string_methods`, `string_split`, `string_trim`, `string_contains`, etc.
- Dict: `dict_methods`, `dict_get_set`, `dict_keys_values`, etc.
- Spec tests: string and dict test files in `tests/spec/`

Verify with `cargo test --all` + `./tests/run_spec.sh` + `cargo clippy -- -D warnings` + `cargo fmt --all -- --check`.
