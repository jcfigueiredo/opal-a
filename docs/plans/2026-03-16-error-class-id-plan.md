# Cache `error_class_id` on Interpreter — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace stringly-typed `"Error"` class identity checks with a cached `ClassId` field, eliminating a production `.expect()` and 5 linear/string lookups.

**Architecture:** Add `error_class_id: ClassId` field to `Interpreter<W>`, set it during `register_error_class()`, replace all 5 identity-check sites with direct `ClassId` comparison.

**Tech Stack:** Rust, opal-interp crate

**Spec:** `docs/plans/2026-03-16-error-class-id-design.md`

---

## File Structure

- Modify: `crates/opal-interp/src/eval.rs` — all changes are in this file

---

## Chunk 1: Cache error_class_id and replace all identity checks

### Task 1: Add field and initialize in both constructors

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:163-208` (struct definition)
- Modify: `crates/opal-interp/src/eval.rs:212-242` (`new()` constructor)
- Modify: `crates/opal-interp/src/eval.rs:258-288` (`with_writer()` constructor)

- [ ] **Step 1: Add the field to the struct**

In `Interpreter<W>` struct (line 207), add after `container_registrations`:

```rust
    /// Cached ClassId for the built-in Error class (set during register_error_class)
    error_class_id: ClassId,
```

- [ ] **Step 2: Initialize in `new()` constructor**

In the `Self { ... }` block of `new()` (line 241), add after `container_registrations: HashMap::new(),`:

```rust
            error_class_id: ClassId(usize::MAX),
```

- [ ] **Step 3: Initialize in `with_writer()` constructor**

In the `Self { ... }` block of `with_writer()` (line 287), add after `container_registrations: HashMap::new(),`:

```rust
            error_class_id: ClassId(usize::MAX),
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p opal-interp 2>&1 | tail -3`
Expected: `Finished` (the field is set but not yet used — no warnings expected since constructors assign it)

- [ ] **Step 5: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: add error_class_id field to Interpreter"
```

---

### Task 2: Set the field during registration

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:707-719` (`register_error_class`)

- [ ] **Step 1: Set `self.error_class_id` in `register_error_class`**

Replace `register_error_class` (lines 707-719):

```rust
    fn register_error_class(&mut self) {
        // Error class has message and cause fields, but the constructor
        // is handled specially (not via the standard .new() path)
        let class_id = ClassId(self.classes.len());
        self.classes.push(StoredClass {
            name: "Error".to_string(),
            parent: None,
            needs: vec![],
            methods: vec![],
            static_methods: vec![],
        });
        self.error_class_id = class_id;
        self.env.set("Error".to_string(), Value::Class(class_id));
    }
```

The only change is adding `self.error_class_id = class_id;` after the push.

- [ ] **Step 2: Fix the doc comment**

The existing doc comment says "Register the built-in Container class" — it's a copy-paste error. Replace line 706:

```rust
    /// Register the built-in Error class for error handling
```

- [ ] **Step 3: Run tests to verify nothing broke**

Run: `cargo test -p opal-interp 2>&1 | tail -5`
Expected: All tests pass (field is set but not yet consumed)

- [ ] **Step 4: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: set error_class_id during Error class registration"
```

---

### Task 3: Replace `make_error_instance` linear scan

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:4819-4838` (`make_error_instance`)

- [ ] **Step 1: Replace the function body**

Replace `make_error_instance` (lines 4819-4838) with:

```rust
    fn make_error_instance(&mut self, val: Value) -> Value {
        let (message, cause) = match &val {
            Value::String(s) => (s.clone(), Value::Null),
            other => (format!("{}", other), val.clone()),
        };
        let instance_id = InstanceId(self.instances.len());
        let mut fields = HashMap::new();
        fields.insert("message".to_string(), Value::String(message));
        fields.insert("cause".to_string(), cause);
        self.instances.push(StoredInstance {
            class_id: self.error_class_id,
            fields,
        });
        Value::Instance(instance_id)
    }
```

This removes the linear scan (`.iter().position(...)`) and the `.expect()` call. The `ClassId` is used directly.

- [ ] **Step 2: Run tests**

Run: `cargo test -p opal-interp 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: use cached error_class_id in make_error_instance"
```

---

### Task 4: Replace `is_error_instance` string check

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:4841-4850` (`is_error_instance`)

- [ ] **Step 1: Replace the function body**

Replace `is_error_instance` (lines 4841-4850) with:

```rust
    fn is_error_instance(&self, val: &Value) -> bool {
        if let Value::Instance(iid) = val && let Some(inst) = self.instances.get(iid.0) {
            return inst.class_id == self.error_class_id;
        }
        false
    }
```

This replaces the class name lookup with a direct `ClassId` comparison. No Vec indexing, no string comparison.

- [ ] **Step 2: Run tests**

Run: `cargo test -p opal-interp 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: use cached error_class_id in is_error_instance"
```

---

### Task 5: Replace `match_pattern` string check

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:2708-2714` (inside `match_pattern`)

- [ ] **Step 1: Replace the Error class check in match_pattern**

Replace lines 2708-2714:

```rust
                        if let Value::Instance(iid) = value
                            && let Some(inst) = self.instances.get(iid.0)
                            && self
                                .classes
                                .get(inst.class_id.0)
                                .map(|c| c.name == "Error")
                                .unwrap_or(false)
```

With:

```rust
                        if let Value::Instance(iid) = value
                            && let Some(inst) = self.instances.get(iid.0)
                            && inst.class_id == self.error_class_id
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opal-interp 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: use cached error_class_id in match_pattern"
```

---

### Task 6: Replace `format_value_display` and `format_value` string checks

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:5009` (`format_value_display`)
- Modify: `crates/opal-interp/src/eval.rs:5049` (`format_value`)

- [ ] **Step 1: Replace in `format_value_display`**

Replace line 5009:

```rust
            if class.name == "Error" && let Some(msg) = inst.fields.get("message").cloned() {
```

With:

```rust
            if inst.class_id == self.error_class_id && let Some(msg) = inst.fields.get("message").cloned() {
```

Note: this site no longer needs the `class` variable for the Error check, but it's still used in other branches (e.g., `try_instance_to_string`), so leave the `let class = ...` line in place.

- [ ] **Step 2: Replace in `format_value`**

Replace line 5049:

```rust
                if class.name == "Error" && let Some(msg) = inst.fields.get("message") {
```

With:

```rust
                if inst.class_id == self.error_class_id && let Some(msg) = inst.fields.get("message") {
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p opal-interp 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 4: Run spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -3`
Expected: `Results: 102 passed, 0 failed, 16 skipped`

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -- -D warnings 2>&1 | tail -3`
Expected: `Finished`

- [ ] **Step 6: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: use cached error_class_id in format functions

Completes the error_class_id caching refactor. All 5 stringly-typed
Error class identity checks now use direct ClassId comparison."
```
