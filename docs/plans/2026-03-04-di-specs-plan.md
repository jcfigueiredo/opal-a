# DI Specs & Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add spec tests and interpreter support for all DI features: needs with defaults, needs on modules, needs on actors, protocol-typed needs validation, Container class, and domain events.

**Architecture:** Each feature follows TDD — write the spec first, verify it fails, implement the minimal interpreter change, verify it passes. Features build on each other: defaults → modules → actors → protocol validation → container → events.

**Tech Stack:** Rust (logos lexer, recursive descent parser, tree-walk interpreter), Opal spec tests (`tests/spec/` with `# expect:` headers)

---

### Task 1: Spec — needs with defaults

**Files:**
- Create: `tests/spec/06-patterns/needs_defaults.opl`

**Step 1: Write the spec**

```opal
# expect: ConsoleLogger | CustomLogger
# needs with default values

class ConsoleLogger
  def log(msg)
    "ConsoleLogger"
  end
end

class CustomLogger
  def log(msg)
    "CustomLogger"
  end
end

class BountyService
  needs logger: Logger = ConsoleLogger.new()

  def who()
    .logger.log("test")
  end
end

# Use default
svc1 = BountyService.new()
print(svc1.who())

# Override default
svc2 = BountyService.new(logger: CustomLogger.new())
print(svc2.who())
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/needs_defaults.opl 2>&1`
Expected: parse error or runtime error (defaults not supported yet)

---

### Task 2: Implement needs with defaults

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `default: Option<Expr>` to `NeedsDecl`
- Modify: `crates/opal-parser/src/parser.rs` — parse `= expr` after type annotation in `parse_needs_decl`
- Modify: `crates/opal-interp/src/eval.rs` — store defaults in `StoredClass.needs`, use fallback in `.new()` resolution

**Step 1: Add default to NeedsDecl AST**

In `crates/opal-parser/src/ast.rs`, change `NeedsDecl`:

```rust
pub struct NeedsDecl {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default: Option<Expr>,
}
```

**Step 2: Parse default expression**

In `crates/opal-parser/src/parser.rs`, in `parse_needs_decl` (line ~813), after the type annotation parsing, add:

```rust
let default = if self.check(&Token::Assign) {
    self.advance();
    Some(self.parse_expression()?)
} else {
    None
};
```

And include `default` in the `NeedsDecl` construction. Also update `parse_class_def` parser test at line ~2989 to expect the new field.

**Step 3: Store defaults in StoredClass**

In `crates/opal-interp/src/eval.rs`, change `StoredClass.needs` type from `Vec<(String, Option<String>)>` to `Vec<(String, Option<String>, Option<Expr>)>`.

Update all places that build this vec (ClassDef handler ~line 759, ModelDef handler ~line 1135) to include the default:

```rust
needs: needs.iter().map(|n| (n.name.clone(), n.type_annotation.clone(), n.default.clone())).collect(),
```

For `ModelNeedsDecl` (models), defaults aren't applicable — pass `None`.

**Step 4: Use default in .new() resolution**

In the `(Value::Class(class_id), "new")` handler (~line 2949), change the error branch to try the default:

```rust
} else {
    // Try positional
    let idx = class.needs.iter().position(|(n, _, _)| n == need_name).unwrap();
    if idx < args.len() {
        fields.insert(need_name.clone(), args[idx].clone());
    } else if let Some(default_expr) = default {
        let val = self.eval_expr(default_expr)?;
        fields.insert(need_name.clone(), val);
    } else {
        return Err(EvalError::TypeError(format!(
            "missing required field '{}' in .new()", need_name
        )));
    }
}
```

The loop needs to iterate over `(need_name, _type, default)` instead of `(need_name, _)`.

**Step 5: Fix all compilation errors**

Update all other references to `class.needs` that destructure as tuples of 2 to tuples of 3. Key locations:
- `typeof(f).fields` handler (~line 1660, 3203): `class.needs.iter().map(|(name, type_ann, _)| ...)`
- `to_dict` handler (~line 3017): `class.needs.iter().map(|(name, _, _)| ...)`
- `copy` handler: similar tuple update

**Step 6: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `needs_defaults.opl`

**Step 7: Commit**

```
feat: support needs with default values
```

---

### Task 3: Spec — needs on modules

**Files:**
- Create: `tests/spec/06-patterns/needs_modules.opl`

**Step 1: Write the spec**

```opal
# expect: 750.0 | 250.0
# Module-level dependency injection with needs

module RewardCalculator
  needs multiplier: Float

  def calculate(base)
    base * .multiplier
  end
end

# Wire module with dependencies
calc1 = RewardCalculator.new(multiplier: 2.5)
print(calc1.calculate(300.0))

calc2 = RewardCalculator.new(multiplier: 1.0)
print(calc2.calculate(250.0))
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/needs_modules.opl 2>&1`
Expected: error (modules don't support `needs` or `.new()`)

---

### Task 4: Implement needs on modules

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `needs: Vec<NeedsDecl>` to `ModuleDef`
- Modify: `crates/opal-parser/src/parser.rs` — parse `needs` inside module body
- Modify: `crates/opal-interp/src/eval.rs` — handle module `needs` and `.new()`

**Step 1: Add needs to ModuleDef AST**

In `ast.rs`, change:

```rust
ModuleDef { name: String, needs: Vec<NeedsDecl>, body: Vec<Stmt> },
```

**Step 2: Parse needs in module body**

In `parser.rs`, find `parse_module_def` (or wherever `ModuleDef` is parsed). Add `needs` collection before the body loop — similar to `parse_class_def`. If the module body parsing currently just collects all statements, split it: collect `needs` declarations first, put the rest in `body`.

Look at how the parser currently handles `ModuleDef` — it likely parses all statements in the body. Change it to check for `Token::Needs` and collect them separately, similar to class parsing.

**Step 3: Implement module as instantiable**

This is the trickiest part. Currently modules are evaluated eagerly — body runs immediately and bindings are captured. With `needs`, modules need to support `.new()` like classes.

Approach: When a module has `needs`, treat it like a class with methods. Store it as a class internally (reuse existing class infrastructure) rather than a traditional module. In the `ModuleDef` handler:

- If `needs` is empty: current behavior (evaluate body, store bindings)
- If `needs` is non-empty: store as a class with the module's functions as methods

This keeps the implementation minimal. The `needs` fields become constructor args, and `def` statements become methods.

**Step 4: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `needs_modules.opl`

**Step 5: Commit**

```
feat: support needs on modules for dependency injection
```

---

### Task 5: Spec — needs on actors

**Files:**
- Create: `tests/spec/06-patterns/needs_actors.opl`

**Step 1: Write the spec**

```opal
# expect: 750.0
# Actor dependency injection with needs

actor AwardProcessor
  needs rate: Float

  def init()
    .total = 0.0
  end

  receive
    case :process(amount)
      .total = .total + (amount * .rate)
    case :get_total
      reply .total
  end
end

proc = AwardProcessor.new(rate: 2.5)
proc.send(:process(100.0))
proc.send(:process(200.0))
total = await proc.send(:get_total)
print(total)
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/needs_actors.opl 2>&1`
Expected: error (actors don't support `needs`)

---

### Task 6: Implement needs on actors

**Files:**
- Modify: `crates/opal-parser/src/ast.rs` — add `needs: Vec<NeedsDecl>` to `ActorDef`
- Modify: `crates/opal-parser/src/parser.rs` — parse `needs` in actor body (line ~1364)
- Modify: `crates/opal-interp/src/eval.rs` — store needs in `StoredActorDef`, inject in `.new()`

**Step 1: Add needs to ActorDef AST**

```rust
ActorDef {
    name: String,
    needs: Vec<NeedsDecl>,
    init: Option<Vec<Stmt>>,
    receive_cases: Vec<MatchCase>,
    methods: Vec<Stmt>,
},
```

**Step 2: Parse needs in actor body**

In `parse_actor_def` (~line 1354), add a `needs` vec and check for `Token::Needs` in the while loop, before the `def` check:

```rust
let mut needs = Vec::new();
// ... in while loop:
if self.check(&Token::Needs) {
    let stmt = self.parse_needs_decl(self.current_span())?;
    if let StmtKind::NeedsDecl(decl) = stmt.kind {
        needs.push(decl);
    }
} else if self.check(&Token::Def) {
```

**Step 3: Store needs in StoredActorDef**

```rust
struct StoredActorDef {
    name: String,
    needs: Vec<(String, Option<String>, Option<Expr>)>,
    init: Option<Vec<Stmt>>,
    receive_cases: Vec<MatchCase>,
}
```

**Step 4: Inject needs in actor .new()**

In the `(Value::ActorClass(def_id), "new")` handler (~line 2886), add needs resolution before init:

```rust
let mut fields = HashMap::new();
// Resolve needs (same logic as class .new())
for (need_name, _, default) in &def.needs {
    let value = named_args.iter()
        .find(|(name, _)| name.as_deref() == Some(need_name.as_str()))
        .map(|(_, v)| v.clone());
    if let Some(val) = value {
        fields.insert(need_name.clone(), val);
    } else if let Some(default_expr) = default {
        let val = self.eval_expr(default_expr)?;
        fields.insert(need_name.clone(), val);
    } else {
        return Err(EvalError::TypeError(format!(
            "missing required field '{}' in actor .new()", need_name
        )));
    }
}
```

Then set those fields on the actor instance so `.rate` works in receive handlers.

**Step 5: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `needs_actors.opl`

**Step 6: Commit**

```
feat: support needs on actors for dependency injection
```

---

### Task 7: Spec — protocol-typed needs validation

**Files:**
- Create: `tests/spec/06-patterns/needs_protocol.opl`

**Step 1: Write the spec**

```opal
# expect: saved bounty | Error: BountyRepo.new() — 'store' must implement Storage
# Protocol validation on needs at construction time

protocol Storage
  def save(record) -> String
end

class MemoryStore implements Storage
  def save(record)
    f"saved {record}"
  end
end

class NotAStore
  def something()
    "nope"
  end
end

class BountyRepo
  needs store: Storage

  def persist(item)
    .store.save(item)
  end
end

# Conforming — works
repo = BountyRepo.new(store: MemoryStore.new())
print(repo.persist("bounty"))

# Non-conforming — error
try
  BountyRepo.new(store: NotAStore.new())
catch as e
  print(f"Error: {e.message}")
end
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/needs_protocol.opl 2>&1`
Expected: the conforming case works, but the non-conforming case does NOT error (no protocol checking on needs yet)

---

### Task 8: Implement protocol-typed needs validation

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — add protocol conformance check in `.new()`

**Step 1: Add protocol check in class .new()**

In the `(Value::Class(class_id), "new")` handler, after inserting each field, check if the type annotation refers to a known protocol. If so, verify the value conforms:

```rust
// After inserting the field value:
if let Some(type_name) = type_ann {
    // Check if type_name refers to a protocol
    if let Some(Value::Protocol(proto_id)) = self.env.get(type_name).cloned() {
        // Check if the value conforms
        let conforms = match &val {
            Value::Instance(iid) => {
                let inst_class = &self.classes[self.instances[iid.0].class_id.0];
                let proto = &self.protocols[proto_id.0];
                proto.required_methods.iter().all(|req| {
                    inst_class.methods.iter().any(|m| m.name == *req)
                })
            }
            _ => false,
        };
        if !conforms {
            return Err(EvalError::RuntimeError(format!(
                "{}.new() — '{}' must implement {}",
                class.name, need_name, type_name
            )));
        }
    }
}
```

**Step 2: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `needs_protocol.opl`

**Step 3: Commit**

```
feat: validate protocol conformance on needs at construction
```

---

### Task 9: Spec — Container

**Files:**
- Create: `tests/spec/06-patterns/container.opl`

**Step 1: Write the spec**

```opal
# expect: saved bounty-1 | Error: No registration for Mailer
# DI Container with register/resolve

protocol Storage
  def save(id) -> String
end

class MemoryStore implements Storage
  def save(id)
    f"saved {id}"
  end
end

class BountyRepo
  needs store: Storage

  def persist(id)
    .store.save(id)
  end
end

# Container wires dependencies
container = Container.new()
container.register(Storage, MemoryStore.new())

# Resolve fills needs automatically
repo = container.resolve(BountyRepo)
print(repo.persist("bounty-1"))

# Missing registration errors
try
  container.resolve_name("Mailer")
catch as e
  print(f"Error: {e.message}")
end
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/container.opl 2>&1`
Expected: error (Container class doesn't exist)

---

### Task 10: Implement Container

**Files:**
- Modify: `crates/opal-interp/src/eval.rs` — add Container as a built-in class

**Step 1: Implement Container as a native object**

Container needs three operations:
- `.new()` — creates an empty container (a dict mapping protocol names to values)
- `.register(Protocol, instance)` — stores the mapping
- `.resolve(Class)` — creates an instance of Class, filling `needs` from registrations

Implement as a `NativeObject` or as a special-cased class. The simplest approach: use a dedicated `Value::Container` variant (or reuse `Value::Dict` internally).

Add a `Container` entry to the environment at startup. When `.new()` is called, create an instance with an internal dict. When `.register()` is called, store the protocol→instance mapping. When `.resolve()` is called, look up each `needs` type annotation in the registry and construct.

For `resolve_name(name_string)`, do a string-based lookup for testing missing registrations.

**Step 2: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `container.opl`

**Step 3: Commit**

```
feat: add Container class for dependency injection
```

---

### Task 11: Spec — domain events

**Files:**
- Create: `tests/spec/06-patterns/domain_events.opl`

**Step 1: Write the spec**

```opal
# expect: claimed: bounty-1 by alice | log: bounty-1 claimed
# Domain events with event/emit/on

event BountyClaimed(bounty_id: String, claimer: String)

results = []

on BountyClaimed do |e|
  results.push(f"claimed: {e.bounty_id} by {e.claimer}")
end

on BountyClaimed do |e|
  results.push(f"log: {e.bounty_id} claimed")
end

emit BountyClaimed.new(bounty_id: "bounty-1", claimer: "alice")

print(results.join(" | "))
```

**Step 2: Run spec to verify it fails**

Run: `cargo run -- tests/spec/06-patterns/domain_events.opl 2>&1`
Expected: parse error (`event` keyword not recognized)

---

### Task 12: Implement domain events

**Files:**
- Modify: `crates/opal-lexer/src/token.rs` — add `Event`, `Emit`, `On` tokens
- Modify: `crates/opal-parser/src/ast.rs` — add `EventDef`, `EmitStmt`, `OnHandler` variants
- Modify: `crates/opal-parser/src/parser.rs` — parse `event`, `emit`, `on` statements
- Modify: `crates/opal-interp/src/eval.rs` — evaluate events

**Step 1: Add tokens**

In `token.rs`, add to the Token enum:

```rust
#[token("event")]
Event,
#[token("emit")]
Emit,
#[token("on")]
On,
```

Note: Check for conflicts — `on` might conflict with identifier parsing. If it's already a keyword or identifier pattern, it needs to be a contextual keyword or handled carefully.

**Step 2: Add AST nodes**

In `ast.rs`:

```rust
/// event Name(field: Type, ...)
EventDef { name: String, fields: Vec<NeedsDecl> },
/// emit expr
EmitStmt(Expr),
/// on EventType do |e| ... end
OnHandler { event_name: String, param: String, body: Vec<Stmt> },
```

**Step 3: Parse event declarations**

`event` parses like a model — name followed by parenthesized field list. Under the hood, it creates an immutable class (like `model`).

**Step 4: Parse emit**

`emit expr` evaluates the expression and dispatches to all registered `on` handlers.

**Step 5: Parse on handlers**

`on EventName do |param| ... end` registers a closure that runs when that event type is emitted.

**Step 6: Implement in evaluator**

- `EventDef`: Create a class (reuse `StoredClass` with the fields as needs). Mark it as an event class.
- `OnHandler`: Store in a `Vec<(String, ClosureId)>` registry mapping event name to handler closures.
- `EmitStmt`: Evaluate the expression, look up its class name, find all registered handlers, call each with the event instance.

Since Opal's interpreter is synchronous, `emit` calls handlers synchronously (sequentially). This matches the current actor model (also synchronous). The `await` variant can be a no-op since execution is already synchronous.

**Step 7: Run spec and all tests**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1 | tail -5`
Expected: all pass including `domain_events.opl`

**Step 8: Commit**

```
feat: add event/emit/on for domain events
```

---

### Task 13: Final verification

**Step 1: Run full test suite**

Run: `cargo test 2>&1 | tail -5`
Run: `bash tests/run_spec.sh 2>&1`
Expected: all unit tests pass, all specs pass (6 new + 31 existing)

**Step 2: Commit any remaining fixes**

If any tests broke, fix and commit.
