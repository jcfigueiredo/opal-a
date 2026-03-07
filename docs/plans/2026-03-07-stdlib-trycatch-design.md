# Try/Catch Fix + Predicate Suffix + Stdlib Gaps Design

## Goal

Fix try/catch variable binding, add `?` predicate suffix to identifiers, and add missing List/String methods.

## Feature 1: Try/Catch Fix

### Current (broken)
```opal
catch e        # e is parsed as error TYPE, not variable → e is unbound
catch Error as e  # works — type=Error, var=e
catch          # swallows silently — bad pattern
```

### New syntax
```opal
catch e          # catch anything, bind to e (REQUIRED variable)
catch e as Error # catch only Error type, bind to e
# bare catch — NOT ALLOWED (removed)
```

### Changes
- **Parser:** After `catch`, always `expect_identifier()` for the variable. Then optionally `as` + identifier for type filter.
- **AST:** `CatchClause { var_name: String, error_type: Option<String>, body }` — var_name is no longer Optional.
- **Interpreter:** Already handles `var_name` and `error_type` correctly — just needs to check type filter.

## Feature 2: `?` Predicate Suffix

### Design
The lexer stays unchanged — `?` lexes as `Question` token. The parser handles combining `Identifier` + `Question` → `name?` in specific contexts:

1. **Method/function definitions:** `def any?(list)` — after `def`, parse name, peek for `?`, append
2. **Method calls:** `list.any?(|x| x > 0)` — after `.identifier`, peek for `?`, append
3. **Function calls:** `adult?(21)` — after `identifier(`, handled naturally if identifier includes `?`

Key insight: `Question` token is distinct from `QuestionDot` (`?.`) and `QuestionQuestion` (`??`). When we see `Identifier` then `Question`, we can safely consume it — the lexer already separated `?.` and `??` into their own tokens.

### Parser changes
- `expect_method_name()`: after parsing identifier, if next is `Question`, consume and append `?`
- `expect_identifier()`: same logic, but only in call/definition contexts
- Method call parsing: when parsing `expr.name(args)`, check for `?` after name

## Feature 3: List Missing Methods

| Method | Signature | Returns |
|--------|-----------|---------|
| `contains(item)` | `List.contains(value)` | `Bool` |
| `first()` | `List.first()` | first element or `null` |
| `last()` | `List.last()` | last element or `null` |
| `min()` | `List.min()` | minimum element |
| `max()` | `List.max()` | maximum element |
| `index(item)` | `List.index(value)` | index `Int` or `null` |
| `count(fn)` | `List.count(\|x\| pred)` | `Int` |
| `take(n)` | `List.take(3)` | `List` (first n) |
| `drop(n)` | `List.drop(3)` | `List` (skip n) |
| `empty?()` | `List.empty?()` | `Bool` |

All are additions to `call_method` in eval.rs — no parser changes.

## Feature 4: String Missing Methods

| Method | Signature | Returns |
|--------|-----------|---------|
| `upcase()` | `String.upcase()` | `String` |
| `downcase()` | `String.downcase()` | `String` |
| `slice(start, end)` | `String.slice(1, 4)` | `String` |
| `index(substr)` | `String.index("ll")` | `Int` or `null` |
| `empty?()` | `String.empty?()` | `Bool` |

## Not In Scope
- `?` in variable names (only method/function names)
- `group_by` (complex — defer)
- Regex methods (defer)
