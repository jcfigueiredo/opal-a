# Async & Concurrency Design for Opal

**Date:** 2026-03-01
**Status:** Approved

---

## Design Principles

- **Sync by default.** All calls block and return values directly. Async is opt-in.
- **No colored functions.** There is no `async def`. Any expression can be made async at the call site with the `async` keyword.
- **Structured concurrency.** The `parallel` block is the primary tool for concurrent work. All concurrent work has a parent scope — no orphaned tasks.
- **One way to do each thing.** Actors for stateful concurrency, `parallel` for fan-out, `async` for individual futures, supervisors for fault tolerance.

---

## Layer 1: Actors

Actors are long-lived concurrent entities with isolated state. All external interaction goes through message passing.

### Rules

- `receive` blocks define the public interface (messages the actor responds to).
- `def` methods are internal — only callable from inside the actor.
- `.send()` sends a message to an actor. **Sync by default:** blocks until the actor replies.
- `reply` sends a value back to the caller.

### Syntax

```opal
actor Counter
  def :init()
    .count = 0
  end

  receive :increment
    .count += 1
    reply .count
  end

  receive :get_count
    reply .count
  end

  receive :reset
    .count = 0
    reply :ok
  end

  # Internal helper — NOT accessible from outside
  private def validate_count()
    .count >= 0
  end
end

c = Counter.new()
c.send(:increment)     # => 1 (blocks until reply)
c.send(:increment)     # => 2
c.send(:get_count)     # => 2
c.send(:reset)         # => :ok
```

### Messages with Arguments

```opal
actor Cache
  def :init(ttl::Int32)
    .store = {:}
    .ttl = ttl
  end

  receive :get(key)
    reply .store[key]
  end

  receive :set(key, value)
    .store[key] = value
    reply :ok
  end
end

cache = Cache.new(ttl: 60)
cache.send(:set, "user:1", "claudio")
cache.send(:get, "user:1")  # => "claudio"
```

---

## Layer 2: Structured Concurrency (`parallel`)

The `parallel` block runs expressions concurrently and waits for all to complete.

### Fan-out

```opal
users, orders, inventory = parallel
  fetch_users()
  fetch_orders()
  fetch_inventory()
end
# All three run concurrently.
# Block returns when all complete.
# Results are returned as a tuple, matching the order of expressions.
```

### Parallel For

```opal
pages = parallel for url in urls
  Net.fetch(url)
end
# Returns a list of responses, fetched concurrently.
```

### Concurrency Limit

```opal
pages = parallel max: 5 for url in urls
  Net.fetch(url)
end
# At most 5 fetches run at a time.
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

### Cancellation Rule

If any branch in a `parallel` block fails, all sibling branches are cancelled and the failure propagates to the caller. This is structured concurrency — no orphaned work.

```opal
try
  a, b = parallel
    fetch_a()   # succeeds
    fetch_b()   # fails!
  end
on fail as e
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

## Layer 3: Async / Futures

For when `parallel` is too rigid and you need fine-grained control over individual concurrent operations.

### Syntax

```opal
# async turns any expression into a Future
user_future = async fetch_user(id)

# Do other work while it runs...
prepare_template()

# Auto-await: using the future's value blocks until ready
print(f"Hello, {user_future.name}")  # blocks here if not yet done
```

### Explicit Await

```opal
# When you want to be clear about the blocking point
user = await user_future
```

### Checking Readiness

```opal
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
on fail as e
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

## Layer 4: Supervisors

Supervisors watch child actors and restart them on failure.

### Syntax

```opal
supervisor AppSupervisor
  strategy :one_for_one
  max_restarts 3 within 60

  supervise Logger.new()
  supervise Cache.new(ttl: 60)
  supervise Worker.new()
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

`max_restarts N within S` — if the supervisor has to restart children more than N times within S seconds, it gives up and propagates the failure upward.

### Supervisor Trees

Supervisors can supervise other supervisors, forming a tree.

```opal
supervisor RootSupervisor
  strategy :one_for_one

  supervise AppSupervisor
  supervise MetricsSupervisor
end
```

### Actor Lifecycle Hooks

```opal
actor Worker
  def :init()
    .jobs = []
  end

  receive :do(job)
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
  def :init(max_per_second)
    .max = max_per_second
    .count = 0
  end

  receive :check
    if .count < .max
      .count += 1
      reply :ok
    else
      reply :limited
    end
  end

  receive :reset
    .count = 0
    reply :ok
  end
end

def fetch_dashboard(user_id)
  limiter = RateLimiter.new(max_per_second: 10)

  # Layer 1: Actor message (sync by default)
  status = limiter.send(:check)
  if status == :limited
    fail RateLimitError.new("Too many requests")
  end

  # Layer 2: Structured concurrency
  profile, notifications, feed = parallel
    fetch_profile(user_id)
    fetch_notifications(user_id)
    fetch_feed(user_id)
  end

  # Layer 3: Async for background work (don't need result now)
  async log_access(user_id)

  {profile: profile, notifications: notifications, feed: feed}
end

# Layer 4: Supervision for production
supervisor DashboardSupervisor
  strategy :one_for_one
  max_restarts 5 within 30

  supervise RateLimiter.new(max_per_second: 100)
end
```

---

## Summary

| Need | Tool | Syntax |
|---|---|---|
| Stateful concurrent entity | Actor | `actor`, `receive`, `.send()` |
| Run N things concurrently, wait for all | Parallel block | `parallel ... end` |
| Run N items concurrently | Parallel for | `parallel for x in xs ... end` |
| Limit concurrency | Parallel max | `parallel max: N for ...` |
| Make one call non-blocking | Async/Future | `async expr`, auto-await on use |
| Fault tolerance | Supervisor | `supervisor`, `strategy`, `supervise` |
| Crash recovery hooks | Lifecycle | `on_crash(reason)`, `on_restart()` |
