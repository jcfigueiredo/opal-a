# Language Gaps Design: docs/01-03 Completeness

**Goal:** Implement all features documented in `docs/01-basics/`, `docs/02-control-flow/`, and `docs/03-functions-and-types/` that are feasible in the tree-walk interpreter.

**Baseline:** 138 unit tests, 83 spec tests, all passing.

---

## Phase A: Core Language Gaps

### A1. `break` / `next` in loops
Add `break` and `next` keywords to the lexer. Use `EvalError::Break` and `EvalError::Next` control flow (same pattern as `Return`). `for` and `while` handlers catch these. Suffix forms (`break if cond`, `next if cond`) parse as `if` wrapping a `break`/`next` statement.

### A2. Compound assignment (`+=`, `-=`, `*=`, `/=`)
Add tokens to lexer. Parse as `StmtKind::CompoundAssign { name, op, value }`. Evaluate: read current value, apply binary op, write back. Works on variables and instance fields (`.field += 1`).

### A3. Default parameter values
Already parsed into `Param.default`. In `call_function`, fill missing args from defaults (evaluated at call time).

### A4. Indexing (`[]` and `[]=`)
Parse `expr[expr]` as `ExprKind::Index { object, index }` in postfix position. Parse `expr[expr] = val` as `StmtKind::IndexAssign`. Implement for List (integer, negative indexing), Dict (string key), String (char access). Range slicing (`s[1..3]`) as stretch goal.

### A5. `in` / `not in`
Add `BinOp::In` and `BinOp::NotIn` (two-token like `is not`). Membership check for List, Dict (keys), Range, String (substring). Same precedence as comparisons.

### A6. Null-safe `?.` and `??`
`?.` short-circuits to null if receiver is null. Parse as variant of MemberAccess/Call. `??` null coalescing with low precedence (just above `or`).

### A7. Suffix `if`
After parsing an expression statement, check for trailing `if` keyword. Parse condition and wrap in conditional. Applies to any statement.

### A8. Parallel assignment
Parse `a, b = expr1, expr2` as `StmtKind::ParallelAssign`. Single RHS list: destructure. Multiple RHS: evaluate each and assign positionally.

### A9. `let` enforcement
Add `frozen: HashSet<String>` to `Environment`. `let` adds name to `frozen`. `assign()` checks and errors on immutable names.

---

## Phase B: Collections & Patterns

### B1. Collection methods
**List:** `.sort()`, `.sort(closure)`, `.reverse()`, `.find(closure)`, `.any?(closure)`, `.all?(closure)`, `.count()`, `.count(closure)`, `.each(closure)`, `.take(n)`, `.drop(n)`, `.flatten()`, `.zip(other)`, `.group_by(closure)`, `.join(sep)`.

**String:** `.to_int()`, `.to_float()`, `* n` repeat, `.reverse()`.

**Dict:** `.has_key?(key)`, `.merge(other)`.

### B2. Comprehensions
Parse `[expr for x in iterable]` and `[expr for x in iterable if cond]` as `ExprKind::ListComprehension`. Dict: `{k: v for x in iterable}`. Nested `for` clauses supported.

### B3. Match guards
Extend `MatchCase` with `guard: Option<Expr>`. Parse `case pattern if expr`. If pattern matches but guard is falsy, skip to next case.

### B4. Or-patterns
Parse `case 1 | 2 | 3` as `Pattern::Or(Vec<Pattern>)`. No variable binding in or-patterns. Match succeeds if any sub-pattern matches.

### B5. Dict patterns
Parse `case {key: var}` as `Pattern::Dict`. Match against `Value::Dict` with partial matching (extra keys ignored).

### B6. Range patterns
Parse `case 1..10` as `Pattern::Range`. Match `Value::Integer` within bounds.

### B7. As-bindings
Parse `case Pattern as name` as `Pattern::As(Box<Pattern>, String)`. Binds whole value while destructuring.

### B8. Destructuring in assignment, for-loops, closures, function params
Extend assignment parser for `[a, b | rest] = list`. For-loops: `for (a, b) in pairs`. Closures: `|(a, b)| expr`. Functions: `def f([head | tail])`. All reuse `match_pattern`.

---

## Phase C: Type System & Protocols

### C1. Iterator protocol
Built-in `Iterable` (requires `iter()`) and `Iterator` (requires `next()`) protocols. `for`-in calls `.iter()` then `.next()` until `None` for non-builtin types.

### C2. Visibility enforcement
Add `visibility` field to `StoredFunction`/`StoredClass`. Parse `private def`, `protected def`. Enforce: private = same class only, protected = class + subclasses.

### C3. `as` casting
Parse `expr as Type` as `ExprKind::Cast`. Runtime conversions: Int<->Float, String->Int/Float (parse), Any->String (format). Unknown casts raise TypeError.

### C4. Nullable `T?` syntax
In type annotations, `T?` = `T | Null`. Store as annotation string. Type checking accepts null for nullable types.

### C5. Type-narrowing in match
Parse `case x: Type` as `Pattern::Typed(name, type_name)`. Matches via `value_is_type` and binds.

### C6. Named enum construction
Match named args to variant field names in enum constructor, like `Class.new()` already does.

### C7. Protocol implementation on enums
Copy default methods, check required methods, update `value_is_type` for protocol checks on enum variants.

### C8. Operator overloading
On TypeError from `eval_binary_op`, fall back to class/enum method lookup for the operator name. Index operations route through `[]`/`[]=` methods for non-builtin types.

---

## Phase D: Advanced Features

### D1. `model` / `settings model`
Parse `model Name needs field: Type where validator ... end`. Immutable after construction, auto-validate, auto-generate `.to_dict()`, `.from_dict()`, `.copy()`. `settings model` adds `.load(env_prefix:, config:)` with source priority: defaults < config < env < explicit args.

### D2. Retroactive conformance
Parse `implements Protocol for Type`. Inject methods into existing class/enum's method table.

### D3. Exhaustiveness checking
For `match` on enums/symbol sets, check all variants covered. Runtime warning (not error) if incomplete and no wildcard.

### D4. Generic enums
Parse `enum Option[T]`. Type params erased at runtime — syntax works, no type-level checking.

### D5. F-string format specifiers
Extend f-string parser for `{expr:spec}` and `{expr=}`. Specs: `:.N` decimals, `:>N`/`:<N` padding, `:,` thousands separator. `{x=}` outputs `x=<value>`.
