# Separate ControlFlow from EvalError — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Separate control flow signals (Return, Break, Next, Reply) from actual errors (Panic, Fail) in the `EvalError` enum using a nested `FlowSignal` type.

**Architecture:** Introduce `FlowSignal` enum with the 4 control flow variants. Nest it inside `EvalError::Flow(FlowSignal)`. Add convenience constructors. Mechanically update all 22 emit/catch sites in `eval.rs`.

**Tech Stack:** Rust, thiserror, opal-interp crate

**Spec:** `docs/plans/2026-03-16-controlflow-separation-design.md`

---

## File Structure

- Modify: `crates/opal-interp/src/eval.rs` — all changes are in this file (type definitions + 22 call sites)

---

## Chunk 1: Introduce FlowSignal and update EvalError

### Task 1: Add FlowSignal enum and restructure EvalError

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:33-47` (EvalError definition)

- [ ] **Step 1: Replace the EvalError enum definition**

Replace lines 33-47 (the current `EvalError` enum) with:

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

impl EvalError {
    fn ret(v: Value) -> Self {
        Self::Flow(FlowSignal::Return(v))
    }
    fn brk() -> Self {
        Self::Flow(FlowSignal::Break)
    }
    fn nxt() -> Self {
        Self::Flow(FlowSignal::Next)
    }
    fn reply(v: Value) -> Self {
        Self::Flow(FlowSignal::Reply(v))
    }
}
```

- [ ] **Step 2: Verify it compiles (will fail — unused old variants referenced)**

Run: `cargo build -p opal-interp 2>&1 | grep "^error" | head -5`
Expected: Errors about `EvalError::Return`, `EvalError::Break`, etc. not existing. This confirms the old variants are removed and we need to update call sites.

- [ ] **Step 3: Commit the type definition change**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: introduce FlowSignal enum and restructure EvalError"
```

---

### Task 2: Update all emit sites (return Err(...))

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — 6 emit sites

The emit sites use the convenience constructors for conciseness.

- [ ] **Step 1: Replace all emit sites**

Line 891:
```
return Err(EvalError::Return(val));
```
→
```
return Err(EvalError::ret(val));
```

Line 1411:
```
return Err(EvalError::Break);
```
→
```
return Err(EvalError::brk());
```

Line 1414:
```
return Err(EvalError::Next);
```
→
```
return Err(EvalError::nxt());
```

Line 1418:
```
return Err(EvalError::Reply(val));
```
→
```
return Err(EvalError::reply(val));
```

Note: Line numbers will shift after Task 1 adds ~30 lines. Use the string content to find these sites, not line numbers.

- [ ] **Step 2: Verify only catch sites remain as errors**

Run: `cargo build -p opal-interp 2>&1 | grep "^error" | wc -l`
Expected: Errors should be reduced to only the catch site references.

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: update emit sites to use FlowSignal constructors"
```

---

### Task 3: Update all catch sites — loop Break/Next handlers

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — 8 catch sites for Break/Next in loops

There are 4 loop constructs, each catching Break and Next:

- [ ] **Step 1: Replace all Break/Next catch sites**

Replace all instances of this pattern (8 sites total, in pairs):
```
Err(EvalError::Break) => break,
Err(EvalError::Next) => continue,
```
→
```
Err(EvalError::Flow(FlowSignal::Break)) => break,
Err(EvalError::Flow(FlowSignal::Next)) => continue,
```

Use `replace_all` since the pattern is identical across all 4 loop constructs.

- [ ] **Step 2: Verify Break/Next errors are gone**

Run: `cargo build -p opal-interp 2>&1 | grep "Break\|Next" | head -5`
Expected: No remaining errors about `Break` or `Next`.

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: update loop Break/Next catch sites to use FlowSignal"
```

---

### Task 4: Update all catch sites — Return handlers

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — 10 catch sites for Return

- [ ] **Step 1: Replace all Return catch sites**

Each of these patterns needs updating. Search for `EvalError::Return` and replace:

```
Err(EvalError::Return(
```
→
```
Err(EvalError::Flow(FlowSignal::Return(
```

There are 10 sites. Each match arm's closing `)` count increases by one due to the extra nesting. The specific patterns:

1. `Err(EvalError::Return(val))` → `Err(EvalError::Flow(FlowSignal::Return(val)))`
2. `Err(EvalError::Return(v))` → `Err(EvalError::Flow(FlowSignal::Return(v)))`
3. `Err(EvalError::Return(Value::Ast(ast_id)))` → `Err(EvalError::Flow(FlowSignal::Return(Value::Ast(ast_id))))`
4. `Err(EvalError::Return(Value::String(s)))` → `Err(EvalError::Flow(FlowSignal::Return(Value::String(s))))`
5. `Err(EvalError::Return(_))` → `Err(EvalError::Flow(FlowSignal::Return(_)))`

- [ ] **Step 2: Verify Return errors are gone**

Run: `cargo build -p opal-interp 2>&1 | grep "Return" | head -5`
Expected: No remaining errors about `Return`.

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: update Return catch sites to use FlowSignal"
```

---

### Task 5: Update Reply catch site

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — 1 catch site for Reply

- [ ] **Step 1: Replace the Reply catch site**

```
Err(EvalError::Reply(val))
```
→
```
Err(EvalError::Flow(FlowSignal::Reply(val)))
```

- [ ] **Step 2: Full build verification**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished` — no errors.

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: update Reply catch site to use FlowSignal"
```

---

## Chunk 2: Full Verification

### Task 6: Run all tests and push

- [ ] **Step 1: Run unit tests**

Run: `cargo test -p opal-interp --lib 2>&1 | tail -3`
Expected: `test result: ok. 179 passed; 0 failed; ...`

- [ ] **Step 2: Run spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -3`
Expected: `Results: 102 passed, 0 failed, 16 skipped`

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -- -D warnings 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 4: Run formatter**

Run: `cargo fmt --all -- --check 2>&1`
Expected: No output (already formatted)

If formatting diffs appear, run `cargo fmt --all` and commit:
```bash
git add -u && git commit -m "style: format FlowSignal changes"
```

- [ ] **Step 5: Verify no remaining references to old variants**

Run: `grep -n 'EvalError::Return\|EvalError::Break\b\|EvalError::Next\b\|EvalError::Reply' crates/opal-interp/src/eval.rs | head -10`
Expected: No matches. All references should use `EvalError::Flow(FlowSignal::...)` or the convenience constructors.

- [ ] **Step 6: Commit design and plan docs, push everything**

```bash
git add docs/plans/2026-03-16-controlflow-separation-design.md docs/plans/2026-03-16-controlflow-separation-plan.md
git commit -m "docs: add ControlFlow separation design spec and plan"
git push
```
