# Concurrency

Opal's concurrency model has four layers: **actors** for stateful concurrent entities, **parallel blocks** for structured concurrency, **async/futures** for individual non-blocking calls, and **supervisors** for fault tolerance.

**Core principles:**
- **Sync by default** — all calls block and return values. Async is opt-in.
- **No colored functions** — there is no `async def`. Any expression can be made async at the call site with the `async` keyword.
- **Structured concurrency** — the `parallel` block is the primary tool for concurrent work. All concurrent work has a parent scope — no orphaned tasks.
- **One way to do each thing** — actors for stateful concurrency, `parallel` for fan-out, `async` for individual futures, supervisors for fault tolerance.

---

## Actors

Actors are long-lived concurrent entities with isolated state. All external interaction goes through message passing via `receive` blocks and `.send()`. Methods defined with `def` are internal only.

### Rules

- `receive` blocks define the public interface (messages the actor responds to).
- `def` methods are internal — only callable from inside the actor.
- `.send()` sends a message to an actor. **Sync by default:** blocks until the actor replies.
- `reply` sends a value back to the caller.

### Basic Actor

```opal
actor Counter
  receives :increment, :get_count, :reset

  def init()
    .count = 0
  end

  receive
    case :increment
      .count += 1
      reply .count
    case :get_count
      reply .count
    case :reset
      .count = 0
      reply :ok
  end

  # Internal helper — not accessible from outside
  private def validate_count()
    .count >= 0
  end
end

# All interaction through .send() — sync by default
c = Counter.new()
c.send(:increment)     # => 1 (blocks until reply)
c.send(:increment)     # => 2
c.send(:get_count)     # => 2
c.send(:reset)         # => :ok
```

### Messages with Arguments

```opal
actor Cache
  receives :get, :set, :delete

  def init(ttl::Int32)
    .store = {:}
    .ttl = ttl
  end

  receive
    case :get(key)
      reply .store[key]
    case :set(key, value)
      .store[key] = value
      reply :ok
    case :delete(key)
      .store.delete(key)
      reply :ok
  end
end

cache = Cache.new(ttl: 60)
cache.send(:set, "user:1", "claudio")
cache.send(:get, "user:1")  # => "claudio"
```

---

## Actor Message Typing

Actors can optionally declare their message interface with `receives`, enabling compile-time checking of `.send()` calls:

```opal
actor Cache
  receives :get, :set, :delete

  receive
    case :get(key)
      reply .store[key]
    case :set(key, value)
      .store[key] = value
      reply :ok
    case :delete(key)
      .store.delete(key)
      reply :ok
  end
end

cache = Cache.new()
cache.send(:get, "user:1")     # OK — :get is in receives
cache.send(:gett, "user:1")    # COMPILE WARNING: :gett not in Cache.receives
```

**Rules:**
- `receives :msg1, :msg2, ...` is optional — actors without it accept any symbol (backward compatible).
- When present, `.send()` calls are checked at compile time.
- `receives` uses symbol sets under the hood.
- Named symbol sets work too: `receives HttpMethod` where `type HttpMethod = :get | :post | :put | :delete | :patch`.
- The `receive` block must handle all declared messages (exhaustiveness check).
- Queryable: `Cache.receives()` returns the set of accepted messages.

---

## Structured Concurrency (`parallel`)

The `parallel` block runs expressions concurrently and waits for all to complete.

### Fan-out

```opal
# Fan-out: run expressions concurrently, collect all results
users, orders, inventory = parallel
  fetch_users()
  fetch_orders()
  fetch_inventory()
end
# Blocks until ALL complete.
# Results returned as a tuple, matching the order of expressions.
# If any expression fails, the others are cancelled.
```

### Parallel For

```opal
# Parallel iteration
pages = parallel for url in urls
  Net.fetch(url)
end
# Returns a list of responses, fetched concurrently
```

### Concurrency Limit

```opal
# With a concurrency limit
pages = parallel max: 5 for url in urls
  Net.fetch(url)
end
# At most 5 fetches run at a time
```

### Parallel with Actors

```opal
counts = parallel
  counter_a.send(:get_count)
  counter_b.send(:get_count)
  counter_c.send(:get_count)
end
total = counts[0] + counts[1] + counts[2]
```

### Cancellation

If any branch in a `parallel` block fails, all sibling branches are cancelled and the failure propagates to the caller. This is structured concurrency — no orphaned work.

```opal
try
  a, b = parallel
    fetch_a()   # succeeds
    fetch_b()   # fails!
  end
catch as e
  # fetch_a() is cancelled, error from fetch_b() is raised here
  print(f"Failed: {e.message}")
end
```

### Nesting

```opal
result = parallel
  parallel
    fetch_a()
    fetch_b()
  end
  fetch_c()
end
# Inner parallel block is a child of the outer block.
```

---

## Async / Futures

For when `parallel` is too rigid and you need fine-grained control over individual concurrent operations.

```opal
# async turns any expression into a Future
user_future = async fetch_user(id)

# Do other work while it runs...
prepare_template()

# Auto-await: using the future's value blocks until ready
print(f"Hello, {user_future.name}")  # blocks here if not yet done

# Explicit await (when you want to be clear about the blocking point)
user = await user_future

# Check readiness without blocking
if user_future.ready?()
  print("done!")
end
```

### Async with Actors

```opal
count_future = async counter.send(:get_count)
# ... do other work ...
count = await count_future
```

### Error Handling

Failures are captured in the Future and re-raised when awaited.

```opal
future = async risky_operation()
try
  result = await future
catch as e
  print(f"Operation failed: {e.message}")
end
```

### Rules

- `async expr` returns a `Future(T)` — the expression runs concurrently.
- **Auto-await on use:** accessing a Future's value blocks until the result is ready.
- `await` is available for explicit blocking points.
- `.ready?()` checks completion without blocking.
- **No colored functions:** you don't mark function definitions as async. Any expression can be made async at the call site.
- Failures are captured in the Future and re-raised on await.

---

## Supervisors

Supervisors watch child actors and restart them on failure.

```opal
supervisor AppSupervisor
  strategy :one_for_one       # only restart the failed child
  max_restarts 3, 60           # give up after 3 crashes in 60 seconds

  supervise Logger()
  supervise Cache(ttl: 60)
  supervise Worker()
end

app = AppSupervisor.start!
```

### Strategies

| Strategy | Behavior |
|---|---|
| `:one_for_one` | Restart only the crashed child. |
| `:all_for_one` | Restart all children if one crashes. |
| `:rest_for_one` | Restart the crashed child and all children started after it. |

### `max_restarts` Circuit Breaker

`max_restarts N, S` — if the supervisor has to restart children more than N times within S seconds, it gives up and propagates the failure upward.

### Supervisor Trees

Supervisors can supervise other supervisors, forming a tree.

```opal
supervisor RootSupervisor
  strategy :one_for_one

  supervise AppSupervisor
  supervise MetricsSupervisor
end
```

**Note:** `strategy`, `max_restarts`, and `supervise` are contextual keywords — they are only reserved inside `supervisor` blocks and can be used as identifiers elsewhere.

### Actor Lifecycle Hooks

```opal
actor Worker
  receives :do

  def init()
    .jobs = []
  end

  receive
    case :do(job)
      .jobs.push(job)
      process(job)
      reply :ok
  end

  # Called before the actor stops (crash or shutdown)
  def on_crash(reason)
    log(f"Worker crashed: {reason}. Had {.jobs.length} pending jobs.")
  end

  # Called after a restart
  def on_restart()
    log("Worker restarted")
  end
end
```

---

## Complete Example

```opal
import Net
import JSON

actor RateLimiter
  receives :check, :reset

  def init(max_per_second)
    .max = max_per_second
    .count = 0
  end

  receive
    case :check
      if .count < .max
        .count += 1
        reply :ok
      else
        reply :limited
      end
    case :reset
      .count = 0
      reply :ok
  end
end

def fetch_dashboard(user_id)
  limiter = RateLimiter(max_per_second: 10)

  # Actor message (sync by default)
  status = limiter.send(:check)
  if status == :limited
    fail RateLimitError("Too many requests")
  end

  # Structured concurrency
  profile, notifications, feed = parallel
    fetch_profile(user_id)
    fetch_notifications(user_id)
    fetch_feed(user_id)
  end

  # Async for background work (don't need result now)
  async log_access(user_id)

  {profile: profile, notifications: notifications, feed: feed}
end

# Supervision for production
supervisor DashboardSupervisor
  strategy :one_for_one
  max_restarts 5, 30

  supervise RateLimiter(max_per_second: 100)
end
```

---

## Design Rationale

### Why Four Layers?

Each concurrency layer addresses a distinct need. This avoids a single overloaded primitive that tries to do everything:

| Layer | Purpose | When to use |
|---|---|---|
| Actors | Stateful concurrent entities with isolated state | Long-lived services, shared mutable state behind message passing |
| `parallel` | Structured fan-out with automatic cancellation | Running N independent tasks and waiting for all results |
| `async` / Futures | Fine-grained non-blocking calls | Fire-and-forget work, or when you need a result later but not now |
| Supervisors | Fault tolerance and crash recovery | Production systems where actors must survive failures |

### Why Sync by Default?

Most concurrency models force developers to choose async from the start, infecting every caller up the chain. Opal inverts this: all calls are synchronous and blocking by default. You opt into concurrency explicitly with `async`, `parallel`, or actor `.send()`.

This means:
- Functions are simpler to write and test — no async/await ceremony.
- The default path (sync) is always correct, just potentially slower.
- Concurrency is a conscious choice at the call site, not a viral annotation.

### Why No Colored Functions?

Many languages have "function coloring" — once a function is `async`, every caller must also be `async`. This creates two incompatible worlds.

Opal avoids this entirely. There is no `async def`. Any expression can be made async at the call site:

```opal
# The function itself is just a normal function
def fetch_user(id)
  Net.get(f"/users/{id}")
end

# The CALLER decides whether to run it async
user = fetch_user(1)                 # sync (blocking)
user_future = async fetch_user(1)    # async (non-blocking)
```

This keeps function signatures simple and gives the caller full control over execution strategy.

---

## Summary

| Need | Tool | Syntax |
|---|---|---|
| Stateful concurrent entity | Actor | `actor`, `receive` with `case`, `.send()` |
| Run N things concurrently, wait for all | Parallel block | `parallel ... end` |
| Run N items concurrently | Parallel for | `parallel for x in xs ... end` |
| Limit concurrency | Parallel max | `parallel max: N for ...` |
| Make one call non-blocking | Async/Future | `async expr`, auto-await on use |
| Declare actor interface | Message typing | `receives :msg1, :msg2` |
| Fault tolerance | Supervisor | `supervisor`, `strategy`, `supervise` |
| Crash recovery hooks | Lifecycle | `on_crash(reason)`, `on_restart()` |
