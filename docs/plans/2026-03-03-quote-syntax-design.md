# Quote Syntax Redesign

## Goal

Replace the `quote` keyword with `ast` and add an inline form `ast(expr)` for single-expression quoting. The current `quote expr end` syntax is too verbose for the most common case — wrapping a single expression as an AST literal inside a macro.

## The Change

Rename `quote` to `ast` everywhere, and add parenthesized form for inline expressions.

**Two forms:**

```opal
# Inline — single expressions (new)
ast($name: $type)
ast(.$name = $name)
ast(def $name() -> $type = .$name)
ast(f($params...))

# Multi-line — blocks (renamed from quote...end)
ast
  def init($params...)
    $assignments...
  end
  $getters...
end
```

## Why `ast`

- **Not a common variable name** — unlike `code`, developers rarely name variables `ast`
- **Self-documenting** — you're creating an AST (Abstract Syntax Tree) node
- **Short** — 3 characters, less ceremony than `quote` (5) for the keyword itself
- **Inline form eliminates 8 characters** — `ast(expr)` vs `quote expr end`

## BNF Change

```bnf
# Before
<quote_expr>    ::= "quote" <expression> "end"
                   | "quote" NEWLINE <block> "end"

# After
<ast_expr>      ::= "ast" "(" <expression> ")"
                   | "ast" NEWLINE <block> "end"
```

## What Stays the Same

- `$expr` interpolation inside both forms — unchanged
- `$list...` splats inside both forms — unchanged
- `macroexpand()` function name — unchanged
- `macro name(params) ... end` definition syntax — unchanged
- `@name` invocation syntax — unchanged
- `esc(expr)` for breaking hygiene — unchanged
- `Expr` type name — unchanged

## Terminology Updates

| Before | After |
|---|---|
| "quoting code" | "capturing code as AST" |
| "inside a quote" | "inside an ast block" |
| "quoted expression" | "AST expression" or "AST literal" |
| `quote ... end` (BNF) | `ast ... end` (BNF) |

## Before / After Comparison

The `needs` macro shows the improvement most clearly:

```opal
# BEFORE
params = declarations.map do |decl|
  if decl.default
    quote $decl.name: $decl.type = $decl.default end
  else
    quote $decl.name: $decl.type end
  end
end

assignments = declarations.map do |decl|
  name = decl.name
  quote .$name = $name end
end

getters = declarations.map do |decl|
  name = decl.name
  type = decl.type
  quote def $name() -> $type = .$name end
end

quote
  def init($params...)
    $assignments...
  end
  $getters...
end


# AFTER
params = declarations.map do |decl|
  if decl.default
    ast($decl.name: $decl.type = $decl.default)
  else
    ast($decl.name: $decl.type)
  end
end

assignments = declarations.map do |decl|
  name = decl.name
  ast(.$name = $name)
end

getters = declarations.map do |decl|
  name = decl.name
  type = decl.type
  ast(def $name() -> $type = .$name)
end

ast
  def init($params...)
    $assignments...
  end
  $getters...
end
```

## Spec Impact

**BNF change in `Opal.md`:** Rename `<quote_expr>` rule, change `"quote"` to `"ast"`, add inline form.

**Files to update (search-and-replace `quote` to `ast` in code blocks):**

| File | Scope |
|---|---|
| `Opal.md` | BNF rule + hub metaprogramming section |
| `docs/07-metaprogramming/metaprogramming.md` | All code examples, prose references, comparison table |
| `docs/appendix/self-hosting.md` | All macro code examples |
| `CLAUDE.md` | If metaprogramming syntax is mentioned |

**Parser core list update:** The parser core diagram in `self-hosting.md` changes from `quote, macro, $` to `ast, macro, $`.

## Kept As-Is

- `$` interpolation syntax
- `$list...` splat syntax
- `macro...end` definition syntax
- `@name` invocation syntax
- `esc()` and `macroexpand()` functions
- `Expr` type and its API (`.head`, `.args`, `.dump()`)
