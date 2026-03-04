# Language Gaps Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement all features documented in docs/01-03 that are feasible in the tree-walk interpreter — 30+ features across 4 phases bringing the interpreter to full language spec coverage for basics, control flow, and functions/types.

**Architecture:** Each phase builds on the last. Phase A adds core language primitives (break/next, indexing, compound assignment). Phase B adds collection methods and pattern matching extensions. Phase C adds type system features (iterators, visibility, casting, operator overloading). Phase D adds advanced features (models, retroactive conformance, exhaustiveness, f-string specs). All changes are additive — no execution model changes.

**Tech Stack:** Rust workspace (opal-lexer, opal-parser, opal-interp, opal-runtime, opal-stdlib). Logos lexer, recursive descent parser, tree-walk interpreter.

---

## Reference: Current File Layout

- **Lexer tokens:** `crates/opal-lexer/src/token.rs` — `Token::Not` (line 290), `Token::Is` (line 292), `Token::Pipe` (line 372), `Token::QuestionDot` (line 380), `Token::QuestionQuestion` (line 382)
- **AST:** `crates/opal-parser/src/ast.rs` — `StmtKind`, `ExprKind`, `Pattern`, `BinOp`, `Param` (line 290 has `default: Option<Expr>`)
- **Parser:** `crates/opal-parser/src/parser.rs` — `parse_statement()` (line 51), `parse_expression()` (line 1380), `parse_postfix()` (line 1476), `peek_binary_op()` (line 2219), `op_precedence()` (line 2249)
- **Environment:** `crates/opal-runtime/src/env.rs` — `Environment` struct (line 6), `set()` (line 27), `assign()` (line 35)
- **Values:** `crates/opal-runtime/src/value.rs` — `Value` enum
- **Interpreter:** `crates/opal-interp/src/eval.rs` — `eval_stmt` (line 434), `eval_expr` (line 892), `eval_call` (line 1543), `call_method` (line 1706), `match_pattern` (line 1405), `values_equal` (line 2977), `call_function` (line 2613)
- **Spec tests:** `tests/spec/` with `# expect:` headers, run via `bash tests/run_spec.sh`

## Reference: Running Tests

```bash
cargo test                           # All unit tests (138 across all crates)
bash tests/run_spec.sh               # Spec tests (83 .opl files with # expect: headers)
cargo test -p opal-interp <name>     # Single unit test
cargo run -- run tests/spec/file.opl # Single spec test
```

---

## PHASE A: Core Language Gaps

---

## Task 1: Add `break` and `next` to Loops

**Files:**
- Modify: `crates/opal-lexer/src/token.rs`
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/02-control-flow/break_next.opl`

**Step 1: Add tokens to lexer**

In `crates/opal-lexer/src/token.rs`, add after `Token::Reply` (line 272):

```rust
#[token("break")]
Break,
#[token("next")]
Next,
```

**Step 2: Add AST nodes**

In `crates/opal-parser/src/ast.rs`, add to `StmtKind`:

```rust
/// Break out of a loop
Break,
/// Skip to next loop iteration
Next,
```

**Step 3: Add EvalError variants**

In `crates/opal-interp/src/eval.rs`, add to `EvalError`:

```rust
#[error("break")]
Break,
#[error("next")]
Next,
```

**Step 4: Parse break/next in parser**

In `parse_statement()`, add before the instance variable assignment check:

```rust
// Break
if self.check(&Token::Break) {
    self.advance();
    self.expect_newline()?;
    return Ok(Stmt { kind: StmtKind::Break, span: Span { start: start.start, end: self.previous_span().end } });
}

// Next
if self.check(&Token::Next) {
    self.advance();
    self.expect_newline()?;
    return Ok(Stmt { kind: StmtKind::Next, span: Span { start: start.start, end: self.previous_span().end } });
}
```

**Step 5: Handle in eval_stmt**

Add to eval_stmt:

```rust
StmtKind::Break => {
    return Err(EvalError::Break);
}
StmtKind::Next => {
    return Err(EvalError::Next);
}
```

**Step 6: Update loop handlers to catch Break/Next**

Update the `For` loop handler (around line 488). Replace `result?;` with:

```rust
match result {
    Ok(_) => {}
    Err(EvalError::Break) => break,
    Err(EvalError::Next) => continue,
    Err(e) => return Err(e),
}
```

Do the same for the Range iteration branch and the While loop handler.

**Step 7: Write spec test**

Create `tests/spec/02-control-flow/break_next.opl`:
```opal
# expect: 1 2 3 | 1 3 5
items = []
for i in 1..10
  if i > 3
    break
  end
  items = items.push(i)
end

evens_skipped = []
for i in 1..6
  if i % 2 == 0
    next
  end
  evens_skipped = evens_skipped.push(i)
end

print(f"{items.join(' ')} | {evens_skipped.join(' ')}")
```

Note: This test depends on `.join()` from Task 9. Write a simpler test first if implementing in order:

```opal
# expect: 3 | 3
sum = 0
for i in 1..10
  if i > 3
    break
  end
  sum = sum + 1
end
count = 0
for i in 1..6
  if i % 2 == 0
    next
  end
  count = count + 1
end
print(f"{sum} | {count}")
```

**Step 8: Write unit tests**

```rust
#[test]
fn break_in_for() {
    assert_eq!(run("sum = 0\nfor i in 1..10\n  if i > 3\n    break\n  end\n  sum = sum + 1\nend\nprint(sum)").unwrap(), "3");
}

#[test]
fn next_in_for() {
    assert_eq!(run("sum = 0\nfor i in 1..6\n  if i % 2 == 0\n    next\n  end\n  sum = sum + i\nend\nprint(sum)").unwrap(), "9");
}

#[test]
fn break_in_while() {
    assert_eq!(run("i = 0\nwhile true\n  i = i + 1\n  if i == 5\n    break\n  end\nend\nprint(i)").unwrap(), "5");
}
```

**Step 9: Run all tests**

Run: `cargo test && bash tests/run_spec.sh`

**Step 10: Commit**

```bash
git add crates/ tests/
git commit -m "feat: implement break and next for loops"
```

---

## Task 2: Compound Assignment (`+=`, `-=`, `*=`, `/=`)

**Files:**
- Modify: `crates/opal-lexer/src/token.rs`
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/compound_assign.opl`

**Step 1: Add tokens**

In token.rs, add after `Token::Eq`:

```rust
#[token("+=")]
PlusEq,
#[token("-=")]
MinusEq,
#[token("*=")]
StarEq,
#[token("/=")]
SlashEq,
```

**Step 2: Add AST node**

In ast.rs, add to `StmtKind`:

```rust
/// Compound assignment: `x += 1`
CompoundAssign { name: String, op: BinOp, value: Expr },
```

**Step 3: Parse compound assignment**

In the parser, the tricky part is that `x += 1` starts with an identifier. After parsing an expression, check if the next token is a compound assignment operator. In `parse_statement()`, after the expression is parsed in the fallback case, check:

In the section where `parse_statement` falls through to expression parsing (near the end), when we have an identifier followed by `+=` etc., parse it as CompoundAssign. This is similar to how regular assignment is handled — look for the pattern `Identifier CompoundOp Expr`.

**Step 4: Handle in eval_stmt**

```rust
StmtKind::CompoundAssign { name, op, value } => {
    let current = self.env.get(name).cloned()
        .ok_or_else(|| EvalError::UndefinedVariable(name.clone()))?;
    let rhs = self.eval_expr(value)?;
    let result = eval_binary_op(*op, current, rhs)?;
    self.env.assign(name.clone(), result);
}
```

**Step 5: Write spec test and unit tests**

`tests/spec/01-basics/compound_assign.opl`:
```opal
# expect: 15 | 5 | 50 | 5
x = 10
x += 5
r1 = x
x -= 10
r2 = x
x *= 10
r3 = x
x /= 10
r4 = x
print(f"{r1} | {r2} | {r3} | {r4}")
```

**Step 6: Run tests and commit**

---

## Task 3: Default Parameter Values

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/02-functions/default_params.opl`

**Step 1: Update call_function**

In `call_function` (around line 2617), replace the strict param count check with default-aware logic:

```rust
fn call_function(&mut self, id: FunctionId, name: &str, mut arg_values: Vec<Value>) -> Result<Value, EvalError> {
    let stored = self.functions[id.0].clone();

    // Fill missing args with defaults
    if arg_values.len() < stored.params.len() {
        let param_ast = // need to store Param defaults...
    }
```

Actually, `StoredFunction` currently only stores param names, not defaults. We need to also store the default expressions. Add `param_defaults: Vec<Option<Expr>>` to `StoredFunction`. Populate it from `params.iter().map(|p| p.default.clone())` everywhere StoredFunction is created.

Then in `call_function`:
```rust
if arg_values.len() < stored.params.len() {
    for i in arg_values.len()..stored.params.len() {
        if let Some(default_expr) = &stored.param_defaults[i] {
            let val = self.eval_expr(default_expr)?;
            arg_values.push(val);
        } else {
            return Err(EvalError::TypeError(format!(
                "{}() expected {} arguments, got {}",
                name, stored.params.len(), arg_values.len()
            )));
        }
    }
}
```

**Step 2: Write spec test**

```opal
# expect: localhost:8080 | localhost:3000
def connect(host, port = 8080)
  f"{host}:{port}"
end
print(f"{connect('localhost')} | {connect('localhost', 3000)}")
```

**Step 3: Run tests and commit**

---

## Task 4: List/Dict/String Indexing (`[]` and `[]=`)

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/indexing.opl`

**Step 1: Add AST nodes**

In ast.rs, add to `ExprKind`:
```rust
/// Index access: `expr[expr]`
Index { object: Box<Expr>, index: Box<Expr> },
```

Add to `StmtKind`:
```rust
/// Index assignment: `expr[expr] = expr`
IndexAssign { object: Expr, index: Expr, value: Expr },
```

**Step 2: Parse indexing in postfix**

In `parse_postfix()`, add a branch for `Token::LBracket` alongside the existing `LParen` and `Dot` branches:

```rust
} else if self.check(&Token::LBracket) {
    self.advance();
    let index = self.parse_expression(0)?;
    self.expect_token(&Token::RBracket, "]")?;
    let span = Span { start: expr.span.start, end: self.previous_span().end };
    expr = Expr {
        kind: ExprKind::Index { object: Box::new(expr), index: Box::new(index) },
        span,
    };
}
```

**Step 3: Parse index assignment**

In `parse_statement()`, after parsing an expression, if it's an `Index` and followed by `=`, parse as `IndexAssign`.

**Step 4: Evaluate indexing**

In `eval_expr`, handle `ExprKind::Index`:

```rust
ExprKind::Index { object, index } => {
    let obj = self.eval_expr(object)?;
    let idx = self.eval_expr(index)?;
    match (&obj, &idx) {
        (Value::List(items), Value::Integer(i)) => {
            let i = if *i < 0 { items.len() as i64 + i } else { *i } as usize;
            Ok(items.get(i).cloned().unwrap_or(Value::Null))
        }
        (Value::Dict(entries), Value::String(key)) => {
            Ok(entries.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or(Value::Null))
        }
        (Value::String(s), Value::Integer(i)) => {
            let i = if *i < 0 { s.len() as i64 + i } else { *i } as usize;
            Ok(s.chars().nth(i).map(|c| Value::String(c.to_string())).unwrap_or(Value::Null))
        }
        _ => Err(EvalError::TypeError("invalid index operation".into())),
    }
}
```

In `eval_stmt`, handle `StmtKind::IndexAssign` similarly — read the collection, set at the index, write back.

**Step 5: Write spec test**

```opal
# expect: b | 2 | h | c
list = [1, 2, 3]
dict = {"a": 1, "b": 2}
s = "hello"
print(f"{dict["b"]} | {list[1]} | {s[0]} | {list[-1]}")
```

Wait — dict literal syntax uses `{key: val}` not `{"key": val}`. Let me adjust:

```opal
# expect: 2 | b | h
list = ["a", "b", "c"]
print(f"{list[1]} | {list[-1]} | {"hello"[0]}")
```

Actually simpler:
```opal
# expect: 2 | c | h
list = [1, 2, 3]
r1 = list[1]
r2 = list[-1]
s = "hello"
r3 = s[0]
print(f"{r1} | {r2} | {r3}")
```

**Step 6: Run tests and commit**

---

## Task 5: `in` / `not in` Membership Operator

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/in_operator.opl`

**Step 1: Add BinOp variants**

In ast.rs, add to `BinOp`:
```rust
In,
NotIn,
```

**Step 2: Parse `in` and `not in`**

In `peek_binary_op()`, add:
```rust
Some(Token::In) => Some(BinOp::In),
```

In `op_precedence()`:
```rust
BinOp::In | BinOp::NotIn => (4, Assoc::Left),
```

Handle `not in` as two-token operator (like `is not`): after consuming `in`, check if preceded by `not`. Actually, `not in` is tricky because `not` is a unary prefix. Better approach: in the binary op loop, after consuming `In`, check next token for `not`. Wait — it's `not in`, so `not` comes first.

Alternative: handle in `parse_expression` — when we see `not` followed by `in`, emit `BinOp::NotIn`. In `peek_binary_op`, when we see `Token::Not`, peek ahead for `Token::In`:

```rust
Some(Token::Not) if self.peek_ahead(1) == Some(&Token::In) => Some(BinOp::NotIn),
```

And in the expression parser, when the op is `NotIn`, advance an extra time to consume the `in` token.

**Step 3: Evaluate**

In `eval_expr`'s BinaryOp handler, add special handling for `In`/`NotIn` (similar to `Is`):

```rust
if *op == BinOp::In || *op == BinOp::NotIn {
    let lval = self.eval_expr(left)?;
    let rval = self.eval_expr(right)?;
    let result = match &rval {
        Value::List(items) => items.iter().any(|item| values_equal(&lval, item)),
        Value::Dict(entries) => {
            if let Value::String(key) = &lval {
                entries.iter().any(|(k, _)| k == key)
            } else { false }
        }
        Value::String(s) => {
            if let Value::String(sub) = &lval { s.contains(sub.as_str()) } else { false }
        }
        Value::Range { start, end, inclusive } => {
            if let Value::Integer(n) = &lval {
                if *inclusive { *n >= *start && *n <= *end } else { *n >= *start && *n < *end }
            } else { false }
        }
        _ => false,
    };
    return Ok(Value::Bool(if *op == BinOp::In { result } else { !result }));
}
```

**Step 4: Write tests and commit**

---

## Task 6: Null-Safe `?.` and `??` Operators

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/null_safe.opl`

**Step 1: Add AST nodes**

In `ExprKind`, add:
```rust
/// Null-safe member access: `expr?.field`
NullSafeMemberAccess { object: Box<Expr>, field: String },
/// Null coalescing: `expr ?? default`
NullCoalesce { left: Box<Expr>, right: Box<Expr> },
```

**Step 2: Parse `?.` in postfix**

In `parse_postfix()`, add a branch for `Token::QuestionDot`:
```rust
} else if self.check(&Token::QuestionDot) {
    self.advance();
    let field = self.expect_method_name()?;
    let span = Span { start: expr.span.start, end: self.previous_span().end };
    expr = Expr {
        kind: ExprKind::NullSafeMemberAccess { object: Box::new(expr), field },
        span,
    };
}
```

**Step 3: Parse `??` as binary op**

In `peek_binary_op`:
```rust
Some(Token::QuestionQuestion) => Some(BinOp::NullCoalesce),
```

Add `NullCoalesce` to BinOp enum. Precedence: just above Or (level 1.5, or use 2 and bump Or to 1):

```rust
BinOp::NullCoalesce => (2, Assoc::Right),  // bump others up
```

Actually simpler: give it precedence 1 (below Or's current 1). Or handle it as a special expression form like pipe.

**Step 4: Evaluate**

`NullSafeMemberAccess`: evaluate object, if null return null, else evaluate as normal MemberAccess.

`NullCoalesce` (or BinOp): evaluate left, if not null return it, else evaluate right. Short-circuit (don't evaluate right if left is non-null).

**Step 5: Write tests and commit**

---

## Task 7: Suffix `if`

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`
- Create: `tests/spec/02-control-flow/suffix_if.opl`

**Step 1: Parse suffix if**

In `parse_statement()`, after the expression-statement fallback (the last branch), before calling `expect_newline`, check if the next token is `Token::If`. If so, parse the condition and wrap:

```rust
// After parsing expression statement:
if self.check(&Token::If) {
    self.advance();
    let condition = self.parse_expression(0)?;
    // Wrap: execute stmt only if condition is true
    let stmt = Stmt { kind: StmtKind::Expr(expr), span };
    return Ok(Stmt {
        kind: StmtKind::Expr(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: vec![stmt],
                elsif_branches: vec![],
                else_branch: None,
            },
            span,
        }),
        span,
    });
}
```

This also needs to work for break/next/return/assign — apply suffix if to all statement types.

**Step 2: Write tests and commit**

---

## Task 8: Parallel Assignment

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/parallel_assign.opl`

**Step 1: Add AST node**

```rust
/// Parallel assignment: `a, b = 1, 2`
ParallelAssign { names: Vec<String>, values: Vec<Expr> },
```

**Step 2: Parse**

When parsing a statement, if we see `Identifier, Identifier = ...`, parse as parallel assign. Collect names until `=`, then collect expressions.

**Step 3: Evaluate**

Evaluate all RHS expressions first, then assign to names. If single RHS evaluates to a List, destructure it.

**Step 4: Write tests and commit**

---

## Task 9: `let` Enforcement

**Files:**
- Modify: `crates/opal-runtime/src/env.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/let_immutable.opl`

**Step 1: Add frozen set to Environment**

```rust
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    frozen: HashSet<String>,
}
```

**Step 2: Update set/assign**

In `set()`: if called for a `let` binding, also insert into `frozen`.
In `assign()`: check if name is in `frozen`, return error.

Add `pub fn set_frozen(&mut self, name: String, value: Value)` that calls `set` then adds to `frozen`.

**Step 3: Update eval_stmt Let handler**

Change from `self.env.set(name, val)` to `self.env.set_frozen(name, val)`.

**Step 4: Write tests and commit**

---

## PHASE B: Collections & Patterns

---

## Task 10: Collection Methods — List

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` (call_method)
- Create: `tests/spec/03-collections/list_methods.opl`

**Step 1: Add all list methods in call_method**

Add after existing list methods (after `.reduce`):

```rust
(Value::List(items), "sort") => {
    let mut sorted = items.clone();
    if args.len() == 1 {
        // Custom comparator
        let closure_id = match &args[0] { Value::Closure(id) => *id, _ => return Err(...) };
        sorted.sort_by(|a, b| {
            let result = self.call_closure(closure_id, vec![a.clone(), b.clone()]).unwrap_or(Value::Integer(0));
            match result { Value::Integer(n) => n.cmp(&0), _ => std::cmp::Ordering::Equal }
        });
    } else {
        sorted.sort_by(|a, b| value_compare(a, b));
    }
    Ok(Value::List(sorted))
}
(Value::List(items), "reverse") => Ok(Value::List(items.iter().rev().cloned().collect())),
(Value::List(items), "find") => { /* closure, return first match or null */ }
(Value::List(items), "any?") => { /* closure, return bool */ }
(Value::List(items), "all?") => { /* closure, return bool */ }
(Value::List(items), "count") => {
    if args.is_empty() { Ok(Value::Integer(items.len() as i64)) }
    else { /* closure, count matching */ }
}
(Value::List(items), "each") => { /* closure, return null */ }
(Value::List(items), "take") => { /* n items from front */ }
(Value::List(items), "drop") => { /* skip n items */ }
(Value::List(items), "flatten") => { /* one level */ }
(Value::List(items), "zip") => { /* with another list */ }
(Value::List(items), "group_by") => { /* closure -> dict */ }
(Value::List(items), "join") => { /* separator -> string */ }
```

Add helper `fn value_compare(a: &Value, b: &Value) -> std::cmp::Ordering` for natural ordering (integers, floats, strings).

**Step 2: Write spec tests and unit tests**

**Step 3: Commit**

---

## Task 11: Collection Methods — String and Dict

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/string_methods_extended.opl`

**Step 1: Add string methods**

```rust
(Value::String(s), "to_int") => {
    Ok(s.parse::<i64>().map(Value::Integer).unwrap_or(Value::Null))
}
(Value::String(s), "to_float") => {
    Ok(s.parse::<f64>().map(Value::Float).unwrap_or(Value::Null))
}
(Value::String(s), "reverse") => Ok(Value::String(s.chars().rev().collect())),
```

Add string repeat via `*` operator in `eval_binary_op`:
```rust
(BinOp::Mul, Value::String(s), Value::Integer(n)) => {
    Ok(Value::String(s.repeat(*n as usize)))
}
```

**Step 2: Add dict methods**

```rust
(Value::Dict(entries), "has_key?") => { /* check key existence */ }
(Value::Dict(entries), "merge") => { /* merge with another dict */ }
```

**Step 3: Write tests and commit**

---

## Task 12: Match Guards

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/02-control-flow/match_guards.opl`

**Step 1: Add guard to MatchCase**

In ast.rs, update `MatchCase`:
```rust
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}
```

**Step 2: Parse guards**

After parsing the pattern, check for `Token::If`. If present, parse the guard expression before the newline.

**Step 3: Evaluate guards**

In the match expression handler, after `match_pattern` succeeds, if there's a guard, evaluate it. If falsy, skip to the next case.

**Step 4: Write tests and commit**

---

## Task 13: Or-Patterns, Dict Patterns, Range Patterns, As-Bindings

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/02-control-flow/advanced_patterns.opl`

**Step 1: Add Pattern variants**

```rust
/// Or-pattern: `1 | 2 | 3`
Or(Vec<Pattern>),
/// Dict pattern: `{key: var}`
Dict(Vec<(String, Pattern)>),
/// Range pattern: matches if value is within range
Range { start: i64, end: i64, inclusive: bool },
/// As-binding: `Pattern as name`
As(Box<Pattern>, String),
/// Typed pattern: `x: Type`
Typed(String, String),
```

**Step 2: Parse each pattern type**

- Or: after parsing a pattern, if `|` follows (inside match case), collect alternatives
- Dict: `{` starts dict pattern, parse `key: pattern` pairs
- Range: `start..end` or `start...end` as literal
- As: after any pattern, if `as` keyword follows, wrap in As
- Typed: `name: Type` parsed as Typed(name, type)

**Step 3: Evaluate each in match_pattern**

**Step 4: Write tests and commit**

---

## Task 14: List/Dict Comprehensions

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/03-collections/comprehensions.opl`

**Step 1: Add AST node**

```rust
/// List comprehension: `[expr for x in iter if cond]`
ListComprehension {
    expr: Box<Expr>,
    var: String,
    iterable: Box<Expr>,
    condition: Option<Box<Expr>>,
},
```

**Step 2: Parse**

In `parse_primary()`, when we see `[`, peek ahead: if after the first expression there's a `for` keyword, it's a comprehension, not a list literal. Parse `expr for var in iterable` with optional `if cond`.

**Step 3: Evaluate**

Iterate over the iterable, bind var, optionally check condition, evaluate expr, collect into list.

**Step 4: Write tests and commit**

---

## Task 15: Destructuring in Assignment, For-Loops, Closures

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/destructuring.opl`

**Step 1: Add DestructureAssign**

```rust
/// Destructure assignment: `[a, b | rest] = list`
DestructureAssign { pattern: Pattern, value: Expr },
```

**Step 2: Parse list destructuring on LHS**

When `parse_statement` sees `[`, it could be a destructuring assignment. Parse as a pattern, then expect `=`, then parse expression.

**Step 3: For-loop destructuring**

Extend `StmtKind::For` to use a `Pattern` instead of a `String` for the variable. In the evaluator, use `match_pattern` to bind.

**Step 4: Write tests and commit**

---

## PHASE C: Type System & Protocols

---

## Task 16: Iterator Protocol

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/iterator_protocol.opl`

**Step 1: Update for-in loop**

When the iterable value is not a List, Range, or String, try calling `.iter()` on it, then repeatedly call `.next()` until `None` is returned.

**Step 2: Write spec test with custom iterator class**

**Step 3: Commit**

---

## Task 17: Visibility Enforcement

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/visibility.opl`

**Step 1: Parse `private`/`protected` before `def`**

**Step 2: Store visibility in StoredFunction**

**Step 3: Enforce in call_method**

**Step 4: Write tests and commit**

---

## Task 18: `as` Casting

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/01-basics/casting.opl`

**Step 1: Parse `expr as Type`**

Add `ExprKind::Cast { expr, type_name }`. Parse as a postfix after the expression.

**Step 2: Implement runtime conversions**

Int->Float, Float->Int (truncate), Any->String (format_value), String->Int/Float (parse, error on failure).

**Step 3: Write tests and commit**

---

## Task 19: Nullable `T?` in Type Annotations

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Parse `T?` as type annotation**

When parsing a type annotation after `:`, if the type name is followed by `?`, append `?` to the stored string.

**Step 2: Update type checking**

In `value_matches_type`, if type_name ends with `?`, strip it and also accept Null.

**Step 3: Write tests and commit**

---

## Task 20: Type-Narrowing in Match (`case x: Type`)

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Add `Pattern::Typed(String, String)`**

**Step 2: Parse `case x: Type`**

In pattern parser, when identifier is followed by `:`, parse as typed pattern.

**Step 3: Evaluate**

Check `value_is_type`, if true, bind variable.

**Step 4: Write tests and commit**

---

## Task 21: Named Enum Construction

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/enum_named_args.opl`

**Step 1: Update enum variant constructor in call_method**

When named args are provided, match by field name instead of position (same pattern as Class.new).

**Step 2: Write tests and commit**

---

## Task 22: Protocol Implementation on Enums

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/enum_protocol.opl`

**Step 1: When evaluating EnumDef with `implements`**

Look up protocols, copy default methods, check required methods.

**Step 2: Update value_is_type for enum variant protocol checking**

**Step 3: Write tests and commit**

---

## Task 23: Operator Overloading

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/operator_overload.opl`

**Step 1: On TypeError from eval_binary_op, try method dispatch**

Check if left value's class has a method named `+`, `-`, `*`, etc. Call it with right as argument.

**Step 2: Route indexing through `[]` method for non-builtins**

**Step 3: Write tests and commit**

---

## PHASE D: Advanced Features

---

## Task 24: `model` Keyword — Basic Structure

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/model_basic.opl`

**Step 1: Parse model definition**

`model Name needs field: Type where validator ... end`

Store as a class with extra metadata: validators, immutability flag.

**Step 2: Construction with validation**

On `.new()`, run type checks and validators. Reject invalid data.

**Step 3: Auto-generate `.to_dict()`, `.copy()`**

**Step 4: Write tests and commit**

---

## Task 25: `settings model`

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/settings_model.opl`

**Step 1: Parse `settings model` modifier**

**Step 2: Implement `.load()` — read from env vars**

Use `std::env::var()` to read env vars with prefix.

**Step 3: Write tests and commit**

---

## Task 26: Retroactive Conformance

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/04-classes/retroactive_conformance.opl`

**Step 1: Parse `implements Protocol for Type`**

New StmtKind variant. Contains protocol name, type name, and method definitions.

**Step 2: Evaluate — inject methods into existing class/enum**

**Step 3: Write tests and commit**

---

## Task 27: Exhaustiveness Checking

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: After match on enum/symbol set, check coverage**

If the matched value is an enum variant, collect all pattern enum names and compare against the enum's variant list. If any are missing and there's no wildcard, emit a runtime warning to stderr.

**Step 2: Write tests and commit**

---

## Task 28: Generic Enums (Syntax Only)

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`

**Step 1: Parse `enum Name[T]`**

Skip the generic parameter in brackets (same approach as function return types). Store but don't enforce.

**Step 2: Write tests and commit**

---

## Task 29: F-String Format Specifiers

**Files:**
- Modify: `crates/opal-parser/src/parser.rs` (f-string parsing)
- Modify: `crates/opal-interp/src/eval.rs` (format_value)
- Create: `tests/spec/01-basics/fstring_format.opl`

**Step 1: Parse `{expr:spec}` and `{expr=}`**

Extend the f-string parser to recognize `:` and `=` inside interpolation braces. Store format spec as part of `FStringPart::Expr`.

**Step 2: Implement format specifiers**

`:.N` — decimal places for floats. `:>N` — right-pad. `:<N` — left-pad. `:,` — thousands separator. `{x=}` — output `x=<value>`.

**Step 3: Write tests and commit**

---

## Task 30: Final Verification & Memory Update

**Step 1: Run full test suite**

```bash
cargo test && bash tests/run_spec.sh
```

**Step 2: Update MEMORY.md with new features and test counts**

**Step 3: Commit**
