# Platform Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add platform integration documentation to the Opal spec — a `Platform` stdlib module with `ServiceProvider[C]` protocol, topology files, auto-DI registration, environment-aware provisioning, infrastructure events, and a Storage provider example.

**Architecture:** One new doc (`docs/06-patterns/platform-integration.md`) is the main feature document. Six existing files get small targeted updates: Opal.md hub (summary + stdlib table), dependency-injection.md (auto-registration section), modules-and-imports.md (infrastructure modules section), stdlib.md (Platform entry), tooling.md (CLI commands), and CLAUDE.md. No BNF changes, no new keywords — this is all library-level design.

**Tech Stack:** Markdown files only. No code, no tests.

---

## Context

**Design doc:** `docs/plans/2026-03-03-platform-integration-design.md`

**Files to create:** 1 — `docs/06-patterns/platform-integration.md`

**Files to modify:** 6 — `Opal.md`, `docs/06-patterns/dependency-injection.md`, `docs/03-functions-and-types/modules-and-imports.md`, `docs/08-stdlib/stdlib.md`, `docs/09-tooling/tooling.md`, `CLAUDE.md`

**Key concepts:**
- `ServiceProvider[C]` protocol — lifecycle management (provision/connect/health/shutdown), returns a client `C`
- Topology file — regular `.opl` module using `Platform.define` to declare infrastructure
- DI auto-registration — runtime loads topology, provisions/connects, registers clients into `Container`
- Environment handling — dev (auto-provision via Docker), production (connect to existing), test (isolated ephemeral)
- Infrastructure events — `ServiceDown`, `ConnectionLost`, `HealthCheckFailed`, `ServiceRecovered`, `ConnectionRestored`
- Storage example — `LocalStorageProvider` (dev) vs `S3StorageProvider` (prod), both implementing `Storage` protocol

---

### Task 1: Create `docs/06-patterns/platform-integration.md`

**Files:**
- Create: `docs/06-patterns/platform-integration.md`
- Reference: `docs/plans/2026-03-03-platform-integration-design.md` (the design doc)
- Reference: `docs/06-patterns/dependency-injection.md` (for matching doc structure)

**What to write:**

Follow the same document structure as other pattern docs (Overview → Core sections → Extended Examples → Summary table). The content comes from the design doc but is reorganized into spec-doc format.

The document must have these sections in this order:

```markdown
# Platform Integration

---

## Overview

2-3 sentences: Infrastructure services feel like native Opal modules. The `Platform` stdlib module handles provisioning, connecting, and lifecycle. Built on existing primitives — no new syntax.

---

## 1. The ServiceProvider Protocol

The `ServiceProvider[C]` protocol definition:

```opal
protocol ServiceProvider[C]
  def name() -> String
  def provision(config: Settings) -> C
  def connect(config: Settings) -> C
  def health_check(client: C) -> Bool
  def shutdown(client: C) -> Null
end
```

Explain:
- Provider handles lifecycle, returns a client `C`
- Provider and client are separate classes
- Application code only sees the client
- Example: `RedisProvider` implements `ServiceProvider[Redis]`

Show a complete Redis provider + client example:

```opal
# RedisProvider — lifecycle (used by Platform runtime)
class RedisProvider implements ServiceProvider[Redis]
  needs port: Int32 = 6379
  needs ttl: Int32 = 3600

  def name() -> String = "redis"

  def provision(config: Settings) -> Redis
    Platform.container("redis:latest", ports: [.port])
    connect(config)
  end

  def connect(config: Settings) -> Redis
    Redis.new(host: config.host, port: config.port, ttl: config.ttl)
  end

  def health_check(client: Redis) -> Bool  ... end
  def shutdown(client: Redis) -> Null  ... end
end

# Redis — the client (used by application code)
class Redis implements Cache
  needs host: String
  needs port: Int32
  needs ttl: Int32

  def get(key: String) -> String?  ... end
  def set(key: String, value: String) -> Null  ... end
  def delete(key: String) -> Null  ... end
end
```

## 2. Topology Files

Explain `Platform.define` and the topology file convention. Show full example:

```opal
# infra.opl
from Platform import define
from OpalRedis import RedisProvider
from OpalPostgres import PostgresProvider
from OpalStorage import LocalStorageProvider, S3StorageProvider

infrastructure = define do |services|
  services.add(:cache, RedisProvider.new(port: 6379, ttl: 60))
  services.add(:db, PostgresProvider.new(
    host: "localhost",
    port: 5432,
    pool_size: 5,
    database: "myapp_dev"
  ))
  services.add(:storage,
    dev: LocalStorageProvider.new(root: "./storage"),
    prod: S3StorageProvider.new(bucket: "myapp-assets", region: "us-east-1")
  )
end

export infrastructure
```

Show `opal.toml` discovery:

```toml
[infrastructure]
file = "infra.opl"
```

Show importing topology for metadata:

```opal
from Infra import infrastructure

infrastructure.services()        # => [:cache, :db, :storage]
infrastructure.config(:cache)    # => RedisProvider config
infrastructure.status(:db)       # => :running | :stopped | :error
```

## 3. DI Auto-Registration

Explain the runtime sequence:
1. Read `opal.toml` → find `infra.opl`
2. Evaluate `Platform.define` block → get providers
3. Based on `OPAL_ENV`: call `provision()` or `connect()`
4. Register clients in global `Container`
5. Classes with `needs cache: Redis` get injected automatically

Show that application code needs zero manual wiring:

```opal
class UserService
  needs cache: Redis
  needs db: Postgres

  def find_user(id: Int32) -> User
    .cache.get(f"user:{id}") or do
      user = .db.query("SELECT * FROM users WHERE id = $1", id)
      .cache.set(f"user:{id}", user)
      user
    end
  end
end
```

## 4. Environment Handling

Table of modes:

| Mode | `OPAL_ENV` | Behavior |
|---|---|---|
| Dev | unset or `dev` | Auto-provision via Docker, tear down on exit |
| Production | `production` | Connect to existing services via config/env vars |
| Test | `test` | Provision isolated ephemeral services, random ports |

Show CLI output for each mode (dev, prod, test) — copy from design doc.

Config priority (extends `settings model`):
1. Provider defaults in topology file
2. Config file (`config/{env}.toml`)
3. `.env` file
4. Environment variables (`OPAL_CACHE_HOST`, `OPAL_DB_PORT`)
5. Explicit overrides

Show `config/production.toml` example:

```toml
[cache]
host = "cache.prod.internal"
port = 6379

[db]
host = "db.prod.internal"
port = 5432
database = "myapp_prod"
pool_size = 20
```

## 5. Infrastructure Events

Explain that Platform emits events via Opal's domain event system.

List the event models:

```opal
model ServiceDown
  needs name: Symbol
  needs reason: String
  needs timestamp: Time
end

model ConnectionLost
  needs service: Symbol
  needs error: Error
  needs retry_count: Int32
  needs timestamp: Time
end

model HealthCheckFailed
  needs service: Symbol
  needs consecutive_failures: Int32
  needs last_error: String
  needs timestamp: Time
end

model ServiceRecovered
  needs name: Symbol
  needs downtime: Duration
  needs timestamp: Time
end

model ConnectionRestored
  needs service: Symbol
  needs downtime: Duration
  needs timestamp: Time
end
```

Show listening and reacting:

```opal
from Platform import ServiceDown, ConnectionLost, ServiceRecovered

on ConnectionLost do |event|
  Logger.error(f"Lost connection to {event.service}: {event.error}")
  match event.service
    case :cache
      Platform.mark_degraded(:cache)
    case :db
      Platform.restart(:db, max_retries: 3)
  end
end

on ServiceRecovered do |event|
  Logger.info(f"Service {event.name} recovered after {event.downtime}")
end
```

## 6. Storage Provider — Environment-Transparent Example

This is the canonical example of how providers swap transparently between dev and prod.

Show the `Storage` protocol:

```opal
protocol Storage
  def read(path: String) -> Bytes?
  def write(path: String, data: Bytes) -> Null
  def delete(path: String) -> Bool
  def exists?(path: String) -> Bool
  def list(prefix: String) -> List[String]
  def url(path: String) -> String
end
```

Show `LocalStorageProvider` + `LocalStorage` (dev):

```opal
class LocalStorageProvider implements ServiceProvider[LocalStorage]
  needs root: String = "./storage"

  def provision(config: Settings) -> LocalStorage
    File.mkdir_p(.root)
    LocalStorage.new(root: .root)
  end

  def connect(config: Settings) -> LocalStorage
    LocalStorage.new(root: config.root)
  end
end

class LocalStorage implements Storage
  needs root: String
  def read(path: String) -> Bytes? = File.read(f"{.root}/{path}")
  def write(path: String, data: Bytes) = File.write(f"{.root}/{path}", data)
  def delete(path: String) -> Bool = File.delete(f"{.root}/{path}")
  def exists?(path: String) -> Bool = File.exists?(f"{.root}/{path}")
  def list(prefix: String) -> List[String] = File.list_dir(f"{.root}/{prefix}")
  def url(path: String) -> String = f"file://{.root}/{path}"
end
```

Show `S3StorageProvider` + `S3Storage` (prod):

```opal
class S3StorageProvider implements ServiceProvider[S3Storage]
  needs bucket: String
  needs region: String = "us-east-1"

  def provision(config: Settings) -> S3Storage
    Platform.container("minio/minio", ports: [9000])
    S3Storage.new(endpoint: "localhost:9000", bucket: .bucket)
  end

  def connect(config: Settings) -> S3Storage
    S3Storage.new(bucket: config.bucket, region: config.region)
  end
end

class S3Storage implements Storage
  needs bucket: String
  needs region: String
  def read(path: String) -> Bytes?  ... end
  def write(path: String, data: Bytes)  ... end
  def url(path: String) -> String = f"https://{.bucket}.s3.{.region}.amazonaws.com/{path}"
end
```

Show application code using the protocol type:

```opal
class AvatarService
  needs storage: Storage

  def upload(user_id: Int32, data: Bytes) -> String
    path = f"avatars/{user_id}.png"
    .storage.write(path, data)
    .storage.url(path)
  end

  def get(user_id: Int32) -> Bytes?
    .storage.read(f"avatars/{user_id}.png")
  end
end
```

## 7. CLI Commands

| Command | Description |
|---|---|
| `opal infra status` | Show infrastructure service status |
| `opal infra up` | Provision services without running the app |
| `opal infra down` | Tear down all services |
| `opal infra health` | Run health checks on all services |

## Summary

| Concept | Mechanism |
|---|---|
| Service lifecycle | `ServiceProvider[C]` protocol |
| Infrastructure declaration | Topology file with `Platform.define` |
| Service discovery | `opal.toml` `[infrastructure]` section |
| Dependency injection | Auto-registration into `Container` via `needs` |
| Environment switching | `OPAL_ENV` — dev (provision), prod (connect), test (isolated) |
| Config priority | Provider defaults → config file → .env → env vars → overrides |
| Resilience | Infrastructure events via `on ServiceDown`, `on ConnectionLost`, etc. |
| Environment transparency | Protocol-typed `needs` (e.g., `needs storage: Storage`) |
| CLI | `opal infra status/up/down/health` |
```

**Commit:** `"Add platform integration spec document"`

---

### Task 2: Update Opal.md hub — add Platform summary and stdlib entry

**Files:**
- Modify: `Opal.md`

**What to change:**

**Change 1 — Add hub summary in Section 9 (Software Engineering Patterns):**

Find the Specifications subsection (the last subsection before section 10). After its closing `---` separator, add a new subsection:

```markdown
### Platform Integration

Infrastructure services (Redis, Postgres, S3, Docker) are imported as modules and declared via `needs` like any dependency. The `Platform` stdlib module handles provisioning in dev, connecting in production, and lifecycle management — built entirely on existing primitives.

```opal
from Platform import define
from OpalRedis import RedisProvider
from OpalPostgres import PostgresProvider

infrastructure = define do |services|
  services.add(:cache, RedisProvider.new(port: 6379))
  services.add(:db, PostgresProvider.new(host: "localhost", port: 5432))
end
```

> See [Platform Integration](docs/06-patterns/platform-integration.md) for topology files, DI auto-registration, environment handling, infrastructure events, and the Storage provider example.

---
```

**Change 2 — Add Platform to the stdlib module table (Section 11):**

Find the stdlib table. Add a new row after `Settings` and before `Reflect`:

```
| `Platform` | Infrastructure services: topology definition, service providers, auto-DI registration, environment handling, health checks, lifecycle events |
```

**How to verify:** Check that the new subsection follows the same format as Dependency Injection, Domain Events, and Specifications subsections. Check that the stdlib table row matches the format of adjacent rows.

**Commit:** `"Add platform integration to Opal.md hub and stdlib table"`

---

### Task 3: Update `docs/06-patterns/dependency-injection.md` — add auto-registration section

**Files:**
- Modify: `docs/06-patterns/dependency-injection.md`

**What to change:**

Find the "Complete DDD Example" section (the last example section before "Design Rationale"). After it ends, insert a new section:

```markdown
---

## Platform Auto-Registration

For applications using infrastructure services (Redis, Postgres, S3, etc.), the `Platform` runtime automatically provisions services and registers them in the DI container — no manual wiring needed.

### How It Works

1. Define infrastructure in a topology file (`infra.opl`):

```opal
from Platform import define
from OpalRedis import RedisProvider
from OpalPostgres import PostgresProvider

infrastructure = define do |services|
  services.add(:cache, RedisProvider.new(port: 6379))
  services.add(:db, PostgresProvider.new(
    host: "localhost",
    port: 5432,
    pool_size: 5
  ))
end
```

2. Reference in `opal.toml`:

```toml
[infrastructure]
file = "infra.opl"
```

3. The runtime:
   - Loads `infra.opl` and evaluates the `Platform.define` block
   - Calls `provider.provision()` (dev) or `provider.connect()` (production)
   - Registers each client in the global `Container`
   - Injects clients via `needs` — just like manually registered dependencies

### Application Code — Zero Changes

```opal
class UserService
  needs cache: Redis
  needs db: Postgres

  def find_user(id: Int32) -> User
    .cache.get(f"user:{id}") or do
      user = .db.query("SELECT * FROM users WHERE id = $1", id)
      .cache.set(f"user:{id}", user)
      user
    end
  end
end

# No explicit wiring — Platform registered Redis and Postgres clients
service = UserService.new()
```

### Environment Modes

| Mode | Behavior |
|---|---|
| Dev (default) | Auto-provision services via Docker, tear down on exit |
| Production | Connect to existing services via config/env vars |
| Test | Provision isolated ephemeral services with random ports |

> See [Platform Integration](platform-integration.md) for topology syntax, `ServiceProvider[C]` protocol, config priority, and infrastructure events.
```

**How to verify:** The new section should sit between the examples and the Design Rationale section. The cross-link uses a relative path since both files are in `docs/06-patterns/`.

**Commit:** `"Add Platform auto-registration section to dependency-injection.md"`

---

### Task 4: Update `docs/03-functions-and-types/modules-and-imports.md` — add infrastructure modules section

**Files:**
- Modify: `docs/03-functions-and-types/modules-and-imports.md`

**What to change:**

Find the "Packages" section (section 5). After it ends, before "Design Rationale" (section 6), insert a new section. Then renumber "Design Rationale" from 6 to 7, and renumber any subsequent sections accordingly.

New section to insert:

```markdown
---

## 6. Infrastructure Modules

Infrastructure providers (Redis, Postgres, S3, etc.) are regular Opal packages that ship a `ServiceProvider[C]` implementation. They are imported like any other module:

```opal
from OpalRedis import RedisProvider
from OpalPostgres import PostgresProvider
from OpalStorage import LocalStorageProvider, S3StorageProvider
```

These providers are used in topology files — regular `.opl` modules that declare infrastructure via `Platform.define`:

```opal
from Platform import define
from OpalRedis import RedisProvider

infrastructure = define do |services|
  services.add(:cache, RedisProvider.new(port: 6379))
end

export infrastructure
```

The `Platform` module reads the topology file (specified in `opal.toml`), provisions or connects services based on the environment, and auto-registers clients in the DI container. Application code imports clients and protocols like any other type — no special import syntax needed.

> See [Platform Integration](../06-patterns/platform-integration.md) for the full topology file spec, `ServiceProvider[C]` protocol, and environment handling.
```

**Also:** Renumber "Design Rationale" from `## 6.` to `## 7.` and any sections after it (increment by 1). Check the Summary table for section references that may need updating.

**Commit:** `"Add infrastructure modules section to modules-and-imports.md"`

---

### Task 5: Update `docs/08-stdlib/stdlib.md` — add Platform entry

**Files:**
- Modify: `docs/08-stdlib/stdlib.md`

**What to change:**

Find the stdlib module table. Add a new row for `Platform` after `Settings` and before `Reflect`:

```
| `Platform` | Infrastructure services: topology definition with `define`, `ServiceProvider[C]` protocol, auto-DI registration, environment handling (dev/prod/test), health checks, lifecycle events (`ServiceDown`, `ConnectionLost`, `ServiceRecovered`) |
```

**How to verify:** The new row should match the format of adjacent rows. Check that it's alphabetically reasonable (Platform comes after `Net`, `Settings` and before `Reflect` — or wherever it fits logically in the existing grouping).

**Commit:** `"Add Platform module to stdlib table"`

---

### Task 6: Update `docs/09-tooling/tooling.md` — add `opal infra` CLI commands

**Files:**
- Modify: `docs/09-tooling/tooling.md`

**What to change:**

Find the CLI Summary table (the table with `opal run`, `opal test`, `opal fmt`, etc.). Add a new group of rows for infrastructure commands, after the package manager commands (`opal pkg ...`):

```
| `opal infra status` | Show infrastructure service status (running, stopped, error) |
| `opal infra up` | Provision infrastructure services without running the app |
| `opal infra down` | Tear down all infrastructure services |
| `opal infra health` | Run health checks on all infrastructure services |
```

**How to verify:** The new rows should match the table format. The `opal infra` commands should be grouped together, after `opal pkg` commands.

**Commit:** `"Add opal infra CLI commands to tooling.md"`

---

### Task 7: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**What to change:**

**Change 1 — Update the Documentation Structure table:**

Find the row for `docs/06-patterns/`. Change its description from:

```
| `docs/06-patterns/` | Dependency injection, events, specifications |
```

To:

```
| `docs/06-patterns/` | Dependency injection, events, specifications, platform integration |
```

**Change 2 — Add to Key Language Design Rules** (if not already implied by existing rules):

After the existing rules, add:

```
- **Platform integration via `Platform` stdlib**: Infrastructure services declared in topology files, auto-registered into DI via `ServiceProvider[C]` protocol. `needs cache: Redis` just works.
```

**Commit:** `"Update CLAUDE.md for platform integration"`

---

### Task 8: Final audit

**Files:** All 7 modified/created files

**Step 1:** Verify all cross-links resolve:
```bash
grep -rn 'platform-integration.md' Opal.md docs/ CLAUDE.md | grep -v 'docs/plans/'
```
Expected: links in Opal.md, dependency-injection.md, modules-and-imports.md, and optionally stdlib.md.

**Step 2:** Verify consistent terminology — search for any inconsistencies:
```bash
grep -rn 'ServiceProvider' docs/06-patterns/platform-integration.md docs/06-patterns/dependency-injection.md Opal.md
```
Expected: all references use `ServiceProvider[C]` (with generic parameter).

**Step 3:** Verify all code examples use the current Opal syntax (`: ` for types, `[T]` for generics, `from X import Y`, `Fn()` for function types, `do...end` for multi-line closures):
```bash
grep -n '::' docs/06-patterns/platform-integration.md
grep -n 'import.*\.{' docs/06-patterns/platform-integration.md
```
Expected: zero matches (no old syntax).

**Step 4:** Check that the new stdlib table entries in `Opal.md` and `docs/08-stdlib/stdlib.md` are consistent (both mention Platform with similar descriptions).

Fix any issues found.

**Commit:** `"Audit and fix platform integration docs"` (only if fixes needed)

---

## Dependency Graph

```
Task 1 (create platform-integration.md)
  └─> Task 2 (update Opal.md hub + stdlib)
  └─> Task 3 (update dependency-injection.md)
  └─> Task 4 (update modules-and-imports.md)
  └─> Task 5 (update stdlib.md)
  └─> Task 6 (update tooling.md)
  └─> Task 7 (update CLAUDE.md)
        └─> Task 8 (final audit)
```

Task 1 must go first (creates the file others link to). Tasks 2-7 are independent of each other and can run in any order. Task 8 is last.
