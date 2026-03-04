# Types & Metaprogramming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add 6 high-impact features bridging type system and metaprogramming: typeof, is, type aliases, enums (with Ok/Error migration), annotations, and the Opal AST evaluator.

**Architecture:** Each feature builds on the last — typeof provides runtime type info, `is` uses it for checking, type aliases extend the type namespace, enums create proper algebraic types (migrating Ok/Error/Some), annotations add metadata to declarations, and an AST evaluator completes the metaprogramming loop. All work in the existing tree-walk interpreter without changing the execution model. Note: the AST evaluator is Opal's metaprogramming primitive (like Elixir's Code.eval_quoted), not JavaScript's eval — it only evaluates Opal AST nodes captured via `ast ... end` blocks.

**Tech Stack:** Rust workspace (opal-lexer, opal-parser, opal-runtime, opal-interp, opal-stdlib). Logos lexer, recursive descent parser, tree-walk interpreter.

---

## Reference: Current File Layout

- **Lexer tokens:** `crates/opal-lexer/src/token.rs` — `Token::Is` (line 292), `Token::Type` (line 242), `Token::Enum` (line 244), `Token::AtBracket` (line 416) already exist
- **AST:** `crates/opal-parser/src/ast.rs` — `StmtKind`, `ExprKind`, `Pattern`, `BinOp`
- **Parser:** `crates/opal-parser/src/parser.rs` — `parse_statement()` (line 51), `parse_expression()` (line 1148), `peek_binary_op()` (line 1987), `op_precedence()` (line 2017)
- **Values:** `crates/opal-runtime/src/value.rs` — `Value` enum, opaque IDs
- **Exports:** `crates/opal-runtime/src/lib.rs` — re-exports from value.rs
- **Interpreter:** `crates/opal-interp/src/eval.rs` — `Interpreter` struct (line 110), `eval_call` (line 1299), `match_pattern` (line 1196), `values_equal` (line 2433), `call_method` (line 1407)
- **Stdlib:** `crates/opal-stdlib/src/lib.rs` — `call_builtin()` (line 15), only `print`/`println`
- **Spec tests:** `tests/spec/` with `# expect:` headers, run via `bash tests/run_spec.sh`
- **Unit tests:** Bottom of each `.rs` file in `#[cfg(test)] mod tests`

## Reference: Running Tests

```bash
# All unit tests (120 tests across all crates)
cargo test

# Spec tests (73 tests, .opl files with # expect: headers)
bash tests/run_spec.sh

# Single unit test
cargo test -p opal-interp typeof_builtin

# Single spec test
cargo run -- tests/spec/02-functions/typeof_basic.opl
```

---

## Task 1: Add `Value::Type` and `TypeInfo` to Runtime

**Files:**
- Modify: `crates/opal-runtime/src/value.rs`
- Modify: `crates/opal-runtime/src/lib.rs`

**Step 1: Add EnumId to value.rs**

Add after `ProtocolId` (line 103):

```rust
/// Opaque ID for an enum definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumId(pub usize);
```

**Step 2: Add TypeInfo and BuiltinType enums to value.rs**

Add after the `EnumId` struct:

```rust
/// Built-in type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    Int,
    Float,
    String,
    Bool,
    Null,
    Symbol,
    List,
    Dict,
    Range,
    Fn,
}

/// Runtime type information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    Builtin(BuiltinType),
    Class(ClassId),
    Protocol(ProtocolId),
    Enum(EnumId),
    EnumVariant(EnumId, usize),
}
```

**Step 3: Add `Value::Type` and `Value::EnumVariant` variants**

Add to the `Value` enum (after `Protocol`):

```rust
/// Type object (returned by typeof)
Type(TypeInfo),
/// Enum variant value
EnumVariant {
    enum_id: EnumId,
    variant_index: usize,
    fields: Vec<Value>,
},
```

**Step 4: Add Display for new variants**

In the `Display` impl, add after the `Protocol` match arm (line 145):

```rust
Value::Type(info) => {
    match info {
        TypeInfo::Builtin(b) => write!(f, "{}", match b {
            BuiltinType::Int => "Int",
            BuiltinType::Float => "Float",
            BuiltinType::String => "String",
            BuiltinType::Bool => "Bool",
            BuiltinType::Null => "Null",
            BuiltinType::Symbol => "Symbol",
            BuiltinType::List => "List",
            BuiltinType::Dict => "Dict",
            BuiltinType::Range => "Range",
            BuiltinType::Fn => "Fn",
        }),
        TypeInfo::Class(id) => write!(f, "<type class #{}>", id.0),
        TypeInfo::Protocol(id) => write!(f, "<type protocol #{}>", id.0),
        TypeInfo::Enum(id) => write!(f, "<type enum #{}>", id.0),
        TypeInfo::EnumVariant(id, v) => write!(f, "<type enum #{} variant {}>", id.0, v),
    }
}
Value::EnumVariant { enum_id, variant_index, fields } => {
    write!(f, "<enum #{}.{}", enum_id.0, variant_index)?;
    if !fields.is_empty() {
        write!(f, "(")?;
        for (i, v) in fields.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", v)?;
        }
        write!(f, ")")?;
    }
    write!(f, ">")
}
```

**Step 5: Update lib.rs exports**

In `crates/opal-runtime/src/lib.rs`, update the `pub use value::` line to include `BuiltinType`, `EnumId`, `TypeInfo`.

**Step 6: Run tests to verify no breakage**

Run: `cargo test`
Expected: All 120 tests pass (no behavioral changes, just new types)

**Step 7: Commit**

```bash
git add crates/opal-runtime/
git commit -m "feat: add Value::Type, TypeInfo, EnumId, Value::EnumVariant to runtime"
```

---

## Task 2: Implement `typeof()` Builtin

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/02-functions/typeof_basic.opl`

**Step 1: Write the failing spec test**

Create `tests/spec/02-functions/typeof_basic.opl`:

```opal
# expect: Int | Float | String | Bool | Null | Symbol | List | Dict | Fn
print(f"{typeof(42).name} | {typeof(3.14).name} | {typeof("hi").name} | {typeof(true).name} | {typeof(null).name} | {typeof(:ok).name} | {typeof([1]).name} | {typeof({:}).name} | {typeof(|x| x).name}")
```

**Step 2: Run test to verify it fails**

Run: `cargo run -- tests/spec/02-functions/typeof_basic.opl`
Expected: FAIL — `UndefinedVariable("typeof")`

**Step 3: Add StoredEnum and enums field to interpreter**

In `eval.rs`, add the struct definitions:

```rust
#[derive(Clone)]
struct StoredEnum {
    name: String,
    variants: Vec<StoredEnumVariant>,
    methods: Vec<StoredFunction>,
}

#[derive(Clone)]
struct StoredEnumVariant {
    name: String,
    fields: Vec<(String, Option<String>)>,
}
```

Add `enums: Vec<StoredEnum>` field to `Interpreter` struct (after `protocols`).
Initialize as `Vec::new()` in all constructors.

**Step 4: Add typeof handling in eval_call**

In the `eval_call` method, add a new match arm in the builtin constructors block (around line 1353, before `_ => {}`):

```rust
"typeof" if arg_values.len() == 1 => {
    let type_info = self.value_type_info(&arg_values[0]);
    return Ok(Value::Type(type_info));
}
```

**Step 5: Add the `value_type_info` helper method**

```rust
fn value_type_info(&self, value: &Value) -> TypeInfo {
    match value {
        Value::Integer(_) => TypeInfo::Builtin(BuiltinType::Int),
        Value::Float(_) => TypeInfo::Builtin(BuiltinType::Float),
        Value::String(_) => TypeInfo::Builtin(BuiltinType::String),
        Value::Bool(_) => TypeInfo::Builtin(BuiltinType::Bool),
        Value::Null => TypeInfo::Builtin(BuiltinType::Null),
        Value::Symbol(_) => TypeInfo::Builtin(BuiltinType::Symbol),
        Value::List(_) => TypeInfo::Builtin(BuiltinType::List),
        Value::Dict(_) => TypeInfo::Builtin(BuiltinType::Dict),
        Value::Range { .. } => TypeInfo::Builtin(BuiltinType::Range),
        Value::Function(_) | Value::MultiFunction(_) | Value::Closure(_) | Value::NativeFunction(_) => {
            TypeInfo::Builtin(BuiltinType::Fn)
        }
        Value::Instance(id) => {
            let inst = &self.instances[id.0];
            TypeInfo::Class(inst.class_id)
        }
        Value::EnumVariant { enum_id, variant_index, .. } => {
            TypeInfo::EnumVariant(*enum_id, *variant_index)
        }
        _ => TypeInfo::Builtin(BuiltinType::Fn), // Actor, Ast, Class, Module, etc.
    }
}
```

**Step 6: Add `.name` and `.fields` methods for Type values**

In `call_method`, add a match arm for `Value::Type`:

```rust
Value::Type(ref info) => {
    match method.as_str() {
        "name" => {
            let name = self.type_info_name(info);
            Ok(Value::String(name))
        }
        "fields" => {
            match info {
                TypeInfo::Class(id) => {
                    let class = &self.classes[id.0];
                    let field_list: Vec<Value> = class.needs.iter().map(|(name, type_ann)| {
                        Value::List(vec![
                            Value::Symbol(name.clone()),
                            Value::String(type_ann.clone().unwrap_or_else(|| "Any".to_string())),
                        ])
                    }).collect();
                    Ok(Value::List(field_list))
                }
                _ => Ok(Value::List(vec![])),
            }
        }
        _ => Err(EvalError::RuntimeError(format!("Type has no method '{}'", method))),
    }
}
```

Add helper:

```rust
fn type_info_name(&self, info: &TypeInfo) -> String {
    match info {
        TypeInfo::Builtin(b) => match b {
            BuiltinType::Int => "Int",
            BuiltinType::Float => "Float",
            BuiltinType::String => "String",
            BuiltinType::Bool => "Bool",
            BuiltinType::Null => "Null",
            BuiltinType::Symbol => "Symbol",
            BuiltinType::List => "List",
            BuiltinType::Dict => "Dict",
            BuiltinType::Range => "Range",
            BuiltinType::Fn => "Fn",
        }.to_string(),
        TypeInfo::Class(id) => self.classes[id.0].name.clone(),
        TypeInfo::Protocol(id) => self.protocols[id.0].name.clone(),
        TypeInfo::Enum(id) => self.enums[id.0].name.clone(),
        TypeInfo::EnumVariant(id, vi) => {
            format!("{}.{}", self.enums[id.0].name, self.enums[id.0].variants[*vi].name)
        }
    }
}
```

**Step 7: Add `format_value` method and use for f-string output**

Add a method that properly formats values using interpreter state:

```rust
fn format_value(&self, value: &Value) -> String {
    match value {
        Value::Type(info) => self.type_info_name(info),
        Value::EnumVariant { enum_id, variant_index, fields } => {
            let e = &self.enums[enum_id.0];
            let v = &e.variants[*variant_index];
            if fields.is_empty() {
                format!("{}.{}", e.name, v.name)
            } else {
                let args: Vec<String> = fields.iter().map(|f| self.format_value(f)).collect();
                format!("{}.{}({})", e.name, v.name, args.join(", "))
            }
        }
        Value::Instance(id) => {
            let inst = &self.instances[id.0];
            let class = &self.classes[inst.class_id.0];
            format!("<{} instance>", class.name)
        }
        other => other.to_string(),
    }
}
```

Use `format_value` in the f-string interpolation evaluation path and in the print output path.

**Step 8: Add values_equal for Type and EnumVariant**

In `values_equal`, add:

```rust
(Value::Type(a), Value::Type(b)) => a == b,
(Value::EnumVariant { enum_id: a_id, variant_index: a_vi, fields: a_f },
 Value::EnumVariant { enum_id: b_id, variant_index: b_vi, fields: b_f }) => {
    a_id == b_id && a_vi == b_vi && a_f.len() == b_f.len() &&
    a_f.iter().zip(b_f.iter()).all(|(a, b)| values_equal(a, b))
}
```

**Step 9: Update imports**

Add `EnumId`, `BuiltinType`, `TypeInfo` to the `use opal_runtime::` import in eval.rs.

**Step 10: Write unit tests**

```rust
#[test]
fn typeof_builtin() {
    assert_eq!(run(r#"print(typeof(42).name)"#).unwrap(), "Int");
    assert_eq!(run(r#"print(typeof("hi").name)"#).unwrap(), "String");
    assert_eq!(run(r#"print(typeof(true).name)"#).unwrap(), "Bool");
    assert_eq!(run(r#"print(typeof(null).name)"#).unwrap(), "Null");
    assert_eq!(run(r#"print(typeof(:ok).name)"#).unwrap(), "Symbol");
}

#[test]
fn typeof_equality() {
    assert_eq!(run(r#"print(typeof(1) == typeof(2))"#).unwrap(), "true");
    assert_eq!(run(r#"print(typeof(1) == typeof("hi"))"#).unwrap(), "false");
}

#[test]
fn typeof_class() {
    assert_eq!(
        run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 1)\nprint(typeof(f).name)").unwrap(),
        "Foo"
    );
}

#[test]
fn typeof_fields() {
    assert_eq!(
        run("class Foo\n  needs x: Int\n  needs y: String\nend\nf = Foo.new(x: 1, y: \"a\")\nprint(typeof(f).fields)").unwrap(),
        "[[:x, Int], [:y, String]]"
    );
}
```

**Step 11: Run all tests**

Run: `cargo test && bash tests/run_spec.sh`
Expected: All pass including new tests

**Step 12: Commit**

```bash
git add crates/ tests/
git commit -m "feat: implement typeof() builtin returning Type objects with .name and .fields"
```

---

## Task 3: Implement `is` / `is not` Operator

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `BinOp::Is`, `BinOp::IsNot`
- Modify: `crates/opal-parser/src/parser.rs` — parse `is` and `is not`
- Modify: `crates/opal-interp/src/eval.rs` — evaluate `is` checks
- Create: `tests/spec/02-functions/is_operator.opl`

**Step 1: Write the failing spec test**

Create `tests/spec/02-functions/is_operator.opl`:

```opal
# expect: true | false | true | true | false
class Dog
  needs name: String
end
protocol Greetable
  def greet() -> String
end
class Cat implements Greetable
  needs name: String
  def greet()
    f"meow from {.name}"
  end
end
d = Dog.new(name: "Rex")
c = Cat.new(name: "Whiskers")
r1 = d is Dog
r2 = d is Cat
r3 = c is Greetable
r4 = 42 is Int
r5 = 42 is String
print(f"{r1} | {r2} | {r3} | {r4} | {r5}")
```

**Step 2: Run test to verify it fails**

Run: `cargo run -- tests/spec/02-functions/is_operator.opl`
Expected: FAIL — parse error (no `is` binary op yet)

**Step 3: Add BinOp::Is and BinOp::IsNot to AST**

In `crates/opal-parser/src/ast.rs`, add to the `BinOp` enum (after `Pipe`):

```rust
Is,
IsNot,
```

**Step 4: Add `is` to parser's `peek_binary_op`**

In `peek_binary_op()` (line 1987), add before `_ => None`:

```rust
Some(Token::Is) => Some(BinOp::Is),
```

**Step 5: Add precedence for Is/IsNot**

In `op_precedence()` (line 2017), add:

```rust
BinOp::Is | BinOp::IsNot => (4, Assoc::Left),
```

**Step 6: Handle `is not` in parse_expression**

In `parse_expression()`, after consuming the operator token on line 1156 (`self.advance()`), add:

```rust
// Handle `is not` as two-token operator
let op = if op == BinOp::Is && self.check(&Token::Not) {
    self.advance(); // consume `not`
    BinOp::IsNot
} else {
    op
};
```

**Step 7: Handle BinOp::Is specially in eval_expr**

In `eval_expr`, in the `ExprKind::BinaryOp` handler (around line 822), add special handling **before** the general binary op evaluation. The `is` operator's RHS is a type name, not an expression to evaluate:

```rust
ExprKind::BinaryOp { left, op, right } => {
    // Special handling for `is` / `is not` — RHS is a type name, not evaluated
    if *op == BinOp::Is || *op == BinOp::IsNot {
        let left_val = self.eval_expr(left)?;
        let type_name = match &right.kind {
            ExprKind::Identifier(name) => name.clone(),
            _ => return Err(EvalError::TypeError("is operator requires a type name".into())),
        };
        let result = self.value_is_type(&left_val, &type_name);
        return Ok(Value::Bool(if *op == BinOp::Is { result } else { !result }));
    }
    // ... rest of existing Pipe handling and binary op evaluation
```

**Step 8: Implement `value_is_type` helper**

```rust
fn value_is_type(&self, value: &Value, type_name: &str) -> bool {
    match type_name {
        "Int" => matches!(value, Value::Integer(_)),
        "Float" => matches!(value, Value::Float(_)),
        "String" => matches!(value, Value::String(_)),
        "Bool" => matches!(value, Value::Bool(_)),
        "Null" => matches!(value, Value::Null),
        "Symbol" => matches!(value, Value::Symbol(_)),
        "List" => matches!(value, Value::List(_)),
        "Dict" => matches!(value, Value::Dict(_)),
        "Range" => matches!(value, Value::Range { .. }),
        "Fn" => matches!(value, Value::Function(_) | Value::MultiFunction(_) | Value::Closure(_)),
        name => {
            if let Value::Instance(id) = value {
                let inst = &self.instances[id.0];
                if self.classes[inst.class_id.0].name == name {
                    return true;
                }
                return self.class_implements_protocol_by_name(inst.class_id, name);
            }
            if let Value::EnumVariant { enum_id, .. } = value {
                if self.enums[enum_id.0].name == name {
                    return true;
                }
            }
            false
        }
    }
}

fn class_implements_protocol_by_name(&self, class_id: ClassId, protocol_name: &str) -> bool {
    for (i, proto) in self.protocols.iter().enumerate() {
        if proto.name == protocol_name {
            return self.class_implements_protocol(class_id, ProtocolId(i));
        }
    }
    false
}
```

**Step 9: Write unit tests**

```rust
#[test]
fn is_operator_builtins() {
    assert_eq!(run("print(42 is Int)").unwrap(), "true");
    assert_eq!(run("print(42 is String)").unwrap(), "false");
    assert_eq!(run(r#"print("hi" is String)"#).unwrap(), "true");
    assert_eq!(run("print(true is Bool)").unwrap(), "true");
    assert_eq!(run("print(null is Null)").unwrap(), "true");
}

#[test]
fn is_not_operator() {
    assert_eq!(run("print(42 is not String)").unwrap(), "true");
    assert_eq!(run("print(42 is not Int)").unwrap(), "false");
}

#[test]
fn is_operator_class() {
    assert_eq!(
        run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 1)\nprint(f is Foo)").unwrap(),
        "true"
    );
}
```

**Step 10: Run all tests**

Run: `cargo test && bash tests/run_spec.sh`
Expected: All pass

**Step 11: Commit**

```bash
git add crates/ tests/
git commit -m "feat: implement is/is not operator for runtime type checking"
```

---

## Task 4: Implement Type Aliases

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `StmtKind::TypeAlias`, `TypeExpr`
- Modify: `crates/opal-parser/src/parser.rs` — parse `type Name = Def`
- Modify: `crates/opal-interp/src/eval.rs` — store and resolve type aliases
- Create: `tests/spec/02-functions/type_alias.opl`

**Step 1: Write the failing spec test**

Create `tests/spec/02-functions/type_alias.opl`:

```opal
# expect: true | true | false | true
type Status = :ok | :error | :pending
type ID = Int

r1 = :ok is Status
r2 = :error is Status
r3 = :unknown is Status
r4 = 42 is ID
print(f"{r1} | {r2} | {r3} | {r4}")
```

**Step 2: Add TypeExpr and StmtKind::TypeAlias to AST**

In `crates/opal-parser/src/ast.rs`:

```rust
/// A type expression used in type aliases
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String),
    Union(Vec<TypeExpr>),
    SymbolSet(Vec<String>),
    Nullable(Box<TypeExpr>),
}
```

Add to `StmtKind`:

```rust
TypeAlias { name: String, definition: TypeExpr },
```

**Step 3: Parse type alias in parser**

In `parse_statement()`, add:

```rust
if self.check(&Token::Type) {
    return self.parse_type_alias(start);
}
```

Add parse methods for `parse_type_alias`, `parse_type_expr`, and `parse_symbol_set_or_union`. The type expression parser needs to handle:
- Simple names: `Int`, `String`
- Symbol sets: `:ok | :error | :pending`
- Unions: `Int | String`

**Step 4: Store type aliases in interpreter**

Add `type_aliases: HashMap<String, TypeExpr>` field. Handle `StmtKind::TypeAlias` in `eval_stmt`.

**Step 5: Integrate with `is` operator**

In `value_is_type`, at the end of the `name =>` branch, add type alias lookup:

```rust
if let Some(type_expr) = self.type_aliases.get(name).cloned() {
    return self.value_matches_type_expr(value, &type_expr);
}
```

Implement `value_matches_type_expr` to recursively match Named, Union, SymbolSet, Nullable.

**Step 6: Write unit tests**

```rust
#[test]
fn type_alias_basic() {
    assert_eq!(run("type ID = Int\nprint(42 is ID)").unwrap(), "true");
}

#[test]
fn type_alias_symbol_set() {
    assert_eq!(run("type Status = :ok | :error\nprint(:ok is Status)").unwrap(), "true");
    assert_eq!(run("type Status = :ok | :error\nprint(:unknown is Status)").unwrap(), "false");
}

#[test]
fn type_alias_union() {
    assert_eq!(run("type NumOrStr = Int | String\nprint(42 is NumOrStr)").unwrap(), "true");
}
```

**Step 7: Run all tests**

Run: `cargo test && bash tests/run_spec.sh`

**Step 8: Commit**

```bash
git add crates/ tests/
git commit -m "feat: implement type aliases with symbol sets, unions, and is integration"
```

---

## Task 5: Implement Enums — Definition, Construction, Pattern Matching

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `StmtKind::EnumDef`, `EnumVariantDef`, `Pattern::EnumVariant`
- Modify: `crates/opal-parser/src/parser.rs` — parse `enum ... end`
- Modify: `crates/opal-interp/src/eval.rs` — store, construct, match, display
- Create: `tests/spec/04-classes/enum_basic.opl`
- Create: `tests/spec/04-classes/enum_match.opl`
- Create: `tests/spec/04-classes/enum_methods.opl`

**Step 1: Write failing spec tests**

`tests/spec/04-classes/enum_basic.opl`:
```opal
# expect: Direction.North | Shape.Circle(5.0) | true
enum Direction
  North
  South
end
enum Shape
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
end
d = Direction.North
s = Shape.Circle(5.0)
print(f"{d} | {s} | {d is Direction}")
```

`tests/spec/04-classes/enum_match.opl`:
```opal
# expect: area: 78.5 | area: 30.0
enum Shape
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
end
def area(s)
  match s
    case Shape.Circle(r)
      3.14 * r * r
    case Shape.Rectangle(w, h)
      w * h
  end
end
print(f"area: {area(Shape.Circle(5.0))} | area: {area(Shape.Rectangle(10.0, 3.0))}")
```

`tests/spec/04-classes/enum_methods.opl`:
```opal
# expect: Circle: area=78.5
enum Shape
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
  def describe()
    match self
      case Shape.Circle(r)
        f"Circle: area={3.14 * r * r}"
      case Shape.Rectangle(w, h)
        f"Rectangle: area={w * h}"
    end
  end
end
print(Shape.Circle(5.0).describe())
```

**Step 2: Add AST nodes**

In `ast.rs`:
```rust
pub struct EnumVariantDef {
    pub name: String,
    pub fields: Vec<NeedsDecl>,
}
```

Add to `StmtKind`:
```rust
EnumDef {
    name: String,
    variants: Vec<EnumVariantDef>,
    methods: Vec<Stmt>,
    implements: Vec<String>,
},
```

Add to `Pattern`:
```rust
EnumVariant(String, String, Vec<Pattern>),
```

**Step 3: Parse enum definitions**

In `parse_statement()`, add enum check. Implement `parse_enum_def` that parses variants (with optional field lists) and methods (def blocks).

**Step 4: Parse enum variant patterns**

In the pattern parser, when an identifier is followed by `.`, parse as `Pattern::EnumVariant(enum_name, variant_name, sub_patterns)`.

**Step 5: Evaluate EnumDef**

Store in `self.enums`. Register enum name as `Value::Type(TypeInfo::Enum(enum_id))` in environment.

**Step 6: Handle enum variant construction**

In `eval_call` for `MemberAccess`, when object is `Value::Type(TypeInfo::Enum(id))`, construct `Value::EnumVariant`. In `eval_expr` for `MemberAccess` (non-call), return singleton variants directly.

**Step 7: Pattern matching**

In `match_pattern`, handle `Pattern::EnumVariant` — check enum name, variant name, and destructure fields.

**Step 8: Method calls on enum variants**

In `call_method`, handle `Value::EnumVariant` — look up method in `StoredEnum`, bind `self` to the variant value, execute.

**Step 9: Display with format_value**

Update `format_value` to render enum variants as `Name.Variant` (singleton) or `Name.Variant(args)` (data-carrying).

**Step 10: Run all tests and commit**

```bash
git add crates/ tests/
git commit -m "feat: implement enums with variants, pattern matching, methods, and is support"
```

---

## Task 6: Migrate Ok/Error/Some/None to Built-in Enums

**Files:**
- Modify: `crates/opal-runtime/src/value.rs` — remove `Ok`, `Error`, `Some` variants
- Modify: `crates/opal-interp/src/eval.rs` — register Result/Option enums, update all usage
- Update: existing spec tests that use Ok/Error/Some/None

**Step 1: Register built-in enums at init**

Add `register_builtin_enums()` called from constructors. Creates Result (Ok, Err) and Option (Some, None) enums at index 0 and 1.

**Step 2: Update constructors in eval_call**

Change `Ok()`, `Error()` (alias for `Err()`), `Some()` to create `Value::EnumVariant` instead of `Value::Ok`/etc.

**Step 3: Change `None` identifier**

`None` now creates `Value::EnumVariant { enum_id: EnumId(1), variant_index: 1, fields: vec![] }` instead of `Value::Null`.

**Step 4: Update pattern matching**

`Pattern::Constructor("Ok", ...)` matches `Value::EnumVariant` with Result enum_id and variant 0. Same for Error/Err, Some, None.

**Step 5: Remove old Value variants**

Remove `Value::Ok(Box<Value>)`, `Value::Error(Box<Value>)`, `Value::Some(Box<Value>)` from value.rs. Fix all compile errors.

**Step 6: Update Display for Result/Option**

In `format_value`, show `Ok(42)` not `Result.Ok(42)` for readability.

**Step 7: Fix all broken references**

Search for `Value::Ok`, `Value::Error`, `Value::Some` across eval.rs and update each to use `Value::EnumVariant`. Key areas:
- Pipe operator Error propagation
- Try/catch Error matching
- RequiresFailed handling
- values_equal

**Step 8: Run all tests iteratively**

This step will likely require multiple fix-test cycles. Run `cargo test && bash tests/run_spec.sh` after each fix.

**Step 9: Commit**

```bash
git add crates/ tests/
git commit -m "feat: migrate Ok/Error/Some/None to built-in Result/Option enums"
```

---

## Task 7: Implement Annotations

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `StmtKind::Annotated`, `Annotation`, `AnnotationEntry`
- Modify: `crates/opal-parser/src/parser.rs` — parse `@[key: val]`
- Modify: `crates/opal-interp/src/eval.rs` — store and query annotations
- Create: `tests/spec/07-macros/annotations.opl`

**Step 1: Write failing spec test**

`tests/spec/07-macros/annotations.opl`:
```opal
# expect: [{deprecated: true, since: 2.0}]
@[deprecated, since: "2.0"]
def old_api()
  "old"
end
print(annotations(old_api))
```

**Step 2: Add AST types**

```rust
pub struct AnnotationEntry {
    pub key: String,
    pub value: Option<Expr>,
}

pub struct Annotation {
    pub entries: Vec<AnnotationEntry>,
}
```

Add to `StmtKind`:
```rust
Annotated {
    annotations: Vec<Annotation>,
    statement: Box<Stmt>,
},
```

**Step 3: Parse annotations**

In `parse_statement()`, when `@[` is seen, collect annotations then parse the following statement. Wrap in `StmtKind::Annotated`.

**Step 4: Store annotations**

Add `annotations` field to `StoredFunction` and `StoredClass`. Evaluate annotation values at definition time. Store as `Vec<Vec<(String, Value)>>`.

**Step 5: Implement `annotations()` builtin**

In `eval_call`, handle `"annotations"` — look up function by value, return its stored annotations as a List of Dicts.

**Step 6: Implement @[deprecated] warning**

When calling a function with `deprecated` annotation, print warning to stderr.

**Step 7: Write unit tests and run**

**Step 8: Commit**

```bash
git add crates/ tests/
git commit -m "feat: implement annotations with storage, querying, and @[deprecated] warning"
```

---

## Task 8: Implement AST Evaluator Builtin

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`
- Create: `tests/spec/07-macros/ast_eval_basic.opl`

Note: This implements Opal's `eval()` for AST metaprogramming — equivalent to
Elixir's `Code.eval_quoted/2`. It only accepts `Value::Ast` (AST nodes captured
via `ast ... end` blocks), not arbitrary strings.

**Step 1: Write failing spec test**

`tests/spec/07-macros/ast_eval_basic.opl`:
```opal
# expect: 20 | still 5
x = 5
code = ast
  x = 10
  x * 2
end
result = eval(code)
print(f"{result} | still {x}")
```

**Step 2: Implement in eval_call**

```rust
"eval" if arg_values.len() == 1 => {
    match &arg_values[0] {
        Value::Ast(ast_id) => {
            let stmts = self.ast_nodes[ast_id.0].clone();
            self.env.push_scope();
            let mut result = Value::Null;
            for stmt in &stmts {
                result = self.eval_stmt(stmt)?;
            }
            self.env.pop_scope();
            return Ok(result);
        }
        _ => {
            return Err(EvalError::TypeError(
                "eval() requires an AST value (from ast ... end block)".into(),
            ));
        }
    }
}
```

**Step 3: Write unit tests**

```rust
#[test]
fn ast_eval_basic() {
    assert_eq!(run("code = ast\n  2 + 3\nend\nprint(eval(code))").unwrap(), "5");
}

#[test]
fn ast_eval_child_scope() {
    assert_eq!(run("x = 1\ncode = ast\n  x = 99\nend\neval(code)\nprint(x)").unwrap(), "1");
}

#[test]
fn ast_eval_reads_parent() {
    assert_eq!(run("x = 10\ncode = ast\n  x * 2\nend\nprint(eval(code))").unwrap(), "20");
}
```

**Step 4: Run all tests and commit**

```bash
git add crates/ tests/
git commit -m "feat: implement eval() builtin for AST evaluation in child scope"
```

---

## Task 9: Integration Tests & Example App

**Files:**
- Create: `tests/spec/04-classes/enum_is_typeof.opl`
- Create: `tests/spec/10-examples/types/main.opl`
- Create: `tests/spec/10-examples/types/shapes.opl`

**Step 1: Write integration spec test**

`tests/spec/04-classes/enum_is_typeof.opl`:
```opal
# expect: Shape | true | false
enum Shape
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
end
s = Shape.Circle(5.0)
print(f"{typeof(s).name} | {s is Shape} | {s is Int}")
```

**Step 2: Write example app**

A multi-file example combining all 6 features. Uses enum Shape from a module, type aliases, typeof, is, annotations, and the AST evaluator.

**Step 3: Run all tests and commit**

```bash
git add tests/
git commit -m "feat: add integration tests and types example app"
```

---

## Task 10: Final Verification & Memory Update

**Step 1: Run full test suite**

```bash
cargo test && bash tests/run_spec.sh
```

**Step 2: Verify all 6 features work together**

Run the types example app manually and verify output.

**Step 3: Update MEMORY.md**

Update memory file with new features, test counts, and architecture notes.
