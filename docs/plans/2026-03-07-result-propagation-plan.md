# Result Propagation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `!` propagation operator, Result helper methods, and unify `!`/`?` token handling.

**Architecture:** Remove `!` from lexer identifier regex (consistency with `?`). Parser combines `identifier` + `!` + `(` into method calls. Parser adds `Propagate` postfix expression for `expr!` without `(`. Interpreter unwraps Ok or returns Error. Helper methods added to `call_method` for EnumVariant values.

**Tech Stack:** Rust

**Reference files:**
- Design: `docs/plans/2026-03-07-result-propagation-design.md`
- Lexer: `crates/opal-lexer/src/token.rs` — identifier regex (~line 360), Bang token (~line 431)
- Parser: `crates/opal-parser/src/parser.rs` — `expect_method_name` (~line 2703), primary expression parsing, postfix expression parsing
- AST: `crates/opal-parser/src/ast.rs` — `ExprKind`
- Interpreter: `crates/opal-interp/src/eval.rs` — `register_builtin_enums` (~line 279), `call_method` (~line 2442)
- Runtime: `crates/opal-runtime/src/value.rs` — `Value::EnumVariant { enum_id, variant_index, fields }`

**Key internal detail:** `Ok(v)` = `Value::EnumVariant { enum_id: EnumId(0), variant_index: 0, fields: [v] }`. `Error(e)` = `Value::EnumVariant { enum_id: EnumId(0), variant_index: 1, fields: [e] }`. The Result enum is at index 0 in `self.enums`.

---

### Task 1: Remove `!` from identifier regex

**Files:**
- Modify: `crates/opal-lexer/src/token.rs`
- Modify: `crates/opal-parser/src/parser.rs`

**Step 1: Remove `!` from regex**

In `crates/opal-lexer/src/token.rs`, find the identifier regex (search for `!?"]`). Change `!?` at the end to nothing:

Before: `...]*!?")]`
After: `...]*")]`

The `!` suffix is now handled by the parser (same as `?`), via the existing `Bang` token at line ~431.

**Step 2: Update parser to combine `identifier` + `!`**

In `expect_method_name` (~line 2703), the function already combines `identifier` + `?` → `name?`. Add the same for `!`:

After the `?` check, add:
```rust
let name = if self.check(&Token::Bang) {
    self.advance();
    format!("{}!", name)
} else {
    name
};
```

Wait — the function already handles `?`. Just extend it to also handle `!`. After the `Question` check block, add a `Bang` check:
```rust
// Already exists for ?:
let name = if self.check(&Token::Question) {
    self.advance();
    format!("{}?", name)
} else if self.check(&Token::Bang) {
    self.advance();
    format!("{}!", name)
} else {
    name
};
```

**Step 3: Handle `identifier!()` in expression parsing**

In the primary expression parser (where `Token::Identifier` is handled), the code already combines identifier + `?` when followed by `(`. Add the same for `!`:

Find where `Token::Question` is checked after identifier in expression parsing. Add `Token::Bang` with the same logic: if `Bang` is followed by `LParen`, combine into `name!` and parse as call.

**IMPORTANT:** If `Bang` is NOT followed by `LParen`, do NOT combine — leave `Bang` for the propagation operator (Task 3).

**Step 4: Handle `method!()` in method call parsing**

In member access / method call parsing (after `.`), the method name is extracted. After extracting it, check for `!` (same as `?` check). If `Bang` is followed by `(`, combine into `name!`. If not, leave for propagation.

**Step 5: Run tests — all 224 unit tests + 83 spec tests must pass**

Run: `cargo test --workspace && ./tests/run_spec.sh`

**Step 6: Commit**

```
refactor: remove ! from identifier regex, handle in parser like ?
```

---

### Task 2: Add `Propagate` to AST and parse `expr!`

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`

**Step 1: Add to AST**

In `ExprKind` enum in ast.rs, add:
```rust
/// Result propagation: expr! — unwraps Ok or returns Err
Propagate(Box<Expr>),
```

**Step 2: Write failing parser test**

```rust
#[test]
fn parse_propagation() {
    let source = "x = foo()!\n";
    let program = crate::parse(source).unwrap();
    match &program.statements[0].kind {
        StmtKind::Assign { value, .. } => {
            assert!(matches!(&value.kind, ExprKind::Propagate(_)));
        }
        _ => panic!("expected Assign"),
    }
}
```

**Step 3: Implement postfix `!` parsing**

In the expression parser, after parsing a complete expression (call, member access, index, etc.), check for `Bang` token. This should be in the postfix parsing loop — find where `.`, `(`, `[` are handled as postfix operators on expressions.

Add at the end of the postfix loop:
```rust
// Propagation: expr!
if self.check(&Token::Bang) {
    // Only if NOT followed by ( — that would be name!() call
    let next_after_bang = self.peek_at(self.pos + 1);
    if next_after_bang != Some(&Token::LParen) {
        self.advance(); // consume !
        expr = Expr {
            kind: ExprKind::Propagate(Box::new(expr)),
            span: Span { start: expr.span.start, end: self.previous_span().end },
        };
        continue; // continue postfix loop
    }
}
```

**IMPORTANT:** Propagation must NOT fire when `!` is part of `name!(args)`. The check `next is not LParen` handles this. But be careful with context: `foo()!` should propagate (the `!` comes after `)`, not after an identifier). The key: only check for `name!()` combining when the expression IS an identifier. If the expression is a call result, member access, etc., `!` is always propagation.

Actually, simpler approach: the `identifier!()` combining happens in the PRIMARY expression parsing (before postfix). The postfix `!` happens AFTER. So by the time we reach postfix, `save!(42)` is already parsed as a call to `save!`. The postfix `!` only fires on complete expressions like `foo()!`, `result!`, `obj.method!`.

**Step 4: Add stub in interpreter**

In eval_expr, add:
```rust
ExprKind::Propagate(expr) => {
    Err(EvalError::RuntimeError("propagation not yet implemented".into()))
}
```

**Step 5: Run parser test, commit**

```
feat: add Propagate AST node and parse expr! syntax
```

---

### Task 3: Implement `!` propagation in interpreter

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn propagation_unwraps_ok() {
    let output = run("def foo()\n  Ok(42)\nend\nx = foo()!\nprint(x)").unwrap();
    assert_eq!(output, "42");
}

#[test]
fn propagation_returns_error() {
    let output = run("def foo()\n  Error(\"fail\")\nend\ndef bar()\n  x = foo()!\n  Ok(x + 1)\nend\nresult = bar()\nmatch result\n  case Error(msg)\n    print(f\"err: {msg}\")\n  case Ok(v)\n    print(f\"ok: {v}\")\nend").unwrap();
    assert_eq!(output, "err: fail");
}

#[test]
fn propagation_chained() {
    let output = run("def step1()\n  Ok(10)\nend\ndef step2(n)\n  Ok(n * 2)\nend\ndef process()\n  a = step1()!\n  b = step2(a)!\n  Ok(b + 1)\nend\nmatch process()\n  case Ok(v)\n    print(v)\n  case Error(e)\n    print(e)\nend").unwrap();
    assert_eq!(output, "21");
}

#[test]
fn propagation_on_non_result_errors() {
    let result = run("x = 42!\nprint(x)");
    assert!(result.is_err());
}
```

**Step 2: Implement**

Replace the stub in `ExprKind::Propagate`:

```rust
ExprKind::Propagate(inner) => {
    let val = self.eval_expr(inner)?;
    match &val {
        Value::EnumVariant { enum_id, variant_index, fields } => {
            // Check if this is the Result enum (index 0)
            if enum_id.0 == 0 {
                if *variant_index == 0 {
                    // Ok(value) — unwrap and return the value
                    Ok(fields.first().cloned().unwrap_or(Value::Null))
                } else {
                    // Error(value) — propagate by returning from function
                    Err(EvalError::Return(val))
                }
            } else {
                Err(EvalError::TypeError(
                    "! operator requires an Ok or Error value".into()
                ))
            }
        }
        _ => Err(EvalError::TypeError(
            "! operator requires an Ok or Error value".into()
        )),
    }
}
```

**Step 3: Run tests, commit**

```
feat: implement ! propagation operator for Result types
```

---

### Task 4: Add Result helper methods

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn result_ok_predicate() {
    assert_eq!(run("print(Ok(42).ok?())").unwrap(), "true");
    assert_eq!(run("print(Error(\"x\").ok?())").unwrap(), "false");
}

#[test]
fn result_err_predicate() {
    assert_eq!(run("print(Ok(42).err?())").unwrap(), "false");
    assert_eq!(run("print(Error(\"x\").err?())").unwrap(), "true");
}

#[test]
fn result_unwrap_ok() {
    assert_eq!(run("print(Ok(42).unwrap())").unwrap(), "42");
}

#[test]
fn result_unwrap_error_raises() {
    let result = run("Ok(1)\nError(\"boom\").unwrap()");
    assert!(result.is_err());
}

#[test]
fn result_unwrap_or() {
    assert_eq!(run("print(Ok(42).unwrap_or(0))").unwrap(), "42");
    assert_eq!(run("print(Error(\"x\").unwrap_or(0))").unwrap(), "0");
}
```

**Step 2: Implement**

In `call_method`, add a section for `Value::EnumVariant` matching the Result enum (enum_id 0):

```rust
(Value::EnumVariant { enum_id, variant_index, fields }, method) if enum_id.0 == 0 => {
    match method {
        "ok?" => Ok(Value::Bool(*variant_index == 0)),
        "err?" => Ok(Value::Bool(*variant_index == 1)),
        "unwrap" => {
            if *variant_index == 0 {
                // Ok — return inner value
                Ok(fields.first().cloned().unwrap_or(Value::Null))
            } else {
                // Error — raise the error value
                let err_val = fields.first().cloned().unwrap_or(Value::Null);
                Err(EvalError::Raise(err_val))
            }
        }
        "unwrap_or" => {
            if args.len() != 1 {
                return Err(EvalError::TypeError("unwrap_or() takes 1 argument".into()));
            }
            if *variant_index == 0 {
                Ok(fields.first().cloned().unwrap_or(Value::Null))
            } else {
                Ok(args.into_iter().next().unwrap())
            }
        }
        _ => Err(EvalError::TypeError(format!("no method '{}' on Result", method))),
    }
}
```

**IMPORTANT:** This match arm must come BEFORE the general enum method dispatch to avoid conflicts.

**Step 3: Run tests, commit**

```
feat: add Result helper methods — ok?, err?, unwrap, unwrap_or
```

---

### Task 5: Update tree-sitter and TextMate grammars

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Modify: `tree-sitter-opal/queries/highlights.scm`
- Modify: `editors/vscode-opal/syntaxes/opal.tmLanguage.json`

**Step 1: Add propagation to tree-sitter**

In grammar.js, add `$.propagation_expression` to `_expression`:
```javascript
propagation_expression: $ => prec(12, seq($._expression, '!')),
```

Remove `!?` from the identifier regex:
```javascript
identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
```
(Remove the `!?` at the end)

**Step 2: Regenerate and test**

```bash
cd tree-sitter-opal && pnpm run generate && pnpm run test
```

**Step 3: Update TextMate**

In `opal.tmLanguage.json`, update the function name and identifier patterns to not include `!` in the regex. The `!` will be matched separately as an operator.

**Step 4: Rebuild Cursor extension**

```bash
./scripts/setup-cursor-extension.sh
```

**Step 5: Commit**

```
feat: add propagation operator to tree-sitter and TextMate grammars
```

---

### Task 6: Spec tests

**Files:**
- Create: `tests/spec/06-errors/result_propagation.opl`
- Create: `tests/spec/06-errors/result_helpers.opl`

**Step 1: Write specs**

`result_propagation.opl`:
```opal
# expect: 42 | err: fail | 21

def returns_ok()
  Ok(42)
end

def returns_err()
  Error("fail")
end

def chain_ok()
  a = returns_ok()!
  Ok(a / 2)
end

def chain_err()
  a = returns_err()!
  Ok(a + 1)
end

# Direct unwrap
x = returns_ok()!

# Propagation stops at Error
result_err = chain_err()

# Chained propagation
result_ok = chain_ok()

results = [
  f"{x}",
  match result_err
    case Error(msg) then f"err: {msg}"
    case Ok(v) then f"ok: {v}"
  end,
  match result_ok
    case Ok(v) then f"{v}"
    case Error(e) then f"err: {e}"
  end
]

print(results.join(" | "))
```

`result_helpers.opl`:
```opal
# expect: true | false | false | true | 42 | 0 | 99

ok = Ok(42)
err = Error("nope")

results = [
  f"{ok.ok?()}",
  f"{ok.err?()}",
  f"{err.ok?()}",
  f"{err.err?()}",
  f"{ok.unwrap()}",
  f"{err.unwrap_or(0)}",
  f"{Ok(99).unwrap_or(0)}"
]

print(results.join(" | "))
```

**Step 2: Run full suite**

```bash
cargo test --workspace && ./tests/run_spec.sh
```

**Step 3: Commit**

```
test: add specs for Result propagation and helper methods
```

---

## Summary

| Task | Deliverable |
|------|-------------|
| 1 | Remove `!` from identifier regex, parser handles `name!()` like `name?()` |
| 2 | `ExprKind::Propagate`, parse `expr!` as postfix |
| 3 | Interpreter: unwrap Ok or return Error on `!` |
| 4 | `ok?()`, `err?()`, `unwrap()`, `unwrap_or()` methods |
| 5 | Tree-sitter + TextMate grammar updates |
| 6 | Spec tests |
