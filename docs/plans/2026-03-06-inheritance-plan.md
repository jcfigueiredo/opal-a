# Inheritance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement single inheritance (`class Dog < Animal`) with inherited needs, method chain lookup, `super()`, and `is` ancestry checks across all layers.

**Architecture:** Add `parent: Option<String>` to AST, `parent: Option<ClassId>` to StoredClass, chain method lookup through parent classes, add `Super` keyword to lexer, parse `super(args)` as expression. Update tree-sitter and TextMate grammars for editor support.

**Tech Stack:** Rust (lexer, parser, interpreter), JavaScript (tree-sitter grammar.js), JSON (TextMate grammar)

**Reference files:**
- Design: `docs/plans/2026-03-06-inheritance-design.md`
- Spec: `docs/03-functions-and-types/classes-and-inheritance.md`
- Lexer: `crates/opal-lexer/src/token.rs`
- Parser: `crates/opal-parser/src/parser.rs:752` (`parse_class_def`)
- AST: `crates/opal-parser/src/ast.rs:46` (`ClassDef`)
- Interpreter: `crates/opal-interp/src/eval.rs:73` (`StoredClass`), `:3290` (method lookup), `:3649` (`value_is_type`)
- Tree-sitter: `tree-sitter-opal/grammar.js:251` (`class_definition`)
- TextMate: `editors/vscode-opal/syntaxes/opal.tmLanguage.json`

---

### Task 1: Add `Super` keyword to lexer and `parent` to AST

**Files:**
- Modify: `crates/opal-lexer/src/token.rs`
- Modify: `crates/opal-parser/src/ast.rs`

**Step 1: Add `Super` token to lexer**

In `crates/opal-lexer/src/token.rs`, after the `Self` token (around line 300), add:

```rust
#[token("super")]
Super,
```

**Step 2: Add `parent` field to ClassDef AST**

In `crates/opal-parser/src/ast.rs`, modify `ClassDef`:

```rust
ClassDef {
    name: String,
    parent: Option<String>,  // NEW
    needs: Vec<NeedsDecl>,
    methods: Vec<Stmt>,
    implements: Vec<String>,
},
```

**Step 3: Fix all match arms on ClassDef**

The new field will cause compiler errors in parser.rs, eval.rs, symbols.rs, goto_def.rs. Fix each by adding `parent` to the pattern and construction. Most are just `parent: None` for existing code.

**Step 4: Build and verify**

Run: `cargo build --workspace`
Expected: Compiles with no errors.

**Step 5: Commit**

```
feat: add Super keyword and parent field to ClassDef AST
```

---

### Task 2: Parse `class Child < Parent` syntax

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`
- Test: `crates/opal-parser/src/parser.rs` (unit tests at bottom)

**Step 1: Write failing test**

Add to parser tests:

```rust
#[test]
fn parse_class_with_parent() {
    let source = "class Dog < Animal\n  needs breed: String\nend\n";
    let program = crate::parse(source).unwrap();
    match &program.statements[0].kind {
        StmtKind::ClassDef { name, parent, needs, .. } => {
            assert_eq!(name, "Dog");
            assert_eq!(parent.as_deref(), Some("Animal"));
            assert_eq!(needs.len(), 1);
        }
        _ => panic!("expected ClassDef"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p opal-parser -- parse_class_with_parent`
Expected: FAIL — `<` is parsed as less-than operator, not inheritance.

**Step 3: Implement parsing**

In `parse_class_def` (parser.rs:752), after parsing the class name and before `implements`:

```rust
// Parse optional parent class: < ParentName
let parent = if self.check(&Token::Lt) {
    self.advance();
    Some(self.expect_identifier()?)
} else {
    None
};
```

Pass `parent` into the `ClassDef` construction.

**Step 4: Run test to verify it passes**

Run: `cargo test -p opal-parser -- parse_class_with_parent`
Expected: PASS

**Step 5: Commit**

```
feat: parse class Child < Parent syntax
```

---

### Task 3: Parse `super(args)` as expression

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`
- Modify: `crates/opal-parser/src/parser.rs`

**Step 1: Add Super to ExprKind**

In ast.rs, add to `ExprKind`:

```rust
/// super() call to parent method
Super(Vec<Expr>),
```

**Step 2: Write failing test**

```rust
#[test]
fn parse_super_call() {
    let source = "class Dog < Animal\n  def speak()\n    super()\n  end\nend\n";
    let program = crate::parse(source).unwrap();
    match &program.statements[0].kind {
        StmtKind::ClassDef { methods, .. } => {
            match &methods[0].kind {
                StmtKind::FuncDef { body, .. } => {
                    match &body[0].kind {
                        StmtKind::Expr(expr) => assert!(matches!(&expr.kind, ExprKind::Super(_))),
                        _ => panic!("expected expression statement"),
                    }
                }
                _ => panic!("expected FuncDef"),
            }
        }
        _ => panic!("expected ClassDef"),
    }
}
```

**Step 3: Run to verify failure**

Run: `cargo test -p opal-parser -- parse_super_call`

**Step 4: Implement**

In the parser's expression parsing (around `parse_primary`), add a case for `Token::Super`:

```rust
Token::Super => {
    self.advance();
    self.expect_token(&Token::LParen, "(")?;
    let args = if !self.check(&Token::RParen) {
        self.parse_call_args()?
    } else {
        vec![]
    };
    self.expect_token(&Token::RParen, ")")?;
    Ok(Expr {
        kind: ExprKind::Super(args),
        span: Span { start, end: self.previous_span().end },
    })
}
```

**Step 5: Run test, verify pass, commit**

```
feat: parse super(args) expression
```

---

### Task 4: Implement inherited needs and parent class storage

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing test**

```rust
#[test]
fn inheritance_basic_needs() {
    let output = run(
        "class Animal\n  needs name: String\nend\n\nclass Dog < Animal\n  needs breed: String\nend\n\nrex = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(f\"{rex.name} {rex.breed}\")",
    ).unwrap();
    assert_eq!(output, "Rex Lab");
}
```

**Step 2: Run to verify failure**

Run: `cargo test -p opal-interp -- inheritance_basic_needs`

**Step 3: Implement**

In `StoredClass`, add `parent: Option<ClassId>`.

In the `ClassDef` evaluation (where classes are stored), resolve the parent name to a `ClassId`:

```rust
let parent_id = if let Some(parent_name) = &parent {
    let pid = self.env.get(parent_name)
        .and_then(|v| if let Value::Class(id) = v { Some(*id) } else { None })
        .ok_or_else(|| EvalError::UndefinedVariable(parent_name.clone()))?;
    Some(pid)
} else {
    None
};
```

When constructing `.new()`, gather all needs by walking the parent chain:

```rust
fn gather_all_needs(&self, class_id: ClassId) -> Vec<(String, Option<String>, Option<Expr>)> {
    let class = &self.classes[class_id.0];
    let mut all_needs = if let Some(pid) = class.parent {
        self.gather_all_needs(pid)
    } else {
        vec![]
    };
    all_needs.extend(class.needs.clone());
    all_needs
}
```

Use `gather_all_needs` in the `.new()` handler instead of just `class.needs`.

**Step 4: Run test, verify pass**

**Step 5: Add test for deep inheritance**

```rust
#[test]
fn inheritance_three_levels() {
    let output = run(
        "class A\n  needs x: Int\nend\nclass B < A\n  needs y: Int\nend\nclass C < B\n  needs z: Int\nend\nc = C.new(x: 1, y: 2, z: 3)\nprint(f\"{c.x} {c.y} {c.z}\")",
    ).unwrap();
    assert_eq!(output, "1 2 3");
}
```

**Step 6: Run all tests, commit**

```
feat: implement inherited needs with parent class chain
```

---

### Task 5: Implement method lookup chain and method override

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn inheritance_method_from_parent() {
    let output = run(
        "class Animal\n  needs name: String\n\n  def speak()\n    f\"{.name} speaks\"\n  end\nend\n\nclass Dog < Animal\n  needs breed: String\nend\n\nrex = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(rex.speak())",
    ).unwrap();
    assert_eq!(output, "Rex speaks");
}

#[test]
fn inheritance_method_override() {
    let output = run(
        "class Animal\n  needs name: String\n\n  def speak()\n    f\"{.name} speaks\"\n  end\nend\n\nclass Dog < Animal\n  needs breed: String\n\n  def speak()\n    f\"{.name} barks\"\n  end\nend\n\nrex = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(rex.speak())",
    ).unwrap();
    assert_eq!(output, "Rex barks");
}
```

**Step 2: Run to verify failure**

**Step 3: Implement**

In the method lookup section (around line 3290), after searching the instance's class methods, if not found, walk the parent chain:

```rust
// After: .or_else(|| class.methods.iter().find(|m| m.name == method));
// Add parent chain lookup:
let method_fn = method_fn.or_else(|| {
    let mut current = class.parent;
    while let Some(pid) = current {
        let parent = &self.classes[pid.0];
        if let Some(f) = parent.methods.iter().find(|m| m.name == method) {
            return Some(f);
        }
        current = parent.parent;
    }
    None
});
```

**Step 4: Run tests, verify pass, commit**

```
feat: implement method lookup chain and method override
```

---

### Task 6: Implement `super()` dispatch

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing test**

```rust
#[test]
fn inheritance_super_call() {
    let output = run(
        "class Animal\n  needs name: String\n\n  def speak()\n    f\"{.name} speaks\"\n  end\nend\n\nclass Dog < Animal\n  needs breed: String\n\n  def speak()\n    f\"{super()} loudly\"\n  end\nend\n\nrex = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(rex.speak())",
    ).unwrap();
    assert_eq!(output, "Rex speaks loudly");
}
```

**Step 2: Run to verify failure**

**Step 3: Implement**

Add `current_method_name: Option<String>` and `current_class_id: Option<ClassId>` to the Interpreter struct. Set them when entering a method call.

In `eval_expr`, handle `ExprKind::Super(args)`:

```rust
ExprKind::Super(args) => {
    let method_name = self.current_method_name.clone()
        .ok_or_else(|| EvalError::RuntimeError("super() outside of method".into()))?;
    let class_id = self.current_class_id
        .ok_or_else(|| EvalError::RuntimeError("super() outside of class".into()))?;
    let parent_id = self.classes[class_id.0].parent
        .ok_or_else(|| EvalError::RuntimeError("super() in class with no parent".into()))?;

    let instance = Value::Instance(self.current_self.unwrap());
    let eval_args: Vec<(Option<String>, Value)> = /* eval args */;

    // Find method on parent (walking chain)
    // Call it with current self
}
```

**Step 4: Run test, verify pass**

**Step 5: Add test for super with args**

```rust
#[test]
fn inheritance_super_with_args() {
    let output = run(
        "class Base\n  needs x: Int\n\n  def calc(n)\n    .x + n\n  end\nend\n\nclass Child < Base\n  def calc(n)\n    super(n) * 2\n  end\nend\n\nc = Child.new(x: 10)\nprint(c.calc(5))",
    ).unwrap();
    assert_eq!(output, "30");
}
```

**Step 6: Run all tests, commit**

```
feat: implement super() dispatch to parent method
```

---

### Task 7: Update `is` operator for ancestry checking

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing test**

```rust
#[test]
fn inheritance_is_operator() {
    let output = run(
        "class Animal\n  needs name: String\nend\n\nclass Dog < Animal\n  needs breed: String\nend\n\nrex = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(f\"{rex is Dog} {rex is Animal}\")",
    ).unwrap();
    assert_eq!(output, "true true");
}
```

**Step 2: Run to verify failure** (currently `rex is Animal` returns false)

**Step 3: Implement**

In `value_is_type` (around line 3670), after checking the class name, walk the parent chain:

```rust
name => {
    if let Value::Instance(id) = value {
        let inst = &self.instances[id.0];
        // Check class and all ancestors
        let mut current_id = Some(inst.class_id);
        while let Some(cid) = current_id {
            if self.classes[cid.0].name == name {
                return true;
            }
            current_id = self.classes[cid.0].parent;
        }
        return self.class_implements_protocol(inst.class_id, name);
    }
    // ...
}
```

**Step 4: Run test, verify pass, commit**

```
feat: is operator walks ancestry chain
```

---

### Task 8: Update tree-sitter grammar and highlight queries

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Modify: `tree-sitter-opal/queries/highlights.scm`
- Create: `tree-sitter-opal/test/corpus/inheritance.txt`

**Step 1: Add inheritance clause and super to grammar**

In `grammar.js`, modify `class_definition`:

```javascript
class_definition: $ => seq(
  'class',
  field('name', $.identifier),
  optional(seq('<', field('parent', $.identifier))),
  optional($.implements_clause),
  repeat(choice(
    $.needs_declaration,
    $.function_definition,
  )),
  'end',
),
```

Add `super_call` to `_expression`:

```javascript
super_call: $ => seq(
  'super',
  '(',
  optional(seq($._expression, repeat(seq(',', $._expression)))),
  ')',
),
```

**Step 2: Add `"super"` to highlights.scm keywords**

**Step 3: Write corpus test**

```
================
Class with parent
================

class Dog < Animal
  needs breed: String
end

---

(source_file
  (class_definition
    (identifier)
    (identifier)
    (needs_declaration (identifier) (type_annotation (identifier)))))
```

**Step 4: Generate and test**

Run: `cd tree-sitter-opal && pnpm run generate && pnpm run test`

**Step 5: Commit**

```
feat(tree-sitter): add inheritance clause and super keyword
```

---

### Task 9: Update TextMate grammar and Cursor extension

**Files:**
- Modify: `editors/vscode-opal/syntaxes/opal.tmLanguage.json`

**Step 1: Add `super` to keyword.other pattern**

Change the keyword.other match to include `super`:

```
"match": "\\b(let|needs|requires|import|from|export|as|implements|with|where|defaults|receive|reply|send|await|emit|on|extern|parallel|async|public|private|super)\\b"
```

**Step 2: Rebuild and reinstall Cursor extension**

Run: `./scripts/setup-cursor-extension.sh`

**Step 3: Commit**

```
feat: add super keyword to Cursor/VS Code extension
```

---

### Task 10: Add spec tests and rebuild LSP

**Files:**
- Create: `tests/spec/03-functions-and-types/inheritance.opl`
- Create: `tests/spec/03-functions-and-types/inheritance_super.opl`
- Create: `tests/spec/03-functions-and-types/inheritance_is.opl`

**Step 1: Write spec tests**

`inheritance.opl`:
```opal
# expect: Rex Lab | Rex speaks | Rex barks | 1 2 3

class Animal
  needs name: String
  def speak()
    f"{.name} speaks"
  end
end

class Dog < Animal
  needs breed: String
  def speak()
    f"{.name} barks"
  end
end

class A
  needs x: Int
end
class B < A
  needs y: Int
end
class C < B
  needs z: Int
end

rex = Dog.new(name: "Rex", breed: "Lab")
animal = Animal.new(name: "Rex")
c = C.new(x: 1, y: 2, z: 3)

results = [
  f"{rex.name} {rex.breed}",
  animal.speak(),
  rex.speak(),
  f"{c.x} {c.y} {c.z}"
]
print(results.join(" | "))
```

`inheritance_super.opl`:
```opal
# expect: Rex speaks loudly | 30 | base middle top

class Animal
  needs name: String
  def speak()
    f"{.name} speaks"
  end
end

class Dog < Animal
  needs breed: String
  def speak()
    f"{super()} loudly"
  end
end

class Base
  needs x: Int
  def calc(n)
    .x + n
  end
end

class Child < Base
  def calc(n)
    super(n) * 2
  end
end

class A
  def chain()
    "base"
  end
end
class B < A
  def chain()
    f"{super()} middle"
  end
end
class C < B
  def chain()
    f"{super()} top"
  end
end

rex = Dog.new(name: "Rex", breed: "Lab")
c = Child.new(x: 10)
deep = C.new()

results = [rex.speak(), f"{c.calc(5)}", deep.chain()]
print(results.join(" | "))
```

`inheritance_is.opl`:
```opal
# expect: true | true | true | false | true

class Animal
  needs name: String
end

class Dog < Animal
  needs breed: String
end

rex = Dog.new(name: "Rex", breed: "Lab")
a = Animal.new(name: "Cat")

results = [
  f"{rex is Dog}",
  f"{rex is Animal}",
  f"{a is Animal}",
  f"{a is Dog}",
  f"{rex is Dog and rex is Animal}"
]
print(results.join(" | "))
```

**Step 2: Run specs**

Run: `./tests/run_spec.sh tests/spec/03-functions-and-types/inheritance*.opl`

**Step 3: Rebuild LSP**

Run: `cargo build --release -p opal-lsp`

**Step 4: Run full test suite**

Run: `cargo test --workspace && ./tests/run_spec.sh`

**Step 5: Commit**

```
test: add inheritance spec tests (needs, super, is operator)
```

---

## Summary

| Task | Deliverable |
|------|-------------|
| 1 | `Super` token + `parent` field in AST |
| 2 | Parse `class Child < Parent` |
| 3 | Parse `super(args)` expression |
| 4 | Inherited needs + parent class storage |
| 5 | Method lookup chain + override |
| 6 | `super()` dispatch |
| 7 | `is` ancestry checking |
| 8 | Tree-sitter grammar + corpus tests |
| 9 | TextMate grammar + Cursor extension |
| 10 | Spec tests + LSP rebuild |
