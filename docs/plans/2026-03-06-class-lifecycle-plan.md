# Class Lifecycle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `init()` lifecycle, `to_string()` protocol, `def self.method()`, and `Type(args)` shorthand.

**Architecture:** All changes are in the interpreter (`crates/opal-interp/src/eval.rs`) except `def self.method()` which also needs parser changes. No lexer changes needed.

**Tech Stack:** Rust

**Reference files:**
- Design: `docs/plans/2026-03-06-class-lifecycle-design.md`
- Interpreter: `crates/opal-interp/src/eval.rs` — `.new()` handler (~line 3090), `call_method` (~line 2442), `value_to_string` (search for `<.*instance>`)
- Parser: `crates/opal-parser/src/parser.rs` — `parse_class_def` (~line 752), `expect_method_name` (~line 2703)
- AST: `crates/opal-parser/src/ast.rs` — `ClassDef`, `StmtKind::FuncDef`

---

### Task 1: Implement `init()` auto-call in `.new()`

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn init_sets_fields() {
    let output = run("class Foo\n  def init()\n    .x = 42\n  end\nend\nf = Foo.new()\nprint(f.x)").unwrap();
    assert_eq!(output, "42");
}

#[test]
fn init_after_needs() {
    let output = run("class Foo\n  needs x: Int\n  def init()\n    .doubled = .x * 2\n  end\nend\nf = Foo.new(x: 5)\nprint(f\"{f.x} {f.doubled}\")").unwrap();
    assert_eq!(output, "5 10");
}

#[test]
fn init_with_super() {
    let output = run("class Base\n  needs x: Int\n  def init()\n    .computed = .x * 10\n  end\nend\nclass Child < Base\n  needs y: Int\n  def init()\n    super()\n    .total = .computed + .y\n  end\nend\nc = Child.new(x: 3, y: 5)\nprint(f\"{c.computed} {c.total}\")").unwrap();
    assert_eq!(output, "30 35");
}

#[test]
fn init_not_required() {
    let output = run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 7)\nprint(f.x)").unwrap();
    assert_eq!(output, "7");
}
```

**Step 2: Implement**

In the `.new()` handler (search for `"new"` in the `Value::Class` method call section, around line 3090-3130), after the instance is created and fields are set from `needs`, add:

```rust
// After instance creation, call init() if it exists
let init_method = {
    let class = &self.classes[class_id.0];
    // Search this class and parents for init
    let mut found = None;
    let mut search = Some(class_id);
    while let Some(cid) = search {
        let c = &self.classes[cid.0];
        if let Some(m) = c.methods.iter().find(|m| m.name == "init") {
            found = Some(m.clone());
            break;
        }
        search = c.parent;
    }
    found
};

if let Some(init_fn) = init_method {
    let prev_self = self.current_self;
    let prev_method = self.current_method_name.take();
    let prev_class = self.current_class_id.take();
    self.current_self = Some(instance_id);
    self.current_method_name = Some("init".to_string());
    self.current_class_id = Some(/* the class where init was found */);
    self.env.push_scope();
    self.env.set("self".to_string(), Value::Instance(instance_id));
    let result = self.eval_block(&init_fn.body);
    self.env.pop_scope();
    self.current_self = prev_self;
    self.current_method_name = prev_method;
    self.current_class_id = prev_class;
    match result {
        Ok(_) | Err(EvalError::Return(_)) => {}
        Err(e) => return Err(e),
    }
}
```

**Note:** Find the exact instance creation code and adapt. The key is: `init()` runs AFTER needs fields are injected, with `self` set to the new instance. `current_class_id` must be set to the class where `init` was actually found (for `super()` to work).

**Step 3: Run tests, commit**

```
feat: implement init() auto-call in .new() with super() support
```

---

### Task 2: Implement `to_string()` protocol

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn to_string_in_print() {
    let output = run("class Dog\n  needs name: String\n  def to_string()\n    f\"Dog({.name})\"\n  end\nend\nd = Dog.new(name: \"Rex\")\nprint(d)").unwrap();
    assert_eq!(output, "Dog(Rex)");
}

#[test]
fn to_string_in_fstring() {
    let output = run("class Dog\n  needs name: String\n  def to_string()\n    f\"Dog({.name})\"\n  end\nend\nd = Dog.new(name: \"Rex\")\nprint(f\"my {d}\")").unwrap();
    assert_eq!(output, "my Dog(Rex)");
}

#[test]
fn to_string_inherited() {
    let output = run("class Animal\n  needs name: String\n  def to_string()\n    .name\n  end\nend\nclass Dog < Animal\n  needs breed: String\nend\nd = Dog.new(name: \"Rex\", breed: \"Lab\")\nprint(f\"{d}\")").unwrap();
    assert_eq!(output, "Rex");
}

#[test]
fn to_string_fallback() {
    let output = run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 1)\nprint(f\"{f}\")").unwrap();
    // Should still show default if no to_string defined
    assert!(output.contains("instance"));
}
```

**Step 2: Implement**

Find where `Value::Instance` is converted to string. Search for `"<"` and `"instance>"` in eval.rs — this is likely in a `value_to_string` function or in the `Display` impl or in the f-string evaluation.

When converting an instance to string, check if the class (or parent chain) has a `to_string` method. If so, call it:

```rust
// In the instance-to-string conversion:
Value::Instance(id) => {
    // Try to_string() method
    match self.call_method(Value::Instance(id), "to_string", vec![]) {
        Ok(Value::String(s)) => s,
        _ => format!("<{} instance>", self.classes[self.instances[id.0].class_id.0].name),
    }
}
```

Apply this in ALL places instances are stringified: `print()`, f-string interpolation, string concatenation.

**Step 3: Run tests, commit**

```
feat: implement to_string() protocol for print and f-strings
```

---

### Task 3: Implement `def self.method()` static methods

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`
- Modify: `crates/opal-parser/src/ast.rs` (maybe — might reuse FuncDef with a flag)
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn static_method() {
    let output = run("class MathUtils\n  def self.max(a, b)\n    if a > b then a else b end\n  end\nend\nprint(MathUtils.max(3, 7))").unwrap();
    assert_eq!(output, "7");
}

#[test]
fn static_method_no_instance() {
    let output = run("class Person\n  needs name: String\n  def self.species()\n    \"Homo sapiens\"\n  end\nend\nprint(Person.species())").unwrap();
    assert_eq!(output, "Homo sapiens");
}

#[test]
fn static_method_inherited() {
    let output = run("class Animal\n  def self.kingdom()\n    \"Animalia\"\n  end\nend\nclass Dog < Animal\nend\nprint(Dog.kingdom())").unwrap();
    assert_eq!(output, "Animalia");
}
```

**Step 2: Parse `def self.method()`**

In the parser, when inside a class body parsing methods, check if after `def` we see `Token::SelfKw` followed by `Token::Dot`. If so, parse `self.name(params)` as a static method.

Option A: Add `is_static: bool` to `FuncDef` AST variant.
Option B: Add a new `StaticMethodDef` variant.

Recommend Option A — simpler, reuses existing method infrastructure.

In `parse_function_def_with_visibility`, before calling `expect_method_name`, check:
```rust
let is_static = if self.check(&Token::SelfKw) {
    // Peek ahead for dot
    if self.peek_at(1) == Some(&Token::Dot) {
        self.advance(); // consume self
        self.advance(); // consume .
        true
    } else {
        false
    }
} else {
    false
};
```

**Step 3: Store and dispatch static methods**

In `StoredClass`, add `static_methods: Vec<StoredFunction>`.

When `Value::Class(id)` is used with method call (e.g., `MathUtils.max(3, 7)`), check static_methods. This already partially works for `.new()` — extend it to handle user-defined static methods.

**Step 4: Run tests, commit**

```
feat: implement def self.method() static/class methods
```

---

### Task 4: Implement `Type(args)` constructor shorthand

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn constructor_shorthand() {
    let output = run("class Point\n  needs x: Int\n  needs y: Int\nend\np = Point(x: 1, y: 2)\nprint(f\"{p.x} {p.y}\")").unwrap();
    assert_eq!(output, "1 2");
}

#[test]
fn constructor_shorthand_with_inheritance() {
    let output = run("class Animal\n  needs name: String\nend\nclass Dog < Animal\n  needs breed: String\nend\nd = Dog(name: \"Rex\", breed: \"Lab\")\nprint(f\"{d.name} {d.breed}\")").unwrap();
    assert_eq!(output, "Rex Lab");
}

#[test]
fn constructor_shorthand_equivalent() {
    let output = run("class Foo\n  needs x: Int\n  def +(other)\n    Self.new(x: .x + other.x)\n  end\nend\na = Foo(x: 1) + Foo(x: 2)\nprint(a.x)").unwrap();
    assert_eq!(output, "3");
}
```

**Step 2: Implement**

In the function call evaluation (where `ExprKind::Call` or `ExprKind::FunctionCall` is handled), when the callee evaluates to `Value::Class(id)`, delegate to the `.new()` handler with the same arguments.

Find where function calls are evaluated. If the callee is a `Value::Class`, treat it as `.new()`:

```rust
// In call expression evaluation:
Value::Class(class_id) => {
    // Type(args) is sugar for Type.new(args)
    self.call_class_new(class_id, named_args)?
}
```

Extract the `.new()` logic into a reusable `call_class_new` method if it isn't already.

**Step 3: Run tests, commit**

```
feat: implement Type(args) constructor shorthand
```

---

### Task 5: Spec tests and tooling updates

**Files:**
- Create: `tests/spec/03-functions-and-types/init_lifecycle.opl`
- Create: `tests/spec/03-functions-and-types/to_string_protocol.opl`
- Create: `tests/spec/03-functions-and-types/static_methods.opl`
- Create: `tests/spec/03-functions-and-types/constructor_shorthand.opl`
- Rebuild LSP and Cursor extension

**Step 1: Write spec tests**

`init_lifecycle.opl`:
```opal
# expect: 42 | 5 10 | 30 35 | 7

class Simple
  def init()
    .x = 42
  end
end

class WithNeeds
  needs x: Int
  def init()
    .doubled = .x * 2
  end
end

class Base
  needs x: Int
  def init()
    .computed = .x * 10
  end
end

class Child < Base
  needs y: Int
  def init()
    super()
    .total = .computed + .y
  end
end

class NeedsOnly
  needs x: Int
end

s = Simple.new()
w = WithNeeds.new(x: 5)
c = Child.new(x: 3, y: 5)
n = NeedsOnly.new(x: 7)

results = [f"{s.x}", f"{w.x} {w.doubled}", f"{c.computed} {c.total}", f"{n.x}"]
print(results.join(" | "))
```

`to_string_protocol.opl`:
```opal
# expect: Dog(Rex) | my Dog(Rex) | Rex | <has no to_string>

class Dog
  needs name: String
  def to_string()
    f"Dog({.name})"
  end
end

class Animal
  needs name: String
  def to_string()
    .name
  end
end

class Cat < Animal
  needs indoor: Bool
end

class Plain
  needs x: Int
end

d = Dog.new(name: "Rex")
cat = Cat.new(name: "Rex", indoor: true)
p = Plain.new(x: 1)

results = [f"{d}", f"my {d}", f"{cat}", "<has no to_string>"]
print(results.join(" | "))
```

Note: The `Plain` test is tricky — if it shows `<Plain instance>` we can't easily match it. Use a static string instead.

`static_methods.opl`:
```opal
# expect: 7 | Homo sapiens | Animalia

class MathUtils
  def self.max(a, b)
    if a > b then a else b end
  end
end

class Person
  needs name: String
  def self.species()
    "Homo sapiens"
  end
end

class Animal
  def self.kingdom()
    "Animalia"
  end
end

class Dog < Animal
end

results = [f"{MathUtils.max(3, 7)}", Person.species(), Dog.kingdom()]
print(results.join(" | "))
```

`constructor_shorthand.opl`:
```opal
# expect: 1 2 | Rex Lab | 3

class Point
  needs x: Int
  needs y: Int
end

class Animal
  needs name: String
end

class Dog < Animal
  needs breed: String
end

class Num
  needs x: Int
  def +(other)
    Self.new(x: .x + other.x)
  end
end

p = Point(x: 1, y: 2)
d = Dog(name: "Rex", breed: "Lab")
n = Num(x: 1) + Num(x: 2)

results = [f"{p.x} {p.y}", f"{d.name} {d.breed}", f"{n.x}"]
print(results.join(" | "))
```

**Step 2: Rebuild and run**

```bash
cargo build --release -p opal-lsp
./scripts/setup-cursor-extension.sh
cargo test --workspace
./tests/run_spec.sh
```

**Step 3: Commit**

```
test: add spec tests for init, to_string, static methods, Type() shorthand
```

---

## Summary

| Task | Deliverable |
|------|-------------|
| 1 | `init()` auto-call in `.new()` with `super()` support |
| 2 | `to_string()` protocol for print/f-strings |
| 3 | `def self.method()` static methods |
| 4 | `Type(args)` constructor shorthand |
| 5 | Spec tests + tooling rebuild |
