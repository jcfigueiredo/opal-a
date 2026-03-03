# Self-Hosting Appendix Design

## Goal

Create a new appendix document (`docs/appendix/self-hosting.md`) that proves Opal's language sugar is self-hosted — implemented as macros that expand to plain Opal code. Shows developers exactly how 6 language features work internally, demonstrating that Opal's metaprogramming is powerful enough to build the language itself.

## Approach

Full macro expansions (Approach A): for each piece of language sugar, show three panels — the sugar as developers write it, the expansion it produces, and the actual macro definition. No new syntax or BNF changes — this is a documentation-only addition.

## Document Structure

**File:** `docs/appendix/self-hosting.md`

**Title:** "Self-Hosting: Opal in Opal"

### Opening: Parser Core vs Macro Layer

Frame the key insight: Opal's parser handles a minimal core (def, class, module, actor, enum, if, for, while, match, try, quote, macro, $). Everything else is built on top using Opal's own metaprogramming.

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

Three properties this gives Opal:
1. **Inspectable** — `macroexpand()` shows what sugar generates
2. **Customizable** — developers can write their own versions
3. **Growable** — new features ship as packages, not compiler patches

### The 6 Self-Hosted Features

Each feature follows a three-panel format:

#### 1. `needs` — Dependency Injection

**Sugar:**
```opal
class OrderService
  needs db: Database
  needs mailer: Mailer
end
```

**Expands to:**
```opal
class OrderService
  def init(db: Database, mailer: Mailer)
    .db = db
    .mailer = mailer
  end

  def db() -> Database = .db
  def mailer() -> Mailer = .mailer
end
```

**The macro:** Collects all `needs` declarations in a class body, generates a single `init()` with all parameters, creates instance variable assignments and getter methods.

Key points:
- Multiple `needs` merge into one `init()` (the macro sees the full class body)
- Default values on `needs` become default parameters on `init()`
- The constructor shorthand `OrderService(db: my_db, mailer: my_mailer)` is parser-level sugar for `.new()`

#### 2. `event` — Domain Events

**Sugar:**
```opal
event OrderPlaced
  needs order: Order
  needs placed_at: Time
end
```

**Expands to:** An immutable model class with fields, automatic equality, and string representation.

**The macro:** Reads the `needs` fields and generates a `model` with all fields frozen.

#### 3. `emit` — Event Dispatch

**Sugar:**
```opal
emit OrderPlaced.new(order: o, placed_at: Time.now())
```

**Expands to:** A message send to the event bus actor, either fire-and-forget or synchronous depending on context.

**The macro:** Wraps the event construction in a bus dispatch call.

#### 4. `on` — Event Handlers

**Sugar:**
```opal
on OrderPlaced do |event|
  send_confirmation(event.order)
end
```

**Expands to:** A handler registration on the event bus that pattern-matches incoming events by type.

**The macro:** Registers a closure with the event bus, filtered by event type.

#### 5. `requires` — Preconditions

**Sugar:**
```opal
def withdraw(amount: Float64)
  requires amount > 0, "amount must be positive"
  requires amount <= .balance, "insufficient funds"
  # ...
end
```

**Expands to:** Guard clauses at the top of the function that raise `PreconditionError` on failure.

**The macro:** Moves `requires` expressions into the function body as `if !condition then raise PreconditionError.new(message) end`.

#### 6. `supervisor` — Fault Tolerance

**Sugar:**
```opal
supervisor AppSupervisor
  strategy :one_for_one
  max_restarts 3, 60

  supervise Logger()
  supervise Cache(ttl: 60)
  supervise Worker()
end
```

**Expands to:** An actor class that manages child actors with restart logic, health monitoring, and strategy-based failure handling.

**The macro:** The most complex — generates an actor with `init()` that starts children, `receive` handlers for child failure notifications, and restart logic based on the declared strategy.

### Closing: Implications for Language Evolution

New language features can be:
1. **Prototyped** as macro packages
2. **Battle-tested** by users in real projects
3. **Promoted** to parser-level only if they prove essential and performance-critical

This keeps Opal's core small while allowing the ecosystem to experiment freely.

## Spec Impact

**New file:** `docs/appendix/self-hosting.md`

**Updates:**

| File | Change |
|---|---|
| `Opal.md` | Add link in appendix section |
| `docs/07-metaprogramming/metaprogramming.md` | Update "Self-Hosting Potential" section to reference the appendix |
| `CLAUDE.md` | Update appendix directory description |

## Kept As-Is

- Metaprogramming doc — stays as-is (solid foundation, teaches the tools)
- Existing appendix.md — stays (reference links, different purpose)
- BNF — no changes (this is documentation, not syntax)
