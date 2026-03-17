# Extract String and Dict Method Dispatch — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract String (180 lines, 18 methods) and Dict (103 lines, 7 methods) dispatch arms from the 1,400-line `call_method` function into separate files.

**Architecture:** Create two new files (`string_methods.rs`, `dict_methods.rs`) each containing an `impl<W: Write> Interpreter<W>` block with a single dispatch method. Replace the corresponding arms in `call_method` with one-line delegation calls.

**Tech Stack:** Rust, opal-interp crate

**Spec:** `docs/plans/2026-03-16-extract-method-dispatch-design.md`

---

## File Structure

- Create: `crates/opal-interp/src/string_methods.rs` — `call_string_method` dispatch
- Create: `crates/opal-interp/src/dict_methods.rs` — `call_dict_method` dispatch
- Modify: `crates/opal-interp/src/lib.rs` — add `mod` declarations
- Modify: `crates/opal-interp/src/eval.rs` — replace String/Dict arms with delegation

---

### Task 1: Create string_methods.rs

**Files:**
- Create: `crates/opal-interp/src/string_methods.rs`

- [ ] **Step 1: Create the file with the function shell**

Create `crates/opal-interp/src/string_methods.rs` with this content:

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
    ) -> Result<Value, EvalError> {
        match method {
            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on String", method),
            )),
        }
    }
}
```

- [ ] **Step 2: Add the module declaration to lib.rs**

In `crates/opal-interp/src/lib.rs`, add after `pub mod loader;`:

```rust
mod string_methods;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished` (the function exists but isn't called yet)

- [ ] **Step 4: Move all 18 string method arms into the match**

Copy lines 3611-3790 from `eval.rs` (the entire String method block) into the `match method { ... }` block in `string_methods.rs`. Each arm changes from matching `(Value::String(s), "method_name")` to just `"method_name"`. For example:

Before (in eval.rs):
```rust
(Value::String(s), "length") => Ok(Value::Integer(s.len() as i64)),
(Value::String(s), "split") => { ... }
```

After (in string_methods.rs):
```rust
"length" => Ok(Value::Integer(s.len() as i64)),
"split" => { ... }
```

Remove the `(Value::String(s), ` prefix and closing `)` from every arm. The variable `s` is now the function parameter. The variable `args` is also a function parameter.

All 18 methods: `length`, `split`, `trim`, `contains`, `replace`, `starts_with`, `ends_with`, `to_upper`, `to_lower`, `chars`, `to_int`, `to_float`, `reverse`, `upcase`, `downcase`, `slice`, `index`, `empty?`.

- [ ] **Step 5: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 6: Commit**

```bash
git add crates/opal-interp/src/string_methods.rs crates/opal-interp/src/lib.rs
git commit -m "refactor: create string_methods.rs with all 18 string method arms"
```

---

### Task 2: Wire up string method delegation in eval.rs

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:3610-3790` (String arm block)

- [ ] **Step 1: Replace the entire String arm block in call_method**

Remove lines 3611-3790 (all the `(Value::String(s), "...")` arms) and replace with a single delegation line. The line before (3610) is the end of the last List arm. The line after (3791, now adjacent) starts the Dict comment.

The new content at the String section:

```rust
            // String methods
            (Value::String(s), _) => self.call_string_method(s, method, args),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 3: Run tests**

Run: `cargo test -p opal-interp --lib 2>&1 | tail -3`
Expected: All 179 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: delegate string method dispatch to string_methods.rs"
```

---

### Task 3: Create dict_methods.rs

**Files:**
- Create: `crates/opal-interp/src/dict_methods.rs`

- [ ] **Step 1: Create the file with the function shell**

Create `crates/opal-interp/src/dict_methods.rs` with this content:

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
    ) -> Result<Value, EvalError> {
        match method {
            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on Dict", method),
            )),
        }
    }
}
```

- [ ] **Step 2: Add the module declaration to lib.rs**

In `crates/opal-interp/src/lib.rs`, add after `mod string_methods;`:

```rust
mod dict_methods;
```

- [ ] **Step 3: Move all 7 dict method arms into the match**

Copy lines from eval.rs for the Dict arm block (currently starting after the string delegation, ending before the Range arm). Each arm changes from `(Value::Dict(entries), "method_name")` to just `"method_name"`.

All 7 methods: `length`, `get`, `keys`, `values`, `set`, `has_key`, `merge`.

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 5: Commit**

```bash
git add crates/opal-interp/src/dict_methods.rs crates/opal-interp/src/lib.rs
git commit -m "refactor: create dict_methods.rs with all 7 dict method arms"
```

---

### Task 4: Wire up dict method delegation in eval.rs

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` (Dict arm block)

- [ ] **Step 1: Replace the entire Dict arm block in call_method**

Remove all the `(Value::Dict(entries), "...")` arms and replace with:

```rust
            // Dict methods
            (Value::Dict(entries), _) => self.call_dict_method(entries, method, args),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 3: Run tests**

Run: `cargo test -p opal-interp --lib 2>&1 | tail -3`
Expected: All 179 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: delegate dict method dispatch to dict_methods.rs"
```

---

### Task 5: Full verification and push

- [ ] **Step 1: Run full test suite**

Run: `cargo test --all 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 2: Run spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -3`
Expected: `Results: 102 passed, 0 failed, 16 skipped`

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -- -D warnings 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 4: Run formatter**

Run: `cargo fmt --all -- --check 2>&1`
Expected: No output

If formatting diffs appear, run `cargo fmt --all` and commit:
```bash
git add -u && git commit -m "style: format extracted method dispatch files"
```

- [ ] **Step 5: Commit design and plan docs, push everything**

```bash
git add docs/plans/2026-03-16-extract-method-dispatch-design.md docs/plans/2026-03-17-extract-method-dispatch-plan.md
git commit -m "docs: add method dispatch extraction design spec and plan"
git push
```
