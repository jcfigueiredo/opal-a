# Inheritance Design

## Goal

Implement single inheritance as defined in the Opal spec: `class Child < Parent`, inherited needs, method override with parent chain lookup, `super()`, and `is` ancestry checks.

## Syntax

```opal
class Animal
  needs name: String
  needs sound: String

  def speak()
    f"{.name} says {.sound}"
  end
end

class Dog < Animal
  needs breed: String

  def speak()
    f"{super()} ({.breed})"
  end
end

rex = Dog.new(name: "Rex", sound: "Woof", breed: "Labrador")
rex.speak()    # => "Rex says Woof (Labrador)"
rex is Animal  # => true
rex is Dog     # => true
```

## Semantics

1. **`class Child < Parent`** — single parent, parent must be defined first
2. **Inherited needs** — child gets parent needs + own. Constructor requires all.
3. **Method override** — child methods shadow parent methods of same name
4. **Method lookup chain** — method not on child → look up parent → grandparent → etc.
5. **`super()`** — calls parent's version of current method with given args. Only valid inside an overriding method.
6. **`Self.new()`** — already implemented, naturally returns correct subclass type
7. **`is` operator** — walks ancestry chain: `dog is Animal` → true

## Changes Per Layer

### AST (`crates/opal-parser/src/ast.rs`)
- Add `parent: Option<String>` to `ClassDef`

### Lexer (`crates/opal-lexer/src/token.rs`)
- Add `Super` keyword token

### Parser (`crates/opal-parser/src/parser.rs`)
- In `parse_class_def`: after class name, check for `Token::Lt` then parse parent identifier
- Add `super` as expression: `ExprKind::Super(Vec<Expr>)` — parsed as `super()` or `super(args)`

### Interpreter (`crates/opal-interp/src/eval.rs`)
- `StoredClass`: add `parent: Option<ClassId>` field
- Class definition: resolve parent name to ClassId, store it
- **Inherited needs**: when constructing, merge parent needs (recursively) with child needs
- **Method lookup**: `call_method` on instance checks class methods, then walks parent chain
- **`super()`**: track current method name, look up parent class, call same-named method on parent
- **`is` operator**: walk ancestry chain — if any ancestor matches the type name, return true

### Tree-sitter (`tree-sitter-opal/grammar.js`)
- Add `inherits_clause` to `class_definition`: `optional(seq('<', field('parent', $.identifier)))`
- Add `super_call` to `_expression`: `seq('super', '(', optional(args), ')')`

### Highlight queries (`tree-sitter-opal/queries/highlights.scm`)
- Add `"super"` to keyword list

### TextMate (`editors/vscode-opal/syntaxes/opal.tmLanguage.json`)
- Add `super` to `keyword.other` pattern

## Not In Scope

- Multiple inheritance (use protocols)
- Abstract classes (use protocols)
- `protected` visibility
- `super()` in `init` with constructor chaining (needs are merged automatically)
