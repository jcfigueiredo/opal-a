# Dependency Injection & Domain Events

Opal provides first-class dependency injection through the `needs` keyword and a domain event system built on actors. These two features are tightly coupled: event handlers in modules use `needs` to access their dependencies, and the optional `Container` wires both together.

---

## Dependency Injection with `needs`

`needs` declares a dependency with a name and a protocol/type. Dependencies become instance variables (`.name`) and must be provided at construction time via `.new()`.

### On Classes

```opal
protocol Database
  def save(record) -> Bool
  def find(id: Int32) -> Record?
end

protocol Mailer
  def send_confirmation(order: Order)
end

class OrderService
  needs db: Database
  needs mailer: Mailer

  def place_order(order)
    .db.save(order)
    .mailer.send_confirmation(order)
  end
end

# Explicit wiring — you see exactly what connects to what
service = OrderService.new(
  db: PostgresDB.new(),
  mailer: SMTPMailer.new()
)

# Testing — swap implementations
test_service = OrderService.new(
  db: MockDB.new(),
  mailer: MockMailer.new()
)
```

### On Modules

```opal
module Billing
  needs payments: PaymentGateway

  def charge(order)
    .payments.charge(order.total)
  end
end
```

### On Actors

```opal
actor PaymentProcessor
  receives :charge
  needs gateway: PaymentGateway

  receive
    case :charge(order)
      .gateway.charge(order.total)
      reply :ok
  end
end
```

### Rules

- `needs name: Protocol` declares a required dependency.
- `needs name: Protocol = default_expr` declares an optional dependency with a default.
- Dependencies are checked at construction -- missing a required `needs` is a runtime error.
- `needs` dependencies are accessible as `.name` (same as instance variables).
- `needs` works on classes, modules, and actors.
- If the class also has `init`, `needs` deps are injected *before* `init` runs.

---

## Optional Container

For small apps, manual wiring with `.new()` is sufficient. For large apps, the `Container` class from the standard library resolves dependencies by protocol.

```opal
import Container

app = Container.new()
app.register(Database, PostgresDB.new())
app.register(Mailer, SMTPMailer.new())
app.register(WarehouseService, LocalWarehouse.new())

# Resolve a class — container fills in all `needs` automatically
service = app.resolve(OrderService)

# Resolve modules — handlers are auto-registered with deps
app.resolve(NotificationHandler)
app.resolve(InventoryHandler)

app.start!
```

```opal
# Testing with container — swap just what you need
test_app = Container.new()
test_app.register(Database, MockDB.new())
test_app.register(Mailer, MockMailer.new())
test_service = test_app.resolve(OrderService)
```

### Rules

- `Container` is a standard library class, not a language keyword.
- `register(Protocol, implementation)` maps a protocol to a concrete instance.
- `resolve(Class)` creates an instance with all `needs` satisfied from the container.
- Missing registration is a runtime error with a clear message.
- Container is optional -- you can always wire manually with `.new()`.

---

## Domain Events

Events are declared as named, immutable data structures. They are emitted with `emit` and handled with `on`. Under the hood, events are dispatched through an actor-based event bus -- handlers get supervision and fault tolerance for free.

### Declaring Events

```opal
event OrderPlaced(order: Order, placed_at: Time)
event OrderShipped(order: Order, tracking: String)
event PaymentFailed(order: Order, reason: String)
```

Events are immutable data -- handlers cannot modify the event.

### Emitting Events

```opal
class OrderService
  needs db: Database

  def place_order(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end
```

### Handling Events

```opal
module NotificationHandler
  needs mailer: Mailer

  on OrderPlaced do |e|
    .mailer.send_confirmation(e.order)
  end

  on OrderShipped do |e|
    .mailer.send_tracking(e.order, e.tracking)
  end

  on PaymentFailed do |e|
    .mailer.send_payment_alert(e.order, e.reason)
  end
end

module InventoryHandler
  needs warehouse: WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end
```

### Events Compose with Existing Features

```opal
# With pattern matching
module AnalyticsHandler
  needs tracker: Analytics

  on OrderPlaced do |e|
    match e.order.total
      case amount if amount > 1000
        .tracker.flag_high_value(e.order)
      case _
        .tracker.record(e.order)
    end
  end
end

# With preconditions
def business_hours?() -> Bool
  hour = Time.now().hour
  hour >= 9 and hour < 17
end

on OrderPlaced do |e|
  if business_hours?()
    notify_sales_team(e.order)
  end
end
```

### Rules

- `event Name(fields...)` declares an event type (immutable data).
- `emit event_instance` dispatches the event to all registered `on` handlers.
- `on EventType do |e| ... end` registers a handler.
- Handlers run **asynchronously** by default (fire-and-forget from the emitter).
- Multiple handlers for the same event run **concurrently** (via actors underneath).
- Handlers in modules have access to the module's `needs` dependencies.
- Events are immutable -- handlers cannot modify the event data.

---

## Emit and Async Interaction

`emit` follows Opal's concurrency model with one key design choice: `emit` is async by default because events semantically represent "something that already happened."

### Async (Default)

```opal
def place_order(order)
  .db.save(order)
  emit OrderPlaced.new(order: order)  # returns immediately
  # handlers run concurrently in the background
end
```

### Sync with `await`

When you need all handlers to finish before continuing:

```opal
def place_order(order)
  .db.save(order)
  emit OrderPlaced.new(order: order) await  # blocks until ALL handlers finish
  print("All side effects complete")
end
```

### Background Sync with `async`

```opal
def place_order(order)
  .db.save(order)
  delivery = async emit OrderPlaced.new(order: order) await
  # delivery is a Future — emit+await runs in background

  do_other_work()

  try
    await delivery  # did all handlers complete ok?
  catch as e
    log(f"Event handling failed: {e.message}")
  end
end
```

### Inside Parallel Blocks

```opal
def process_batch(orders)
  parallel for order in orders
    .db.save(order)
    emit OrderPlaced.new(order: order)
  end
end
```

### Inside Actors

```opal
actor OrderProcessor
  receives :process

  needs db: Database

  receive
    case :process(order)
      .db.save(order)
      emit OrderPlaced.new(order: order)  # works from inside actors
      reply :ok
  end
end
```

### Emit Patterns Summary

| Pattern | Behavior |
|---|---|
| `emit Event.new(...)` | Async -- fire and forget, returns immediately |
| `emit Event.new(...) await` | Sync -- blocks until all handlers complete |
| `async emit Event.new(...) await` | Background sync -- all handlers run, returns Future |
| `emit` inside `parallel` | Each branch emits independently |
| `emit` inside actor `receive` | Works normally, handlers run outside the actor |

---

## Complete DDD Example

This example ties together dependency injection, domain events, actors, and the container in a realistic domain-driven design scenario.

```opal
import Container
import Time

# --- Domain Events ---
event OrderPlaced(order: Order, placed_at: Time)
event PaymentFailed(order: Order, reason: String)

# --- Domain Service (with DI) ---
class OrderService
  needs db: Database
  needs validator: OrderValidator

  def place_order(order)
    .validator.validate!(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end

# --- Event Handlers (with DI) ---
module NotificationHandler
  needs mailer: Mailer

  on OrderPlaced do |e|
    .mailer.send_confirmation(e.order)
  end

  on PaymentFailed do |e|
    .mailer.send_payment_alert(e.order, e.reason)
  end
end

module InventoryHandler
  needs warehouse: WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end

# --- Actor for stateful concurrent work ---
actor PaymentProcessor
  receives :charge
  needs gateway: PaymentGateway

  receive
    case :charge(order)
      try
        .gateway.charge(order.total)
        reply :ok
      catch as e
        emit PaymentFailed.new(order: order, reason: e.message)
        reply :failed
      end
  end
end

# --- App Wiring ---
app = Container.new()
app.register(Database, PostgresDB.new())
app.register(Mailer, SMTPMailer.new())
app.register(OrderValidator, StrictValidator.new())
app.register(WarehouseService, LocalWarehouse.new())
app.register(PaymentGateway, StripeGateway.new())

order_service = app.resolve(OrderService)
app.resolve(NotificationHandler)
app.resolve(InventoryHandler)
payment = app.resolve(PaymentProcessor)

supervisor AppSupervisor
  strategy :one_for_one
  supervise payment
end

AppSupervisor.start!

# --- Use it ---
order_service.place_order(new_order)
# 1. Validates order        (via injected validator)
# 2. Saves to DB            (via injected db)
# 3. Emits OrderPlaced
# 4. Sends email            (async, via NotificationHandler)
# 5. Reserves stock         (async, via InventoryHandler)
```

---

## Design Rationale

### Why `needs` as a Keyword

- **Explicitness.** Dependencies are declared at the top of a class/module/actor, not hidden in constructor bodies. You can read the dependency list at a glance.
- **Testability.** Every `needs` dependency can be swapped at construction time with `.new()`, making unit testing trivial without mocking frameworks.
- **No magic by default.** Small apps wire manually. The `Container` is opt-in for large apps that benefit from automatic resolution.

### Why Events are Sugar over Actors

- **Consistency.** Events dispatch through Opal's actor infrastructure, so they automatically get supervision, fault tolerance, and the same concurrency guarantees.
- **Connected through modules.** Modules declare both their dependencies (`needs`) and their event handlers (`on`). Injected deps are available inside handlers.
- **Async-first.** Events represent things that already happened, so fire-and-forget is the natural default. `await` and `async` compose when you need synchronous guarantees.

### New Keywords

| Keyword | Purpose | Context |
|---|---|---|
| `needs` | Declare a dependency | Classes, modules, actors |
| `event` | Declare an event type | Top-level |
| `emit` | Dispatch an event | Anywhere |
| `on` | Register an event handler | Modules, top-level |
| `await` (after emit) | Wait for all handlers to complete | After `emit` |

---

## Summary

Opal's dependency injection and event system follows two principles: dependencies are declared, not hidden; and events are async by default because they represent things that already happened. The `needs` keyword makes dependencies visible at the declaration site. The `event`/`emit`/`on` system provides domain events that run on the actor infrastructure. The optional `Container` ties everything together for large applications. Both systems compose naturally with pattern matching, preconditions, actors, and the concurrency model.
