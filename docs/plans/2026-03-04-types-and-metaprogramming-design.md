# Types & Metaprogramming Design

**Goal:** Add 6 high-impact features that bridge the type system and metaprogramming gaps in the Phase 1 interpreter.

**Build order:** typeof → is → type aliases → enums (with Ok/Error migration) → annotations → eval

---

## 1. `typeof(value)` — Type Objects

Returns a `Type` object (not a string) representing the runtime type.

**New Value variant:** `Value::Type(TypeInfo)`

```rust
enum TypeInfo {
    Builtin(BuiltinType),       // Int, Float, String, Bool, Null, Symbol, List, Dict, Range, Fn
    Class(ClassId),              // user-defined classes
    Protocol(ProtocolId),        // protocols
    Enum(EnumId),                // enum types
    EnumVariant(EnumId, usize),  // specific variant
}
```

**Behavior:**
- `typeof(42).name` → `"Int"`
- `typeof(circle).name` → `"Circle"` (class name)
- `typeof(42) == typeof(0)` → `true`
- `typeof(circle).fields` → `[(:radius, "Float")]` — list of (symbol, type_string) from `needs`
- `typeof(builtin).fields` → `[]`
- Display: prints type name directly (`f"type: {typeof(x)}"` works)

## 2. `is` / `is not` Operator

Runtime type check returning Bool. Works with classes, protocols, builtins.

**Syntax:** `value is TypeName`, `value is not TypeName`

**Parser:** New `BinOp::Is` and `BinOp::IsNot`. `is` keyword token.

**Resolution:**
- Builtins: `42 is Int` → checks BuiltinType match
- Classes: `circle is Circle` → checks ClassId match
- Protocols: `circle is Measurable` → uses `class_implements_protocol`
- Type aliases: `value is Status` → resolves alias, checks underlying type/symbol set

**No type narrowing in Phase 1** — returns Bool only.

## 3. Type Aliases

`type Name = Definition` creates transparent aliases.

**AST:** `StmtKind::TypeAlias { name: String, definition: TypeExpr }`

```rust
enum TypeExpr {
    Named(String),               // Int, String, Circle
    Union(Vec<TypeExpr>),        // A | B | C
    SymbolSet(Vec<String>),      // :ok | :error
    Nullable(Box<TypeExpr>),     // T?
}
```

**Storage:** `HashMap<String, TypeExpr>` in interpreter.

**`is` integration:** `value is Status` resolves `Status` → `SymbolSet(["ok", "error", "pending"])` → checks if value is one of those symbols.

**Phase 1 scope:** Named, Union, SymbolSet, Nullable. No generics, no function types.

## 4. Enums

Full algebraic data types with variants, methods, and protocol conformance.

### Definition
```opal
enum Direction
  North; South; East; West
end

enum Shape implements Printable
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)

  def area()
    match self
      case Shape.Circle(r)    -> 3.14 * r * r
      case Shape.Rectangle(w, h) -> w * h
    end
  end
end
```

### AST
```rust
StmtKind::EnumDef {
    name: String,
    variants: Vec<EnumVariantDef>,
    methods: Vec<Stmt>,
    implements: Vec<String>,
}

struct EnumVariantDef {
    name: String,
    fields: Vec<NeedsDecl>,
}
```

### Runtime
- `StoredEnum { name, variants, methods, implements }`
- `Value::EnumVariant(EnumId, variant_index, Vec<Value>)`
- Construction: `Direction.North` (singleton), `Shape.Circle(5.0)` (positional), `Shape.Circle(radius: 5.0)` (named)
- Methods dispatched like class methods; `self` = the variant instance

### Pattern Matching
```opal
match shape
  case Shape.Circle(r)
    3.14 * r * r
  case Shape.Rectangle(w, h)
    w * h
end
```

New: `Pattern::EnumVariant(enum_name, variant_name, Vec<Pattern>)`

### Ok/Error/Some/None Migration

**Remove:** `Value::Ok`, `Value::Error`, `Value::Some` from Value enum.

**Add built-in enums at interpreter init:**
```opal
enum Result
  Ok(value)
  Err(error)
end

enum Option
  Some(value)
  None
end
```

**Register** `Ok`, `Err`, `Some`, `None` as top-level constructors so existing code works unchanged.

**Semantic change:** `None` becomes `Option.None` (enum variant), NOT `Value::Null`. `nil` remains `Value::Null`. They are separate concepts per spec.

**All 73+ spec tests updated** as part of migration.

### Deferred
- Exhaustiveness checking (Phase 2)
- Generic enums: `Option[T]`, `Result[T, E]` (Phase 2)

## 5. Annotations

`@[key: val, ...]` metadata on declarations. Distinct from `@name` macro invocations.

### Syntax
```opal
@[deprecated, since: "2.0", use: "new_api"]
def old_api()
  "old"
end

class Config
  @[env: "DB_URL"]
  needs db_url: String
end
```

### Implementation
- **Lexer:** New `@[` token (distinct from `@` for macros)
- **Parser:** Parse annotation key-value pairs, attach to following statement
- **AST:** `annotations: Vec<Annotation>` field on FuncDef, ClassDef, EnumDef, NeedsDecl
- **Annotation** = `Vec<(String, Option<Value>)>` — key with optional value

### Storage
- `StoredFunction.annotations`, `StoredClass.annotations`, `StoredEnum.annotations`
- Field-level: `NeedsDecl.annotations`

### Querying
- `annotations(func_name)` → list of annotation dicts
- `ClassName.annotations()` → class annotations
- `ClassName.field_annotations(:field_name)` → field annotations

### `@[deprecated]` Warning
When calling a function with `@[deprecated]`:
- Print warning to stderr: `warning: old_api is deprecated (since 2.0, use new_api instead)`
- Extract `since:` and `use:` from annotation metadata

## 6. `eval(expr)` Builtin

Evaluates a `Value::Ast(AstId)` at runtime in a **child scope**.

Note: This is Opal's AST evaluation function for its macro/metaprogramming system,
not JavaScript's eval(). It only executes Opal AST nodes captured via `ast ... end` blocks,
not arbitrary strings. This is a standard feature in languages with homoiconic-style
metaprogramming (similar to Elixir's Code.eval_quoted or Julia's eval).

**Behavior:**
- Creates a child scope (can read parent, new bindings don't leak)
- Evaluates stored AST statements
- Returns value of last expression
- Errors on non-AST argument

```opal
code = ast
  x = 10
  x * 2
end
result = eval(code)  # => 20
# x is NOT defined here — child scope
```

**Distinction from macros:** Macros auto-eval in current scope (for injection). `eval()` is the safe, isolated version.

---

## Testing Strategy

Each feature gets:
1. Unit tests in the relevant crate (lexer/parser/interpreter)
2. Spec tests in `tests/spec/` with `# expect:` headers
3. Integration via example apps where applicable

## Files Modified

- `crates/opal-lexer/src/token.rs` — `is`, `type`, `@[` tokens
- `crates/opal-parser/src/ast.rs` — TypeExpr, EnumDef, EnumVariantDef, Annotation, TypeAlias, Pattern::EnumVariant
- `crates/opal-parser/src/parser.rs` — parse_type_alias, parse_enum_def, parse_annotation, is operator
- `crates/opal-runtime/src/value.rs` — Value::Type, Value::EnumVariant, TypeInfo, remove Ok/Error/Some
- `crates/opal-interp/src/eval.rs` — typeof builtin, is evaluation, type alias storage, enum storage + dispatch, annotation storage + query, eval builtin, migrate Ok/Error/Some/None
