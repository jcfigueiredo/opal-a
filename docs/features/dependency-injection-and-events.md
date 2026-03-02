# Dependency Injection & Domain Events

---

## Design Principles

- **`needs` as a first-class keyword** — dependencies are declared, not hidden in constructors.
- **Events are sugar over actors** — they look like pub/sub but run on Opal's actor infrastructure, getting supervision and fault tolerance for free.
- **Connected through modules** — modules declare both their dependencies (`needs`) and their event handlers (`on`). Injected deps are available inside handlers.
- **Optional Container** — manual wiring with `.new()` for small apps; `Container` from stdlib for large apps. No magic unless you opt in.

---

## 1. Dependency Injection (`needs`)

`needs` declares a dependency with a name and a protocol/type. Dependencies become instance variables (`.name`) and must be provided at construction time via `.new()`.

### On Classes

```opal
protocol Database
  def save(record) -> Bool
  def find(id::Int32) -> Record?
end

protocol Mailer
  def send_confirmation(order::Order)
end

class OrderService
  needs db::Database
  needs mailer::Mailer

  def place_order(order)
    .db.save(order)
    .mailer.send_confirmation(order)
  end
end

# Explicit wiring
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
  needs payments::PaymentGateway

  on OrderPlaced do |e|
    .payments.charge(e.order)
  end
end
```

### On Actors

```opal
actor PaymentProcessor
  needs gateway::PaymentGateway

  receive :charge(order)
    .gateway.charge(order.total)
    reply :ok
  end
end
```

### Rules

- `needs name::Protocol` declares a required dependency.
- `needs name::Protocol = default_expr` declares an optional dependency with a default.
- Dependencies are checked at construction — missing a required `needs` is a runtime error.
- `needs` dependencies are accessible as `.name` (same as instance variables).
- `needs` works on classes, modules, and actors.
- If the class also has `:init`, `needs` deps are injected *before* `:init` runs.

---

## 2. Domain Events (`event`, `emit`, `on`)

Events are declared as named data structures. They're emitted with `emit` and handled with `on`. Under the hood, events are dispatched through an actor-based event bus.

### Declaring Events

```opal
event OrderPlaced(order::Order, placed_at::Time)
event OrderShipped(order::Order, tracking::String)
event PaymentFailed(order::Order, reason::String)
```

Events are immutable data — handlers can't modify the event.

### Emitting Events

```opal
class OrderService
  needs db::Database

  def place_order(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end
```

### Handling Events

```opal
module NotificationHandler
  needs mailer::Mailer

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
  needs warehouse::WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end
```

### Events Compose with Existing Features

```opal
# With pattern matching
module AnalyticsHandler
  needs tracker::Analytics

  on OrderPlaced do |e|
    match e.order.total
      case amount if amount > 1000
        .tracker.flag_high_value(e.order)
      case _
        .tracker.record(e.order)
    end
  end
end

# With guards
@only_business_hours
on OrderPlaced do |e|
  notify_sales_team(e.order)
end
```

### Rules

- `event Name(fields...)` declares an event type (lightweight immutable data structure).
- `emit event_instance` dispatches the event to all registered `on` handlers.
- `on EventType do |e| ... end` registers a handler.
- Handlers run **asynchronously** by default (fire-and-forget from the emitter's perspective).
- Multiple handlers for the same event run **concurrently** (parallel, via actors underneath).
- Handlers in modules have access to the module's `needs` dependencies.
- Events are immutable — handlers can't modify the event data.

---

## 3. Emit and Async Interaction

`emit` follows Opal's concurrency model with one exception: `emit` is async by default because events semantically represent "something that already happened."

### Async (default)

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
  on fail as e
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
  needs db::Database

  receive :process(order)
    .db.save(order)
    emit OrderPlaced.new(order: order)  # works from inside actors
    reply :ok
  end
end
```

### Summary

| Pattern | Behavior |
|---|---|
| `emit Event.new(...)` | Async — fire and forget, returns immediately |
| `emit Event.new(...) await` | Sync — blocks until all handlers complete |
| `async emit Event.new(...) await` | Background sync — all handlers run, returns Future |
| `emit` inside `parallel` | Each branch emits independently |
| `emit` inside actor `receive` | Works normally, handlers run outside the actor |

---

## 4. Optional Container (stdlib)

For large applications, the `Container` class resolves dependencies by protocol.

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
# Testing
test_app = Container.new()
test_app.register(Database, MockDB.new())
test_app.register(Mailer, MockMailer.new())
test_service = test_app.resolve(OrderService)
```

### Rules

- `Container` is a standard library class, not a language keyword.
- `register(Protocol, implementation)` maps a protocol to a concrete instance.
- `resolve(Class)` creates an instance with all `needs` satisfied from the container.
- Missing registration = runtime error with a clear message.
- Container is optional — you can always wire manually with `.new()`.

---

## 5. Complete DDD Example

```opal
import Container
import Time

# --- Domain Events ---
event OrderPlaced(order::Order, placed_at::Time)
event OrderShipped(order::Order, tracking::String)
event PaymentFailed(order::Order, reason::String)

# --- Domain Service (with DI) ---
class OrderService
  needs db::Database
  needs validator::OrderValidator

  def place_order(order)
    .validator.validate!(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end

# --- Event Handlers (with DI) ---
module NotificationHandler
  needs mailer::Mailer

  on OrderPlaced do |e|
    .mailer.send_confirmation(e.order)
  end

  on PaymentFailed do |e|
    .mailer.send_payment_alert(e.order, e.reason)
  end
end

module InventoryHandler
  needs warehouse::WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end

# --- Actor for stateful concurrent work ---
actor PaymentProcessor
  needs gateway::PaymentGateway

  receive :charge(order)
    try
      .gateway.charge(order.total)
      reply :ok
    on fail as e
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
# 1. Validates order (via injected validator)
# 2. Saves to DB (via injected db)
# 3. Emits OrderPlaced
# 4. NotificationHandler sends email (async, via injected mailer)
# 5. InventoryHandler reserves stock (async, via injected warehouse)
```

---

## New Keywords Summary

| Keyword | Purpose | Context |
|---|---|---|
| `needs` | Declare a dependency | Classes, modules, actors |
| `event` | Declare an event type | Top-level |
| `emit` | Dispatch an event | Anywhere |
| `on` | Register an event handler | Modules, top-level |
| `await` (after emit) | Wait for all handlers to complete | After `emit` |
