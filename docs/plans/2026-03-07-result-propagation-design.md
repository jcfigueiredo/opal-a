# Result Propagation & Helper Methods Design

## Goal

Implement the `!` propagation operator for Result types, add Result helper methods (`unwrap`, `unwrap_or`, `ok?`, `err?`), and unify `!`/`?` suffix handling by removing `!` from the identifier regex.

## Feature 1: Unify `!` and `?` Token Handling

### Current state
- `?` — separate `Question` token, parser combines `name` + `?` → `name?`
- `!` — baked into identifier regex, lexer produces `save!` as one token

### Change
Remove `!` from the identifier regex. Both `!` and `?` are handled identically:

```
Lexer:  save  !  (       →  three tokens
Parser: save! (           →  combines into call to save!()

Lexer:  result  !  \n    →  two tokens + newline
Parser: result!           →  propagation operator on result
```

### Rule
- `identifier` + `!` + `(` → parser combines into `name!(args)` call
- `identifier` + `?` + `(` → parser combines into `name?(args)` call
- `expr` + `!` (no `(` after) → propagation operator
- Works in method definitions too: `def save!()`, `def empty?()`

### Disambiguation
| Code | Tokens | Meaning |
|------|--------|---------|
| `save!(42)` | `save` `!` `(` `42` `)` | Call method `save!` with arg 42 |
| `result!` | `result` `!` | Propagation on `result` |
| `foo()!` | `foo` `(` `)` `!` | Propagation on foo()'s result |
| `foo()!.bar()` | `foo` `(` `)` `!` `.` `bar` `(` `)` | Propagate, then call bar on unwrapped value |
| `obj.save!()` | `obj` `.` `save` `!` `(` `)` | Call save!() method on obj |
| `obj.method!` | `obj` `.` `method` `!` | Propagation on obj.method's result |

## Feature 2: `!` Propagation Operator

### Syntax
```opal
def process(path)
  content = read_file(path)!     # unwrap Ok or return Err
  config = parse(content)!       # chain propagation
  Result.Ok(config)
end
```

### Semantics
- If value is `Ok(v)` → evaluate to `v` (unwrapped)
- If value is `Error(e)` → immediately return `Error(e)` from enclosing function
- Uses existing `EvalError::Return` mechanism for early return

### Precedence
`!` binds very tightly — same level as `.` and `()`:
- `result! + 1` → `(result!) + 1`
- `foo()!.bar()` → `(foo()!).bar()`
- `a + b!` → `a + (b!)`

### AST
Add `ExprKind::Propagate(Box<Expr>)` — postfix `!` on any expression.

### Parser
In the postfix expression loop (where `.`, `()`, `[]` are handled), add `!`:
- After parsing any expression, if next token is `Bang` AND next after that is NOT `(`, consume `Bang` and wrap in `Propagate`.
- If next is `Bang` followed by `(`, it's a call to `name!()` — combine identifier.

### Interpreter
```rust
ExprKind::Propagate(expr) => {
    let val = self.eval_expr(expr)?;
    match &val {
        Value::EnumVariant(name, _) if name == "Ok" => {
            // Extract the inner value from Ok(value)
            // Return the unwrapped value
        }
        Value::EnumVariant(name, _) if name == "Error" || name == "Err" => {
            // Return Err from the enclosing function
            return Err(EvalError::Return(val));
        }
        _ => {
            // Not a Result — runtime error
            return Err(EvalError::TypeError("! requires Ok or Error value".into()));
        }
    }
}
```

## Feature 3: Result Helper Methods

Add to `call_method` for `Value::EnumVariant`:

| Method | Implementation |
|--------|---------------|
| `ok?()` | `true` if variant name is `"Ok"` |
| `err?()` | `true` if variant name is `"Error"` |
| `unwrap()` | If `Ok`, return inner value. If `Error`, raise exception. |
| `unwrap_or(default)` | If `Ok`, return inner value. If `Error`, return `default`. |

### Examples
```opal
result = Ok(42)
result.ok?()           # => true
result.err?()          # => false
result.unwrap()        # => 42
result.unwrap_or(0)    # => 42

error = Error("fail")
error.ok?()            # => false
error.err?()           # => true
error.unwrap()         # raises "fail" as exception
error.unwrap_or(0)     # => 0
```

## Changes Per Layer

| Layer | Change |
|-------|--------|
| **Lexer** | Remove `!` from identifier regex (`!?` → nothing at end) |
| **Parser** | Combine `identifier` + `!` + `(` → `name!()` call (same as `?`). Add `Propagate` postfix expression. |
| **AST** | Add `ExprKind::Propagate(Box<Expr>)` |
| **Interpreter** | Evaluate `Propagate`: unwrap Ok or return Error. Add `ok?`, `err?`, `unwrap`, `unwrap_or` methods. |
| **Tree-sitter** | Add `propagation` rule: `seq(expression, '!')`. Update identifier regex. |
| **TextMate** | Update identifier regex to not include `!`. |

## Not In Scope
- `map()`, `map_err()` — defer
- `Result.from do ... end` — defer
- Compile-time check that `!` is in a Result-returning function — we're an interpreter
- `fail` keyword (spec uses it but current code uses `raise` — keep `raise` for now)
