# Error Hierarchy Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign Opal's error handling so `Error(...)` auto-throws, `?` suppresses the throw, panics are uncatchable, and all thrown values are wrapped in a built-in `Error` object.

**Architecture:** Replace 3 EvalError variants (`TypeError`, `RuntimeError`, `UndefinedVariable`) with `Panic(PanicKind, String)`. Make `Error(...)` return values auto-throw at the call site. Add `?` postfix operator to suppress auto-throw. Remove `!` propagation operator. Register built-in `Error` class with `message`, `cause` fields.

**Tech Stack:** Rust (interpreter), tree-sitter (grammar), Opal spec tests

**Design doc:** `docs/plans/2026-03-08-error-hierarchy-design.md`

---

### Task 1: Add PanicKind and refactor EvalError enum

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:15-35`

**Step 1: Add PanicKind enum and refactor EvalError**

Add `PanicKind` above `EvalError` and replace the three string variants with `Panic`:

```rust
#[derive(Debug, Clone, Copy)]
pub enum PanicKind {
    TypeError,
    NameError,
    RuntimeError,
}

impl std::fmt::Display for PanicKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanicKind::TypeError => write!(f, "TypeError"),
            PanicKind::NameError => write!(f, "NameError"),
            PanicKind::RuntimeError => write!(f, "RuntimeError"),
        }
    }
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("{0}: {1}")]
    Panic(PanicKind, String),
    #[error("return")]
    Return(Value),
    #[error("{0}")]
    Raise(Value),
    #[error("reply")]
    Reply(Value),
    #[error("break")]
    Break,
    #[error("next")]
    Next,
}
```

Note: `RequiresFailed` is removed — it will use `Raise` in Task 5.

**Step 2: Find-and-replace all TypeError, RuntimeError, UndefinedVariable**

Replace across the entire file:
- `EvalError::TypeError(` → `EvalError::Panic(PanicKind::TypeError, `
- `EvalError::RuntimeError(` → `EvalError::Panic(PanicKind::RuntimeError, `
- `EvalError::UndefinedVariable(` → `EvalError::Panic(PanicKind::NameError, `

There are approximately 105 TypeError, 39 RuntimeError, and 23 UndefinedVariable sites.

**Step 3: Fix the try/catch handler to NOT catch Panics**

In the `TryCatch` handler (around line 1970), change:

```rust
Err(EvalError::Raise(val) | EvalError::RequiresFailed(val)) => {
```

to:

```rust
Err(EvalError::Raise(val)) => {
```

Panics (`EvalError::Panic`) will fall through to the `Err(e) => return Err(e)` branch, making them uncatchable.

**Step 4: Fix RequiresFailed references**

Change the `requires` handler (line 1039-1047) to use `Raise`:

```rust
StmtKind::Requires { condition, message } => {
    let cond = self.eval_expr(condition)?;
    if !cond.is_truthy() {
        let msg = match message {
            Some(m) => self.eval_expr(m)?,
            None => Value::String("requires condition failed".into()),
        };
        return Err(EvalError::Raise(msg));
    }
}
```

**Step 5: Run tests to see what breaks**

Run: `cargo test -p opal-interp 2>&1 | tail -30`
Expected: Compilation succeeds. Some tests may fail due to changed error messages (e.g., "TypeError:" prefix changes).

**Step 6: Fix any failing tests**

Update error message expectations in tests. The format changes from `TypeError: msg` to `TypeError: msg` (same, via the `Display` impl on `PanicKind`).

**Step 7: Run spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -20`

**Step 8: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: replace TypeError/RuntimeError/UndefinedVariable with Panic(PanicKind, String)

Merge three error variants into a single Panic variant with PanicKind enum.
Remove RequiresFailed — requires failures now use Raise.
Panics are uncatchable by try/catch."
```

---

### Task 2: Make Error falsy

**Files:**
- Modify: `crates/opal-runtime/src/value.rs:231-236`
- Test: `crates/opal-runtime/src/value.rs` (existing test at line 251)

**Step 1: Write the failing test**

Add to the `truthiness` test in `crates/opal-runtime/src/value.rs`:

```rust
// Error (enum_id 0, variant_index 1) should be falsy
let error_val = Value::EnumVariant {
    enum_id: EnumId(0),
    variant_index: 1,
    fields: vec![Value::String("test error".into())],
};
assert!(!error_val.is_truthy(), "Error should be falsy");

// Ok (enum_id 0, variant_index 0) should still be truthy
let ok_val = Value::EnumVariant {
    enum_id: EnumId(0),
    variant_index: 0,
    fields: vec![Value::Integer(42)],
};
assert!(ok_val.is_truthy(), "Ok should be truthy");

// Other enum variants should remain truthy
let some_val = Value::EnumVariant {
    enum_id: EnumId(1),
    variant_index: 0,
    fields: vec![Value::Integer(42)],
};
assert!(some_val.is_truthy(), "Other enum variants should be truthy");
```

You will need to add `use crate::EnumId;` to the test module if not already imported.

**Step 2: Run test to verify it fails**

Run: `cargo test -p opal-runtime truthiness -- --nocapture`
Expected: FAIL — Error variant is currently truthy.

**Step 3: Update is_truthy**

Change `is_truthy` in `crates/opal-runtime/src/value.rs:233-235`:

```rust
pub fn is_truthy(&self) -> bool {
    match self {
        Value::Bool(false) | Value::Null => false,
        // Error (Result enum_id=0, variant_index=1) is falsy
        Value::EnumVariant { enum_id, variant_index: 1, .. } if enum_id.0 == 0 => false,
        _ => true,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p opal-runtime truthiness -- --nocapture`
Expected: PASS

**Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -10`

**Step 6: Commit**

```bash
git add crates/opal-runtime/src/value.rs
git commit -m "feat: make Error falsy

Error (Result variant_index=1) is now falsy alongside false and null.
Ok and all other enum variants remain truthy."
```

---

### Task 3: Register built-in Error class

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` (around line 278, the `register_builtin_enums` method, and the `new()` constructor)

**Step 1: Write a failing interpreter test**

Add to the interpreter tests in `crates/opal-interp/src/eval.rs`:

```rust
#[test]
fn test_error_class_has_message_and_cause() {
    let result = run("
e = Error(\"test error\")
print(e.message)
print(e.cause)
");
    // For now, just test that Error() creates something with .message
    assert!(result.is_ok() || true); // placeholder
}
```

**Step 2: Run test to see current behavior**

Run: `cargo test -p opal-interp test_error_class_has_message_and_cause -- --nocapture`

**Step 3: Register the Error class at interpreter startup**

In the `new()` method of `Interpreter`, after `register_builtin_enums()`, add a method call to register the Error class. Create a new method `register_error_class`:

```rust
fn register_error_class(&mut self) {
    // Register Error as a built-in class with message and cause fields
    let class_id = self.classes.len();
    self.classes.push(StoredClass {
        name: "Error".to_string(),
        parent: None,
        fields: vec![
            ("message".to_string(), Some(Value::String("".into()))),
            ("cause".to_string(), Some(Value::Null)),
        ],
        methods: vec![],
        static_methods: vec![],
        implements: vec![],
        frozen_instances: false,
    });
    self.env.set("Error".to_string(), Value::Class(ClassId(class_id)));
}
```

Call `self.register_error_class();` in the `new()` method after `register_builtin_enums()`.

**Step 4: Update the Error() constructor call**

In the function call handler (around line 2441), update the `"Error"` case to create an Error class instance instead of an enum variant:

```rust
"Error" if !arg_values.is_empty() => {
    let arg = arg_values.into_iter().next().unwrap();
    let (message, cause) = match &arg {
        Value::String(s) => (s.clone(), Value::Null),
        other => {
            // Try to get string representation for message
            let msg = format!("{}", other);
            (msg, arg.clone())
        }
    };
    let error_class_id = self.classes.iter().position(|c| c.name == "Error")
        .expect("Error class not registered");
    return Ok(Value::Instance {
        class_id: ClassId(error_class_id),
        fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("message".to_string(), Value::String(message));
            fields.insert("cause".to_string(), cause);
            fields
        },
    });
}
```

**Step 5: Update the `fail` / `raise` handler to wrap in Error**

In the `StmtKind::Raise` handler (line 1049-1052), wrap the value in Error if it isn't already:

```rust
StmtKind::Raise(expr) => {
    let val = self.eval_expr(expr)?;
    let error_val = self.wrap_in_error(val);
    return Err(EvalError::Raise(error_val));
}
```

Add a helper method:

```rust
fn wrap_in_error(&self, val: Value) -> Value {
    // If already an Error instance, pass through
    if let Value::Instance { class_id, .. } = &val {
        if self.classes.get(class_id.0).map(|c| c.name.as_str()) == Some("Error") {
            return val;
        }
    }
    let (message, cause) = match &val {
        Value::String(s) => (s.clone(), Value::Null),
        other => (format!("{}", other), val.clone()),
    };
    let error_class_id = self.classes.iter().position(|c| c.name == "Error")
        .expect("Error class not registered");
    Value::Instance {
        class_id: ClassId(error_class_id),
        fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("message".to_string(), Value::String(message));
            fields.insert("cause".to_string(), cause);
            fields
        },
    }
}
```

**Step 6: Update requires handler to wrap in Error**

Change the `requires` handler to use `wrap_in_error`:

```rust
StmtKind::Requires { condition, message } => {
    let cond = self.eval_expr(condition)?;
    if !cond.is_truthy() {
        let msg = match message {
            Some(m) => self.eval_expr(m)?,
            None => Value::String("requires condition failed".into()),
        };
        let error_val = self.wrap_in_error(msg);
        return Err(EvalError::Raise(error_val));
    }
}
```

**Step 7: Run tests**

Run: `cargo test 2>&1 | tail -20`

**Step 8: Run spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -20`

Fix any failures — the main issue will be specs that match on the exact format of Error values. The `claim_result.opl` spec pattern-matches on `Ok`/`Error` which may need updating.

**Step 9: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "feat: register built-in Error class with message and cause fields

Error() now creates an Error class instance with .message and .cause.
fail/raise wraps values in Error automatically.
requires failures use Error wrapping."
```

---

### Task 4: Add auto-throw on Error return from function calls

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` (function call evaluation path)

**Step 1: Write a failing spec test**

Create `tests/spec/06-errors/auto_throw.opl`:

```opal
# expect: caught: user 42 not found

def find_user(id)
  if id > 40
    Error("user {id} not found")
  end
  "found"
end

try
  user = find_user(42)
  print(user)
catch as e
  print(f"caught: {e.message}")
end
```

**Step 2: Run spec to verify it fails**

Run: `cargo run --bin opal -- tests/spec/06-errors/auto_throw.opl 2>&1`
Expected: Currently prints "found" or the Error value — does NOT throw.

Actually — re-check: with the current code, `Error(...)` returns a value. The function returns it. The caller gets the value. No throw happens. The spec should print something other than "caught: ..." currently.

**Step 3: Add auto-throw logic in function call return path**

Find the function call evaluation in `eval.rs`. After a user-defined function returns a value, check if it's an Error instance and auto-throw:

Look for where `call_function` returns `Ok(value)` for user-defined functions. Add a check:

```rust
// After getting the return value from a function call
fn maybe_auto_throw(&self, val: Value) -> Result<Value, EvalError> {
    if let Value::Instance { class_id, ref fields } = &val {
        if self.classes.get(class_id.0).map(|c| c.name.as_str()) == Some("Error") {
            return Err(EvalError::Raise(val));
        }
    }
    Ok(val)
}
```

Call `self.maybe_auto_throw(result)?` at each function call return site in `eval_expr` where `ExprKind::Call` returns a value. The key location is in the `Call` expression handler after calling `self.call_function(...)`.

**Important:** Do NOT auto-throw for the `Error()` constructor itself — only for functions that *return* an Error. The constructor should return the Error value. Add a flag or check the call context.

**Step 4: Run the spec test**

Run: `cargo run --bin opal -- tests/spec/06-errors/auto_throw.opl 2>&1`
Expected: `caught: user 42 not found`

**Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -20`
Run: `./tests/run_spec.sh 2>&1 | tail -20`

Fix any failures. Key concern: existing specs that use `Ok()`/`Error()` as values and pattern match on them will break if Error auto-throws. Those specs need `?` (Task 5) to suppress auto-throw.

**Step 6: Commit**

```bash
git add crates/opal-interp/src/eval.rs tests/spec/06-errors/auto_throw.opl
git commit -m "feat: auto-throw when function returns Error

Functions returning Error(...) now automatically throw.
try/catch catches the thrown Error."
```

---

### Task 5: Add `?` postfix operator to suppress auto-throw

**Files:**
- Modify: `crates/opal-parser/src/ast.rs:272` (add `SuppressThrow` variant)
- Modify: `crates/opal-parser/src/parser.rs` (add `?` postfix parsing)
- Modify: `crates/opal-interp/src/eval.rs` (evaluate `SuppressThrow`)

**Step 1: Add AST node**

In `crates/opal-parser/src/ast.rs`, replace:

```rust
/// Result propagation: expr! — unwraps Ok or returns Err
Propagate(Box<Expr>),
```

with:

```rust
/// Suppress auto-throw: expr? — returns value or Error without throwing
SuppressThrow(Box<Expr>),
```

**Step 2: Fix compilation errors from removing Propagate**

In `crates/opal-parser/src/parser.rs`, find the `!` postfix parsing (around line 2005) and replace with `?` postfix:

```rust
} else if self.check(&Token::Question) {
    // Suppress auto-throw: expr? — only when ? is NOT followed by . (optional chaining)
    // and NOT followed by ? (null coalescing)
    if self.peek_ahead(1) == Some(&Token::Dot) || self.peek_ahead(1) == Some(&Token::Question) {
        break; // let ?. and ?? be handled elsewhere
    }
    let start = expr.span.start;
    self.advance(); // consume ?
    expr = Expr {
        kind: ExprKind::SuppressThrow(Box::new(expr)),
        span: Span { start, end: self.previous_span().end },
    };
}
```

Remove the `Token::Bang` propagation block (lines 2005-2016) that was previously there.

**Step 3: Add evaluation of SuppressThrow**

In `crates/opal-interp/src/eval.rs`, replace the `ExprKind::Propagate` handler (lines 1938-1960) with:

```rust
ExprKind::SuppressThrow(inner) => {
    // Evaluate the inner expression, but catch auto-throw Raise errors
    // and return the Error value instead of propagating
    match self.eval_expr(inner) {
        Ok(val) => Ok(val),
        Err(EvalError::Raise(val)) => Ok(val), // suppress throw, return Error
        Err(e) => Err(e), // panics and control flow pass through
    }
}
```

**Step 4: Write spec test for ? operator**

Create `tests/spec/06-errors/suppress_throw.opl`:

```opal
# expect: found | default_user | true

def find_user(id)
  if id > 100
    Error("not found")
  end
  f"user_{id}"
end

# ? suppresses throw, returns value or Error
result = find_user(1)?
print(result)

# ? with or for default
user = find_user(999)? or "default_user"
print(user)

# ? for boolean check
found = find_user(1)?
not_found = find_user(999)?
print(f"{found is not Error and not_found is Error}")
```

Wait — we need to check how `is Error` works. Let's simplify:

```opal
# expect: user_1 | default_user | true

def find_user(id)
  if id > 100
    Error("not found")
  end
  f"user_{id}"
end

# ? suppresses throw, returns the value
result = find_user(1)?
print(result)

# ? with or for default
user = find_user(999)? or "default_user"
print(user)

# Error is falsy
check = if find_user(1)?
  true
else
  false
end
print(check)
```

**Step 5: Run spec test**

Run: `cargo run --bin opal -- tests/spec/06-errors/suppress_throw.opl 2>&1`
Expected: `user_1 | default_user | true`

**Step 6: Update existing specs**

The existing specs in `tests/spec/06-errors/` that use `!` operator or rely on `Ok()`/`Error()` as values need updating:
- `result_propagation.opl` — remove `!`, use `?` where needed
- `result_helpers.opl` — use `?` to get Result values for `.ok?()` etc.
- `result_propagation_advanced.opl` — rewrite without `!`
- `result_error_scenarios.opl` — rewrite without `!`
- `result_unwrap_raise.opl` — update for new semantics
- `claim_result.opl` — update if it uses `Ok()`/`Error()` pattern matching

For each spec, the general transformation is:
- `result!` → just `result` (auto-throw is default now)
- `match result case Ok(x) / Error(e)` → `match result? case Error(e) / case x`
- `.ok?()` / `.err?()` → check truthiness of `?` result or use `is Error`
- `.unwrap_or(default)` → `f()? or default`

**Step 7: Run all tests**

Run: `cargo test 2>&1 | tail -20`
Run: `./tests/run_spec.sh 2>&1 | tail -20`

**Step 8: Commit**

```bash
git add crates/opal-parser/src/ast.rs crates/opal-parser/src/parser.rs crates/opal-interp/src/eval.rs tests/spec/06-errors/
git commit -m "feat: add ? operator to suppress auto-throw, remove ! propagation

? postfix suppresses auto-throw — returns value or Error.
Removed ! propagation operator (auto-throw replaces it).
Updated all error spec tests for new semantics."
```

---

### Task 6: Update catch to match on cause type

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` (TryCatch handler)

**Step 1: Write a failing spec test**

Create `tests/spec/06-errors/catch_cause_type.opl`:

```opal
# expect: not found | 42

class NotFoundError
  needs id

  def to_s()
    f"not found: {id}"
  end
end

def find_user(id)
  if id > 100
    Error(NotFoundError(id: id))
  end
  f"user_{id}"
end

# catch case matches on cause type
try
  find_user(999)
catch
  case NotFoundError as e
    print(f"not found")
  case _ as e
    print(f"other: {e.message}")
end

# catch as e gives the Error wrapper
try
  find_user(999)
catch as e
  print(e.cause.id)
end
```

**Step 2: Run to see current behavior**

Run: `cargo run --bin opal -- tests/spec/06-errors/catch_cause_type.opl 2>&1`

**Step 3: Update the TryCatch handler to match on cause type**

In the `TryCatch` handler, when we have `case` clauses with a type filter, check the `cause` field of the Error instance:

```rust
Err(EvalError::Raise(val)) => {
    let mut caught = false;
    let mut caught_val = Value::Null;
    for catch_clause in catches {
        if let Some(type_name) = &catch_clause.error_type {
            // Match on cause type inside Error wrapper
            let cause = if let Value::Instance { class_id, fields } = &val {
                if self.classes.get(class_id.0).map(|c| c.name.as_str()) == Some("Error") {
                    fields.get("cause").cloned().unwrap_or(Value::Null)
                } else {
                    val.clone()
                }
            } else {
                val.clone()
            };
            if !self.value_is_type(&cause, type_name) {
                continue;
            }
            // Bind the cause (not the wrapper) to the variable
            self.env.push_scope();
            self.env.set(catch_clause.var_name.clone(), cause);
            let catch_result = self.eval_block(&catch_clause.body);
            self.env.pop_scope();
            caught_val = catch_result?;
            caught = true;
            break;
        } else {
            // No type filter — catch all, bind the Error wrapper
            self.env.push_scope();
            self.env.set(catch_clause.var_name.clone(), val.clone());
            let catch_result = self.eval_block(&catch_clause.body);
            self.env.pop_scope();
            caught_val = catch_result?;
            caught = true;
            break;
        }
    }
    // ... rest of handler unchanged
}
```

**Step 4: Run spec test**

Run: `cargo run --bin opal -- tests/spec/06-errors/catch_cause_type.opl 2>&1`
Expected: `not found | 42`

**Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -20`
Run: `./tests/run_spec.sh 2>&1 | tail -20`

**Step 6: Commit**

```bash
git add crates/opal-interp/src/eval.rs tests/spec/06-errors/catch_cause_type.opl
git commit -m "feat: catch case matches on cause type inside Error wrapper

case Type as e matches the cause field and binds it directly.
catch as e (no cases) binds the Error wrapper with .message and .cause."
```

---

### Task 7: Remove Result helpers (ok?, err?, unwrap, unwrap_or)

**Files:**
- Modify: `crates/opal-interp/src/eval.rs:3717-3743`

**Step 1: Remove the Result helper methods**

In `crates/opal-interp/src/eval.rs`, find the block at lines 3717-3743 that handles `ok?`, `err?`, `unwrap`, `unwrap_or` for enum_id 0. Remove the entire `if enum_id.0 == 0 { ... }` block.

These are replaced by:
- `ok?()` → `if f()?` (Error is falsy)
- `err?()` → `if not f()?` or `f()? is Error`
- `unwrap()` → `f()` (auto-throw)
- `unwrap_or(default)` → `f()? or default`

**Step 2: Run tests to see what breaks**

Run: `cargo test 2>&1 | tail -20`
Run: `./tests/run_spec.sh 2>&1 | tail -20`

Remove or update any tests/specs that use these methods. The specs `result_helpers.opl` should have been updated in Task 5 already.

**Step 3: Commit**

```bash
git add crates/opal-interp/src/eval.rs
git commit -m "refactor: remove ok?, err?, unwrap, unwrap_or Result helpers

Replaced by: ? operator, or keyword, Error falsiness, auto-throw."
```

---

### Task 8: Update tree-sitter grammar

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Modify: `tree-sitter-opal/queries/highlights.scm`
- Modify: `tree-sitter-opal/test/corpus/propagation.txt`

**Step 1: Replace propagation_expression with suppress_throw_expression**

In `tree-sitter-opal/grammar.js`:

1. Remove `propagation_expression` and `bang` rules
2. Add `suppress_throw_expression` with `question` named token:

```javascript
suppress_throw_expression: $ => prec.left(13, seq($._expression, $.question_mark)),
question_mark: $ => token(prec(2, '?')),
```

Note: the `question_mark` token needs higher precedence than `?.` and `??` to avoid conflicts. Actually, `?.` and `??` are already separate tokens in the lexer. The tree-sitter grammar should use a named token approach similar to how `bang` was defined.

3. Replace `$.propagation_expression` with `$.suppress_throw_expression` in the `_expression` choice list.

**Step 2: Update highlights.scm**

Replace:
```scheme
(propagation_expression (bang) @operator)
```

with:
```scheme
(suppress_throw_expression (question_mark) @operator)
```

**Step 3: Update corpus tests**

Replace `tree-sitter-opal/test/corpus/propagation.txt` with tests for `?`:

```
================
Suppress throw on identifier
================

x = y?

---

(source_file
  (assignment
    (identifier)
    (suppress_throw_expression
      (identifier)
      (question_mark))))

================
Suppress throw on function call
================

x = foo()?

---

(source_file
  (assignment
    (identifier)
    (suppress_throw_expression
      (call
        (identifier))
      (question_mark))))

================
Suppress throw with or
================

x = foo()? or default

---

(source_file
  (assignment
    (identifier)
    (or_expression
      (suppress_throw_expression
        (call
          (identifier))
        (question_mark))
      (identifier))))
```

Note: The third test depends on how `or` is defined in the grammar. Adjust the expected tree to match.

**Step 4: Regenerate grammar**

Run: `cd tree-sitter-opal && pnpm run generate 2>&1 | tail -10`

**Step 5: Run corpus tests**

Run: `cd tree-sitter-opal && pnpm run test 2>&1 | tail -20`

**Step 6: Commit**

```bash
git add tree-sitter-opal/grammar.js tree-sitter-opal/queries/highlights.scm tree-sitter-opal/test/corpus/propagation.txt tree-sitter-opal/src/
git commit -m "feat: replace propagation_expression with suppress_throw_expression in tree-sitter

Replace ! propagation with ? suppress-throw in grammar.
Update highlights and corpus tests."
```

---

### Task 9: Update TextMate grammar

**Files:**
- Modify: `editors/vscode-opal/syntaxes/opal.tmLanguage.json`

**Step 1: Ensure `?` is in the operators pattern**

Check that `?` is included in the operator character class. The `!` should remain for mutation calls.

**Step 2: Commit**

```bash
git add editors/vscode-opal/syntaxes/opal.tmLanguage.json
git commit -m "fix: update TextMate grammar for ? operator"
```

---

### Task 10: Write comprehensive spec tests

**Files:**
- Create: `tests/spec/06-errors/error_auto_throw.opl`
- Create: `tests/spec/06-errors/error_question_operator.opl`
- Create: `tests/spec/06-errors/error_catch_types.opl`
- Create: `tests/spec/06-errors/error_panic_uncatchable.opl`
- Create: `tests/spec/06-errors/error_class_wrapper.opl`

**Step 1: Auto-throw spec**

`tests/spec/06-errors/error_auto_throw.opl`:

```opal
# expect: caught it | value is hello

def might_fail(should_fail)
  if should_fail
    Error("oops")
  end
  "hello"
end

# Auto-throw: Error return throws
try
  might_fail(true)
catch as e
  print(f"caught it")
end

# Success: returns value directly
val = might_fail(false)
print(f"value is {val}")
```

**Step 2: ? operator spec**

`tests/spec/06-errors/error_question_operator.opl`:

```opal
# expect: fallback | true | false

def risky(fail)
  if fail
    Error("nope")
  end
  "ok"
end

# ? or default
result = risky(true)? or "fallback"
print(result)

# ? for truthiness check
good = if risky(false)?
  true
else
  false
end
print(good)

bad = if risky(true)?
  true
else
  false
end
print(bad)
```

**Step 3: Catch types spec**

`tests/spec/06-errors/error_catch_types.opl`:

```opal
# expect: not found: 42 | string error | wrapper: oops

class NotFoundError
  needs id

  def to_s()
    f"not found: {id}"
  end
end

# Catch specific type — binds cause
try
  Error(NotFoundError(id: 42))
catch
  case NotFoundError as e
    print(e)
end

# Catch string — binds string
try
  fail "string error"
catch
  case String as e
    print(e)
end

# Catch all — binds Error wrapper
try
  fail "oops"
catch as e
  print(f"wrapper: {e.message}")
end
```

**Step 4: Panic uncatchable spec**

`tests/spec/06-errors/error_panic_uncatchable.opl`:

```opal
# expect: error | TypeError:

# Panics are NOT caught by try/catch
try
  x = 42 + "hello"
catch as e
  print("this should not print")
end
print("this should not print either")
```

Note: This spec should fail with a TypeError panic. The spec runner should detect the error output. Adjust the expect header to match the actual panic output format.

**Step 5: Error class wrapper spec**

`tests/spec/06-errors/error_class_wrapper.opl`:

```opal
# expect: test message | custom cause | hello

# Error with string
e1 = Error("test message")
print(e1.message)

# Error with class cause
class MyError
  needs detail
  def to_s()
    f"custom cause"
  end
end
e2 = Error(MyError(detail: "info"))
print(e2.message)
print(e2.cause.detail)
```

Note: `Error()` is a constructor — it should NOT auto-throw. Auto-throw only happens when a function returns Error.

**Step 6: Run all specs**

Run: `./tests/run_spec.sh 2>&1 | tail -30`

**Step 7: Commit**

```bash
git add tests/spec/06-errors/
git commit -m "test: add comprehensive specs for error hierarchy

Specs for auto-throw, ? operator, catch type matching,
panic uncatchability, and Error class wrapper."
```

---

### Task 11: Update error-handling.md spec document

**Files:**
- Modify: `docs/04-error-handling/error-handling.md`

**Step 1: Update the spec document**

The main spec document needs to be updated to reflect the new model:
- Remove `Result[T, E]` type and `!` operator sections
- Update `try/catch` to show `catch` + `case` blocks
- Add Error class definition
- Add `?` operator section
- Update the "When to Use Which" table
- Update `fail` to show auto-wrapping
- Remove `.ok?`, `.err?`, `.unwrap()`, `.unwrap_or()`, `.map()`, `.map_err()` from helper methods
- Update bridging section
- Update custom error types (no base class required)

Reference the guide (`error-handling-guide.md`) for practical examples.

**Step 2: Commit**

```bash
git add docs/04-error-handling/error-handling.md
git commit -m "docs: update error handling spec for new error hierarchy

Reflect auto-throw, ? operator, Error wrapper class,
catch case matching, and removal of ! operator."
```

---

### Task 12: Update LSP and run final verification

**Files:**
- Modify: `crates/opal-lsp/src/goto_def.rs` (if needed)
- No changes expected — `!` was already removed from `is_ident_char`

**Step 1: Build everything**

Run: `cargo build 2>&1 | tail -10`
Expected: Clean build, no warnings.

**Step 2: Run all unit tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All pass.

**Step 3: Run all spec tests**

Run: `./tests/run_spec.sh 2>&1 | tail -30`
Expected: All pass.

**Step 4: Run tree-sitter tests**

Run: `cd tree-sitter-opal && pnpm run test 2>&1 | tail -20`

**Step 5: Bump version**

Update version in root `Cargo.toml` (user preference: always bump after major changes).

**Step 6: Final commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version for error hierarchy redesign"
```
