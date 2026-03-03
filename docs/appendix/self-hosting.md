# Self-Hosting: Opal in Opal

---

## Overview

Opal's parser handles a minimal core: definitions, control flow, metaprogramming primitives, and operators. Most language sugar -- `needs`, `event`, `emit`, `on`, `requires`, `supervisor` -- is self-hosted as macros. This appendix shows exactly how each feature desugars, with three panels per feature: the sugar you write, the code it expands to, and the macro that performs the transformation.

---

## Parser Core vs Macro Layer

```
┌─────────────────────────────────────┐
│         Parser Core (fixed)         │
│  def, class, module, actor, enum    │
│  if, for, while, match, try        │
│  ast, macro, $, @, @[...]          │
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

The parser core is roughly 20 keywords -- enough to define types, control flow, and macros. Everything in the macro layer is defined using those primitives. This split gives three properties:

1. **Inspectable.** `macroexpand()` shows what any piece of sugar generates. No hidden compiler magic.
2. **Customizable.** If the built-in `needs` doesn't fit your use case, write your own version. Macros are just functions on AST.
3. **Growable.** New features ship as packages of macros, not compiler patches. The language grows without touching the parser.

---

## 1. `needs` -- Dependency Injection

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

Multiple `needs` declarations merge into a single `init()`. Default values become default parameters. Each dependency gets a typed getter method.

### The Macro

```opal
macro needs(body)
  # Collect all needs declarations from the class body
  declarations = body.select(|node| node.head == :needs)

  # Build init parameters
  params = declarations.map do |decl|
    if decl.default
      ast($decl.name: $decl.type = $decl.default)
    else
      ast($decl.name: $decl.type)
    end
  end

  # Build assignments
  assignments = declarations.map do |decl|
    name = decl.name
    ast(.$name = $name)
  end

  # Build getter methods
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
end
```

The macro operates on the class body at parse time. It extracts `needs` declarations, generates a single `init()` with all parameters, creates instance variable assignments, and adds typed getter methods. The `$list...` splat inserts each item from a list into the generated code.

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

## 2. `event` -- Domain Events

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

That's it -- `event` is syntactic sugar for `model`. Both produce immutable data classes with named fields, automatic equality, and string representation. The distinction is semantic: `event` signals intent ("this is a thing that happened").

### The Macro

```opal
macro event(name, body)
  ast
    model $name
      $body
    end
  end
end
```

The simplest macro in the set. It literally rewrites `event` as `model`. This is a powerful pattern -- sometimes macros exist purely for **readability and intent**, not for complex code generation.

---

## 3. `emit` -- Event Dispatch

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
  ast
    EventBus.dispatch($event_expr)
  end
end
```

`emit` wraps any expression in an `EventBus.dispatch()` call. The `EventBus` is a supervised actor (started by the application supervisor) that routes events to registered handlers. Fire-and-forget by default -- the sender doesn't wait for handlers to complete.

For synchronous dispatch (wait for all handlers):

```opal
await emit OrderPlaced.new(order: o, placed_at: Time.now())
# expands to:
await EventBus.dispatch(OrderPlaced.new(order: o, placed_at: Time.now()))
```

`await` is parser-level -- it works on any expression that returns a future, including the dispatch call.

---

## 4. `on` -- Event Handlers

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
  ast
    EventBus.register($event_type, $handler)
  end
end
```

`on` registers a handler closure with the `EventBus`, filtered by event type. When an event of that type is dispatched, the bus calls every registered handler. The handler receives the event as its parameter.

Multiple handlers for the same event type are supported -- they all fire:

```opal
on OrderPlaced do |e| send_confirmation(e.order) end
on OrderPlaced do |e| update_inventory(e.order) end
on OrderPlaced do |e| record_analytics(e.order) end
```

---

## 5. `requires` -- Preconditions

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
  ast
    if !($condition)
      raise PreconditionError.new(message: $message)
    end
  end
end
```

`requires` inserts a guard clause at the expansion site. The condition is evaluated at runtime -- if it fails, a `PreconditionError` is raised with the given message. The macro is intentionally simple: it doesn't need to understand the condition, just negate it and wrap it in an if/raise.

The message parameter is optional. Without it, the macro uses the condition's source code as the message:

```opal
macro requires(condition)
  source = condition.to_string()
  ast
    if !($condition)
      raise PreconditionError.new(message: $source)
    end
  end
end
```

---

## 6. `supervisor` -- Fault Tolerance

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

A 6-line declaration expands into a complete actor with initialization, message handling, child lifecycle management, restart strategies, and a circuit breaker. This is the most complex macro in the set.

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
    ast(start_child($type, $args...))
  end

  ast
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

The most complex macro in the set. It:

1. **Parses configuration** -- extracts `strategy` and `max_restarts` declarations from the body AST.
2. **Collects children** -- gathers `supervise` entries with their constructor arguments.
3. **Generates a full actor** -- produces init, receive handlers, and restart logic.
4. **Preserves strategy semantics** -- the restart strategy (`:one_for_one`, `:all_for_one`, `:rest_for_one`) determines behavior in `handle_child_exit`.

This is a macro that generates an entire actor with complete supervision behavior -- tens of lines of boilerplate from a few declarative lines.

---

## Implications for Language Evolution

The self-hosting model creates a natural pipeline for language evolution:

1. **Prototype.** A new feature starts as a macro in a package. Anyone can publish one.
2. **Battle-test.** Real users exercise it in production. Rough edges get filed down through iteration.
3. **Promote.** Only features that prove essential *and* performance-critical move to parser-level keywords. Most never need to.

For example, suppose the community builds a popular `@pipeline` macro for data transformation:

```opal
macro pipeline(steps)
  steps.reduce do |acc, step|
    ast($step($acc))
  end
end

result = @pipeline [data, parse, validate, transform, save]
# expands to: save(transform(validate(parse(data))))
```

This doesn't need a language change -- it ships as a package. If it proves essential enough after years of use, it *could* later become a parser keyword. But it doesn't have to. The macro version works today.

This keeps Opal's core small (~20 keywords) while the ecosystem grows without bounds.

---

## Summary

| Feature | Complexity | What the Macro Does |
|---|---|---|
| `event` | Trivial | Rewrites as `model` |
| `emit` | Trivial | Wraps in `EventBus.dispatch()` |
| `on` | Trivial | Wraps in `EventBus.register()` |
| `requires` | Simple | Generates if/raise guard clause |
| `needs` | Medium | Collects declarations, generates init + getters |
| `supervisor` | Complex | Generates full actor with restart logic |

The complexity gradient is the point. Trivial macros like `event` prove that macros are useful even when the transformation is minimal -- the value is in naming and intent. Complex macros like `supervisor` prove that the macro system is powerful enough to generate entire subsystems. Everything in between confirms that Opal's `ast`/`$`/`macro` primitives scale smoothly from one-liners to full code generators.
