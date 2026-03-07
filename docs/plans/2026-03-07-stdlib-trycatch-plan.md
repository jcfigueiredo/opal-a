# Try/Catch + Predicate Suffix + Stdlib Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix try/catch syntax, add `?` predicate suffix, add missing List/String methods.

**Architecture:** Parser changes for try/catch and `?` suffix, interpreter `call_method` additions for stdlib. TDD throughout.

**Tech Stack:** Rust

**Reference files:**
- Design: `docs/plans/2026-03-07-stdlib-trycatch-design.md`
- Parser: `crates/opal-parser/src/parser.rs` — `parse_try_catch_expr` (~line 1343), `expect_method_name` (~line 2703)
- AST: `crates/opal-parser/src/ast.rs` — `CatchClause`
- Interpreter: `crates/opal-interp/src/eval.rs` — `call_method` (~line 2442), try/catch eval (~line 1938)

---

### Task 1: Fix try/catch variable binding

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

In `crates/opal-interp/src/eval.rs` tests:

```rust
#[test]
fn try_catch_binds_variable() {
    let output = run("try\n  raise \"oops\"\ncatch e\n  print(e)\nend").unwrap();
    assert_eq!(output, "oops");
}

#[test]
fn try_catch_with_type_filter() {
    let output = run("try\n  raise \"boom\"\ncatch e as String\n  print(f\"caught: {e}\")\nend").unwrap();
    assert_eq!(output, "caught: boom");
}

#[test]
fn try_catch_ensure() {
    let output = run("x = \"start\"\ntry\n  raise \"err\"\ncatch e\n  x = f\"{x} caught\"\nensure\n  x = f\"{x} done\"\nend\nprint(x)").unwrap();
    assert_eq!(output, "start caught done");
}
```

**Step 2: Change AST**

In `ast.rs`, change `CatchClause`:

```rust
pub struct CatchClause {
    pub var_name: String,           // was Option<String> — now REQUIRED
    pub error_type: Option<String>, // unchanged
    pub body: Vec<Stmt>,
}
```

**Step 3: Fix parser**

In `parse_try_catch_expr` (~line 1349-1369), replace the catch parsing with:

```rust
while self.check(&Token::Catch) {
    self.advance(); // consume 'catch'

    // Variable is REQUIRED: catch e, catch e as ErrorType
    let var_name = self.expect_identifier()?;

    // Optional type filter: as ErrorType
    let error_type = if self.check(&Token::As) {
        self.advance();
        Some(self.expect_identifier()?)
    } else {
        None
    };

    self.expect_newline()?;
    let catch_body = self.parse_block()?;
    catches.push(CatchClause {
        var_name,
        error_type,
        body: catch_body,
    });
}
```

**Step 4: Fix interpreter**

In the try/catch eval (~line 1946-1954), update to handle the new required var_name:

```rust
if let Some(catch) = catches.first() {
    self.env.push_scope();
    // var_name is always set now
    self.env.set(catch.var_name.clone(), val.clone());

    // Check type filter if present
    if let Some(ref expected_type) = catch.error_type {
        if !self.value_is_type(&val, expected_type) {
            self.env.pop_scope();
            // Type doesn't match — re-raise
            return Err(EvalError::Raise(val));
        }
    }

    let catch_result = self.eval_block(&catch.body);
    self.env.pop_scope();
    catch_result?
}
```

Also fix the ExprKind::TryCatch match in the expression cloner (~line 2163) — update `var_name` field access.

**Step 5: Fix any existing tests that use bare `catch` or `catch Type as var`**

Search for `catch` in test strings and update syntax to `catch e` / `catch e as Type`.

**Step 6: Run all tests, commit**

```
feat: fix try/catch — variable required, catch e as Type syntax
```

---

### Task 2: Add `?` predicate suffix to identifiers

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`

**Step 1: Write failing tests**

Parser test:
```rust
#[test]
fn parse_predicate_method_def() {
    let source = "class Foo\n  needs items: List\n  def empty?()\n    .items.length() == 0\n  end\nend\n";
    let program = crate::parse(source).unwrap();
    match &program.statements[0].kind {
        StmtKind::ClassDef { methods, .. } => {
            match &methods[0].kind {
                StmtKind::FuncDef { name, .. } => assert_eq!(name, "empty?"),
                _ => panic!("expected FuncDef"),
            }
        }
        _ => panic!("expected ClassDef"),
    }
}
```

Interpreter test:
```rust
#[test]
fn predicate_method_call() {
    let output = run("class Box\n  needs items: List\n  def empty?()\n    .items.length() == 0\n  end\nend\nb = Box.new(items: [1, 2])\nprint(b.empty?())").unwrap();
    assert_eq!(output, "false");
}

#[test]
fn predicate_function_call() {
    let output = run("def adult?(age)\n  age >= 18\nend\nprint(f\"{adult?(21)} {adult?(15)}\")").unwrap();
    assert_eq!(output, "true false");
}
```

**Step 2: Implement in `expect_method_name`**

In `expect_method_name` (~line 2703), after successfully parsing an identifier, check for `?`:

```rust
fn expect_method_name(&mut self) -> Result<String, ParseError> {
    let text = self.extract_text(&self.current_span());
    match self.peek() {
        Some(Token::Identifier | Token::Send | Token::Receive | ...) => {
            self.advance();
            // Check for ? suffix (predicate methods)
            let name = if self.check(&Token::Question) {
                self.advance();
                format!("{}?", text)
            } else {
                text
            };
            Ok(name)
        }
        // ... operator tokens ...
    }
}
```

**Step 3: Handle in method call parsing**

Find where method calls are parsed (after `.identifier`). After parsing the method name identifier, check for `?`:

Search for where `Token::Dot` is handled in expression parsing, find where the method name is extracted. Add `?` suffix check there.

Also handle in function call parsing — when `identifier(args)` is parsed, if identifier is followed by `?` followed by `(`, combine them.

**Step 4: Run tests, commit**

```
feat: add ? predicate suffix for method and function names
```

---

### Task 3: Add missing List methods

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn list_contains() {
    assert_eq!(run("print([1, 2, 3].contains(2))").unwrap(), "true");
    assert_eq!(run("print([1, 2, 3].contains(5))").unwrap(), "false");
}

#[test]
fn list_first_last() {
    assert_eq!(run("print([1, 2, 3].first())").unwrap(), "1");
    assert_eq!(run("print([1, 2, 3].last())").unwrap(), "3");
    assert_eq!(run("print([].first())").unwrap(), "null");
    assert_eq!(run("print([].last())").unwrap(), "null");
}

#[test]
fn list_min_max() {
    assert_eq!(run("print([3, 1, 2].min())").unwrap(), "1");
    assert_eq!(run("print([3, 1, 2].max())").unwrap(), "3");
}

#[test]
fn list_index() {
    assert_eq!(run("print([10, 20, 30].index(20))").unwrap(), "1");
    assert_eq!(run("print([10, 20, 30].index(99))").unwrap(), "null");
}

#[test]
fn list_count() {
    assert_eq!(run("print([1, 2, 3, 4, 5].count(|x| x > 3))").unwrap(), "2");
}

#[test]
fn list_take_drop() {
    assert_eq!(run("print([1, 2, 3, 4, 5].take(3))").unwrap(), "[1, 2, 3]");
    assert_eq!(run("print([1, 2, 3, 4, 5].drop(3))").unwrap(), "[4, 5]");
}
```

**Step 2: Implement**

In `call_method`, in the `Value::List` section, add each method:

```rust
(Value::List(items), "contains") => {
    if args.len() != 1 { return Err(...); }
    let target = &args[0];
    Ok(Value::Bool(items.iter().any(|item| values_equal(item, target))))
}

(Value::List(items), "first") => {
    Ok(items.first().cloned().unwrap_or(Value::Null))
}

(Value::List(items), "last") => {
    Ok(items.last().cloned().unwrap_or(Value::Null))
}

(Value::List(items), "min") => {
    // Compare integers/floats
    // Return first item that is <= all others
}

(Value::List(items), "max") => {
    // Similar to min
}

(Value::List(items), "index") => {
    if args.len() != 1 { return Err(...); }
    let target = &args[0];
    let pos = items.iter().position(|item| values_equal(item, target));
    Ok(pos.map(|i| Value::Integer(i as i64)).unwrap_or(Value::Null))
}

(Value::List(items), "count") => {
    // Takes a closure, count matching items
    // Similar to filter but returns count
}

(Value::List(items), "take") => {
    if args.len() != 1 { return Err(...); }
    match &args[0] {
        Value::Integer(n) => Ok(Value::List(items.iter().take(*n as usize).cloned().collect())),
        _ => Err(...)
    }
}

(Value::List(items), "drop") => {
    if args.len() != 1 { return Err(...); }
    match &args[0] {
        Value::Integer(n) => Ok(Value::List(items.iter().skip(*n as usize).cloned().collect())),
        _ => Err(...)
    }
}
```

For `count`, you need to call the closure for each item (similar to `filter`). Look at how `filter` is implemented and follow the same pattern.

For `min`/`max`, compare using the existing `eval_binary_op` with `BinOp::Lt`.

**Step 3: Run tests, commit**

```
feat: add List methods — contains, first, last, min, max, index, count, take, drop
```

---

### Task 4: Add missing String methods

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn string_upcase_downcase() {
    assert_eq!(run(r#"print("hello".upcase())"#).unwrap(), "HELLO");
    assert_eq!(run(r#"print("HELLO".downcase())"#).unwrap(), "hello");
}

#[test]
fn string_slice() {
    assert_eq!(run(r#"print("hello world".slice(0, 5))"#).unwrap(), "hello");
    assert_eq!(run(r#"print("hello".slice(1, 3))"#).unwrap(), "el");
}

#[test]
fn string_index() {
    assert_eq!(run(r#"print("hello".index("ll"))"#).unwrap(), "2");
    assert_eq!(run(r#"print("hello".index("xyz"))"#).unwrap(), "null");
}
```

**Step 2: Implement**

In `call_method`, in the `Value::String` section:

```rust
(Value::String(s), "upcase") => Ok(Value::String(s.to_uppercase())),

(Value::String(s), "downcase") => Ok(Value::String(s.to_lowercase())),

(Value::String(s), "slice") => {
    if args.len() != 2 { return Err(...); }
    match (&args[0], &args[1]) {
        (Value::Integer(start), Value::Integer(end)) => {
            let start = *start as usize;
            let end = (*end as usize).min(s.len());
            Ok(Value::String(s.chars().skip(start).take(end - start).collect()))
        }
        _ => Err(...)
    }
}

(Value::String(s), "index") => {
    if args.len() != 1 { return Err(...); }
    match &args[0] {
        Value::String(needle) => {
            Ok(s.find(needle.as_str())
                .map(|i| Value::Integer(i as i64))
                .unwrap_or(Value::Null))
        }
        _ => Err(...)
    }
}
```

**Step 3: Run tests, commit**

```
feat: add String methods — upcase, downcase, slice, index
```

---

### Task 5: Add `empty?` methods (depends on Task 2)

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn list_empty_predicate() {
    assert_eq!(run("print([].empty?())").unwrap(), "true");
    assert_eq!(run("print([1].empty?())").unwrap(), "false");
}

#[test]
fn string_empty_predicate() {
    assert_eq!(run(r#"print("".empty?())"#).unwrap(), "true");
    assert_eq!(run(r#"print("hi".empty?())"#).unwrap(), "false");
}
```

**Step 2: Implement**

```rust
(Value::List(items), "empty?") => Ok(Value::Bool(items.is_empty())),
(Value::String(s), "empty?") => Ok(Value::Bool(s.is_empty())),
```

**Step 3: Run all tests, commit**

```
feat: add empty?() for List and String
```

---

### Task 6: Spec tests for all new features

**Files:**
- Create: `tests/spec/04-error-handling/try_catch_binding.opl`
- Create: `tests/spec/03-collections/list_methods.opl`
- Create: `tests/spec/03-collections/string_methods.opl`
- Create: `tests/spec/03-functions-and-types/predicate_methods.opl`

**Step 1: Write specs**

`try_catch_binding.opl`:
```opal
# expect: oops | caught: boom | start caught done

try
  raise "oops"
catch e
  print(e)
end

try
  raise "boom"
catch e as String
  print(f"caught: {e}")
end

x = "start"
try
  raise "err"
catch e
  x = f"{x} caught"
ensure
  x = f"{x} done"
end
print(x)
```

(Note: this uses 3 separate print statements joined by `|` — adjust to single results array if the spec runner expects single-line output.)

`list_methods.opl`:
```opal
# expect: true | false | 1 | 3 | 1 | 3 | 1 | null | 2 | [1, 2, 3] | [4, 5]

results = [
  f"{[1,2,3].contains(2)}",
  f"{[1,2,3].contains(5)}",
  f"{[1,2,3].first()}",
  f"{[1,2,3].last()}",
  f"{[3,1,2].min()}",
  f"{[3,1,2].max()}",
  f"{[10,20,30].index(20)}",
  f"{[10,20,30].index(99)}",
  f"{[1,2,3,4,5].count(|x| x > 3)}",
  f"{[1,2,3,4,5].take(3)}",
  f"{[1,2,3,4,5].drop(3)}"
]

print(results.join(" | "))
```

`string_methods.opl`:
```opal
# expect: HELLO | hello | ell | 2 | null

results = [
  "hello".upcase(),
  "HELLO".downcase(),
  "hello".slice(1, 4),
  f"{"hello".index("ll")}",
  f"{"hello".index("xyz")}"
]

print(results.join(" | "))
```

`predicate_methods.opl`:
```opal
# expect: true | false | true | false | true | false

results = [
  f"{[].empty?()}",
  f"{[1].empty?()}",
  f"{"".empty?()}",
  f"{"hi".empty?()}",
  f"{[1,2,3].any?(|x| x > 2)}",
  f"{[1,2,3].all?(|x| x > 2)}"
]

print(results.join(" | "))
```

**Step 2: Rebuild LSP and Cursor extension**

```bash
cargo build --release -p opal-lsp
./scripts/setup-cursor-extension.sh
```

**Step 3: Run full suite**

```bash
cargo test --workspace && ./tests/run_spec.sh
```

**Step 4: Commit**

```
test: add specs for try/catch, list methods, string methods, predicates
```

---

## Summary

| Task | Deliverable |
|------|-------------|
| 1 | Fix try/catch: `catch e`, `catch e as Type`, no bare catch |
| 2 | `?` suffix: `def empty?()`, `list.any?()`, `adult?()` |
| 3 | List: `contains`, `first`, `last`, `min`, `max`, `index`, `count`, `take`, `drop` |
| 4 | String: `upcase`, `downcase`, `slice`, `index` |
| 5 | `empty?()` for List and String (depends on Task 2) |
| 6 | Spec tests + LSP rebuild |
