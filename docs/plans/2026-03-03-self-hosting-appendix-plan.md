# Self-Hosting Appendix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create an appendix document showing how 6 pieces of Opal's language sugar (`needs`, `event`, `emit`, `on`, `requires`, `supervisor`) are self-hosted as macros, with three panels each: sugar, expansion, and macro source.

**Architecture:** One new document (`docs/appendix/self-hosting.md`) with an opening framing section, 6 feature sections in three-panel format, and a closing section on language evolution. Three existing files get small updates: Opal.md (appendix link), metaprogramming.md (cross-reference), CLAUDE.md (directory description).

**Tech Stack:** Markdown files only. No code, no tests.

---

## Context

**Design doc:** `docs/plans/2026-03-03-self-hosting-appendix-design.md`

**Key references:**
- `docs/07-metaprogramming/metaprogramming.md` — existing metaprogramming spec (quote, macros, annotations, hygiene, subdomains). Lines 447-471 list the 6 features as "could be macros" but don't show implementations.
- `docs/06-patterns/dependency-injection.md` — how `needs` works from the user's perspective
- `docs/05-concurrency/concurrency.md` — how `supervisor` and actors work
- `docs/04-error-handling/preconditions.md` — how `requires` works

**Opal syntax conventions:**
- `: ` for type annotations, `[T]` for generics, `from X import Y`
- `do |params| ... end` for multi-line closures, `|params| expr` for inline
- `.name` for instance variables, `needs` for DI
- `quote...end` for AST capture, `$expr` for interpolation, `$list...` for splats
- `macro name(params) ... end` for macro definitions, `@name` for invocation
- `esc(expr)` to break hygiene

---

### Task 1: Create `docs/appendix/self-hosting.md`

**Files:**
- Create: `docs/appendix/self-hosting.md`
- Reference: `docs/plans/2026-03-03-self-hosting-appendix-design.md`
- Reference: `docs/07-metaprogramming/metaprogramming.md` (macro syntax patterns, lines 78-162)
- Reference: `docs/06-patterns/dependency-injection.md` (how `needs` works)
- Reference: `docs/05-concurrency/concurrency.md` (supervisors, actors, events)
- Reference: `docs/04-error-handling/preconditions.md` (how `requires` works)

**What to write:**

The document should have this structure:

```markdown
# Self-Hosting: Opal in Opal

---

## Overview

2-3 sentences: Opal's parser handles a minimal core. Most language sugar is self-hosted as macros. This appendix shows exactly how.

---

## Parser Core vs Macro Layer

The ASCII diagram from the design doc:

```
┌─────────────────────────────────────┐
│         Parser Core (fixed)         │
│  def, class, module, actor, enum    │
│  if, for, while, match, try        │
│  quote, macro, $, @, @[...]        │
│  =, ., :, operators                │
└─────────────────────────────────────┘
              ▲ builds on
┌─────────────────────────────────────┐
│       Macro Layer (self-hosted)     │
│  needs, event, emit, on            │
│  requires, supervisor              │
│  @test, @describe, @get, @post     │
│  @memoize, @json_serializable      │
└─────────────────────────────────────┘
```

Then 3 properties:
1. Inspectable — `macroexpand()` shows what sugar generates
2. Customizable — write your own version of any sugar
3. Growable — new features ship as packages, not compiler patches

---

## 1. `needs` — Dependency Injection

### The Sugar

```opal
class OrderService
  needs db: Database
  needs mailer: Mailer
  needs logger: Logger = Logger.default()
end
```

### What It Expands To

```opal
class OrderService
  def init(db: Database, mailer: Mailer, logger: Logger = Logger.default())
    .db = db
    .mailer = mailer
    .logger = logger
  end

  def db() -> Database = .db
  def mailer() -> Mailer = .mailer
  def logger() -> Logger = .logger
end
```

Key: Multiple `needs` merge into one `init()`. Default values become default parameters.

### The Macro

```opal
macro needs(body)
  # Collect all needs declarations from the class body
  declarations = body.select(|node| node.head == :needs)

  # Build init parameters
  params = declarations.map do |decl|
    if decl.default
      quote $decl.name: $decl.type = $decl.default end
    else
      quote $decl.name: $decl.type end
    end
  end

  # Build assignments
  assignments = declarations.map do |decl|
    name = decl.name
    quote .$name = $name end
  end

  # Build getter methods
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
end
```

Explain: The macro operates on the class body at parse time. It extracts `needs` declarations, generates a single `init()` with all parameters, creates instance variable assignments, and adds typed getter methods. The `$list...` splat inserts each item from a list.

### Verify with `macroexpand`

```opal
macroexpand do
  class OrderService
    needs db: Database
  end
end
# => Expr: class OrderService
#      def init(db: Database)
#        .db = db
#      end
#      def db() -> Database = .db
#    end
```

---

## 2. `event` — Domain Events

### The Sugar

```opal
event OrderPlaced
  needs order: Order
  needs placed_at: Time
end
```

### What It Expands To

```opal
model OrderPlaced
  needs order: Order
  needs placed_at: Time
end
```

That's it — `event` is syntactic sugar for `model`. Both produce immutable data classes with named fields, automatic equality, and string representation. The distinction is semantic: `event` signals intent ("this is a thing that happened").

### The Macro

```opal
macro event(name, body)
  quote
    model $name
      $body
    end
  end
end
```

Explain: The simplest macro in the set. It literally rewrites `event` as `model`. This is a powerful pattern — sometimes macros exist purely for **readability and intent**, not for complex code generation.

---

## 3. `emit` — Event Dispatch

### The Sugar

```opal
emit OrderPlaced.new(order: current_order, placed_at: Time.now())
```

### What It Expands To

```opal
EventBus.dispatch(OrderPlaced.new(order: current_order, placed_at: Time.now()))
```

### The Macro

```opal
macro emit(event_expr)
  quote
    EventBus.dispatch($event_expr)
  end
end
```

Explain: `emit` wraps any expression in an `EventBus.dispatch()` call. The `EventBus` is a supervised actor (started by the application supervisor) that routes events to registered handlers. Fire-and-forget by default — the sender doesn't wait for handlers to complete.

For synchronous dispatch (wait for all handlers):

```opal
await emit OrderPlaced.new(order: o, placed_at: Time.now())
# expands to:
await EventBus.dispatch(OrderPlaced.new(order: o, placed_at: Time.now()))
```

`await` is parser-level — it works on any expression that returns a future, including the dispatch call.

---

## 4. `on` — Event Handlers

### The Sugar

```opal
on OrderPlaced do |event|
  Mailer.send_confirmation(event.order)
  Logger.info(f"Order placed: {event.order.id}")
end
```

### What It Expands To

```opal
EventBus.register(OrderPlaced) do |event|
  Mailer.send_confirmation(event.order)
  Logger.info(f"Order placed: {event.order.id}")
end
```

### The Macro

```opal
macro on(event_type, handler)
  quote
    EventBus.register($event_type, $handler)
  end
end
```

Explain: `on` registers a handler closure with the `EventBus`, filtered by event type. When an event of that type is dispatched, the bus calls every registered handler. The handler receives the event as its parameter.

Multiple handlers for the same event type are supported — they all fire:

```opal
on OrderPlaced do |e| send_confirmation(e.order) end
on OrderPlaced do |e| update_inventory(e.order) end
on OrderPlaced do |e| record_analytics(e.order) end
```

---

## 5. `requires` — Preconditions

### The Sugar

```opal
def withdraw(amount: Float64) -> Float64
  requires amount > 0, "amount must be positive"
  requires amount <= .balance, "insufficient funds"

  .balance -= amount
  .balance
end
```

### What It Expands To

```opal
def withdraw(amount: Float64) -> Float64
  if !(amount > 0)
    raise PreconditionError.new(message: "amount must be positive")
  end
  if !(amount <= .balance)
    raise PreconditionError.new(message: "insufficient funds")
  end

  .balance -= amount
  .balance
end
```

### The Macro

```opal
macro requires(condition, message)
  quote
    if !($condition)
      raise PreconditionError.new(message: $message)
    end
  end
end
```

Explain: `requires` inserts a guard clause at the expansion site. The condition is evaluated at runtime — if it fails, a `PreconditionError` is raised with the given message. The macro is intentionally simple: it doesn't need to understand the condition, just negate it and wrap it in an if/raise.

The message parameter is optional. Without it, the macro uses the condition's source code as the message:

```opal
macro requires(condition)
  source = condition.to_string()
  quote
    if !($condition)
      raise PreconditionError.new(message: $source)
    end
  end
end
```

---

## 6. `supervisor` — Fault Tolerance

### The Sugar

```opal
supervisor AppSupervisor
  strategy :one_for_one
  max_restarts 3, 60

  supervise Logger()
  supervise Cache(ttl: 60)
  supervise Worker()
end
```

### What It Expands To

```opal
actor AppSupervisor
  receives :child_exited, :status

  def init()
    .strategy = :one_for_one
    .max_restarts = 3
    .restart_window = 60
    .restart_count = 0
    .window_start = Time.now()

    .children = [
      start_child(Logger),
      start_child(Cache, ttl: 60),
      start_child(Worker)
    ]
  end

  receive
    case :child_exited(child, reason)
      handle_child_exit(child, reason)
    case :status
      reply .children.map(|c| {name: c.name, alive: c.alive?()})
  end

  private def start_child(type, **args) -> ActorRef
    child = type.new(**args)
    child.link(self)
    child
  end

  private def handle_child_exit(child, reason)
    reset_window_if_expired()
    .restart_count += 1

    if .restart_count > .max_restarts
      raise SupervisorError.new(
        message: f"Max restarts ({.max_restarts}) exceeded in {.restart_window}s"
      )
    end

    match .strategy
      case :one_for_one
        restart_child(child)
      case :all_for_one
        restart_all_children()
      case :rest_for_one
        restart_from(child)
    end
  end

  private def restart_child(child)
    index = .children.index(child)
    .children[index] = start_child(child.type, **child.args)
  end

  private def restart_all_children()
    .children = .children.map(|c| start_child(c.type, **c.args))
  end

  private def restart_from(child)
    index = .children.index(child)
    index.upto(.children.length() - 1) do |i|
      .children[i] = start_child(.children[i].type, **.children[i].args)
    end
  end

  private def reset_window_if_expired()
    elapsed = Time.now() - .window_start
    if elapsed > .restart_window
      .restart_count = 0
      .window_start = Time.now()
    end
  end
end
```

### The Macro

```opal
macro supervisor(name, body)
  # Parse declarations from the body
  strat = body.find(|n| n.head == :strategy).args[0]
  restarts = body.find(|n| n.head == :max_restarts)
  max = restarts.args[0]
  window = restarts.args[1]

  # Collect supervised children
  children = body.select(|n| n.head == :supervise).map do |node|
    type = node.args[0]
    args = node.args[1..]
    quote start_child($type, $args...) end
  end

  quote
    actor $name
      receives :child_exited, :status

      def init()
        .strategy = $strat
        .max_restarts = $max
        .restart_window = $window
        .restart_count = 0
        .window_start = Time.now()
        .children = [$children...]
      end

      receive
        case :child_exited(child, reason)
          handle_child_exit(child, reason)
        case :status
          reply .children.map(|c| {name: c.name, alive: c.alive?()})
      end

      # ... private restart methods (as shown in expansion)
    end
  end
end
```

Explain: The most complex macro. It:
1. Parses configuration declarations (`strategy`, `max_restarts`) from the body
2. Collects `supervise` entries with their arguments
3. Generates a full actor with init, receive handlers, and restart logic
4. The restart strategy (`:one_for_one`, `:all_for_one`, `:rest_for_one`) determines behavior in `handle_child_exit`

This is a macro that generates an entire actor with complete supervision behavior — tens of lines of boilerplate from a few declarative lines.

---

## Implications for Language Evolution

Closing section explaining:

1. **Prototype** — new features start as macro packages
2. **Battle-test** — real users exercise them in production
3. **Promote** — only essential, performance-critical features move to parser-level

Example: if the community builds a popular `@pipeline` macro for data transformation pipelines, it doesn't need a language change — it ships as a package. If it proves essential enough, it could later become a parser keyword.

This keeps Opal's core small (~20 keywords) while the ecosystem grows without bounds.

---

## Summary

Table of all 6 features with their complexity level:

| Feature | Complexity | What the Macro Does |
|---|---|---|
| `event` | Trivial | Rewrites as `model` |
| `emit` | Trivial | Wraps in `EventBus.dispatch()` |
| `on` | Trivial | Wraps in `EventBus.register()` |
| `requires` | Simple | Generates if/raise guard clause |
| `needs` | Medium | Collects declarations, generates init + getters |
| `supervisor` | Complex | Generates full actor with restart logic |

```

**Commit:** `"Add self-hosting appendix: Opal in Opal"`

---

### Task 2: Update cross-references

**Files:**
- Modify: `Opal.md` (appendix section, line 725-729)
- Modify: `docs/07-metaprogramming/metaprogramming.md` (self-hosting section, lines 447-471)
- Modify: `CLAUDE.md` (appendix directory description)

**What to change:**

**Change 1 — Opal.md appendix section (line 725-729):**

Replace:
```markdown
## Appendix

Links, references, tutorials, and implementation ideas for building the Opal runtime.

> See [Appendix](docs/appendix/appendix.md) for all reference materials.
```

With:
```markdown
## Appendix

Links, references, tutorials, and implementation ideas for building the Opal runtime.

> See [Appendix](docs/appendix/appendix.md) for reference materials and [Self-Hosting: Opal in Opal](docs/appendix/self-hosting.md) for how language sugar is implemented as macros.
```

**Change 2 — metaprogramming.md self-hosting section (line 470):**

After the line "Whether they stay as keywords or become macros is an implementation decision. The key insight is that the macro system is *expressive enough* to define them.", add:

```markdown

> See [Self-Hosting: Opal in Opal](../appendix/self-hosting.md) for complete macro implementations of all 6 features — sugar, expansion, and macro source code.
```

**Change 3 — CLAUDE.md appendix directory row:**

Find the appendix row in the Documentation Structure table. Change from:
```
| `docs/appendix/` | Links, references, ideas |
```
To:
```
| `docs/appendix/` | Links, references, ideas, self-hosting examples |
```

**Commit:** `"Update cross-references for self-hosting appendix"`

---

## Dependency Graph

```
Task 1 (create self-hosting.md)
  └─> Task 2 (update cross-references)
```

Task 1 must go first (creates the file others link to). Task 2 updates the 3 existing files.
