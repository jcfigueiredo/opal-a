# Platform Integration

Opal makes infrastructure services (Redis, Postgres, S3, Docker) feel like native modules. Developers import a client, declare it as a dependency via `needs`, and the runtime handles provisioning (dev), connecting (prod), and lifecycle management. The `Platform` stdlib module orchestrates everything — built entirely on existing primitives with no new syntax.

---

## Overview

The `Platform` module provides a convention-over-configuration approach to infrastructure. Service providers implement a standard protocol (`ServiceProvider[C]`) that separates lifecycle management from client usage. A topology file declares which services an application needs, and the runtime auto-registers clients into the DI container based on the environment. Infrastructure events let application code react to failures and recovery.

---

## 1. The ServiceProvider Protocol

Every infrastructure package ships two things:

1. **Provider** — implements `ServiceProvider[C]`, handles lifecycle (provision, connect, health check, shutdown). Used by the `Platform` runtime.
2. **Client** — the object application code uses (`Redis`, `Postgres`, `S3Storage`). Implements a domain protocol (`Cache`, `Database`, `Storage`).

The `ServiceProvider[C]` protocol defines the lifecycle interface:

```opal
protocol ServiceProvider[C]
  def name() -> String
  def provision(config: Settings) -> C
  def connect(config: Settings) -> C
  def health_check(client: C) -> Bool
  def shutdown(client: C) -> Null
end
```

- `provision` spins up a new service (e.g., Docker container) and returns a connected client.
- `connect` connects to an existing service and returns a client.
- `health_check` verifies the service is responsive.
- `shutdown` cleanly disconnects and tears down.
- The generic parameter `C` is the client type that application code uses.

Application code only sees the client — never the provider.

### Example: Redis Provider and Client

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

`RedisProvider` implements `ServiceProvider[Redis]` — it manages the Redis lifecycle and produces `Redis` client instances. Application code declares `needs cache: Redis` and uses the client directly, unaware of how it was provisioned.

---

## 2. Topology Files

A topology file is a regular `.opl` module that declares infrastructure using `Platform.define`. It lists which services the application needs and how to configure them.

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

Services that differ by environment (like `:storage` above) pass named `dev:` and `prod:` providers. Services that use the same provider in all environments pass a single provider.

### Discovery via `opal.toml`

The runtime finds the topology file through the `[infrastructure]` section in `opal.toml`:

```toml
[package]
name = "my_web_app"
version = "0.1.0"

[infrastructure]
file = "infra.opl"

[dependencies]
opal_redis = "1.0"
opal_postgres = "1.0"
opal_storage = "1.0"
opal_web = "1.2"
```

### Importing Topology for Metadata

The topology is a regular module and can be imported from any file for runtime metadata access:

```opal
from Infra import infrastructure

infrastructure.services()        # => [:cache, :db, :storage]
infrastructure.config(:cache)    # => RedisProvider config
infrastructure.status(:db)       # => :running | :stopped | :error
```

---

## 3. DI Auto-Registration

When the runtime starts, it automatically provisions or connects to infrastructure services and registers them in the DI container. The sequence is:

1. Read `opal.toml` → find `infra.opl`
2. Evaluate the `Platform.define` block → get the list of providers
3. Based on `OPAL_ENV`:
   - `dev` (default): call `provider.provision()` (spins up containers)
   - `production`: call `provider.connect()` (connects to existing)
   - `test`: call `provider.provision()` with isolated random ports
4. Register each resulting **client** in the global `Container`
5. Any class with `needs cache: Redis` gets the live client injected

Application code needs zero manual wiring:

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

`UserService` declares its dependencies via `needs`. The `Platform` runtime has already provisioned Redis and Postgres and registered their clients in the container — `UserService` gets them injected automatically.

---

## 4. Environment Handling

| Mode | `OPAL_ENV` | Behavior |
|---|---|---|
| Dev | unset or `dev` | Auto-provision via Docker, tear down on exit |
| Production | `production` | Connect to existing services via config/env vars |
| Test | `test` | Provision isolated ephemeral services, random ports |

### Dev (default) — auto-provision

```bash
$ opal run app.opl
▶ [dev] Reading infrastructure from infra.opl...
▶ Provisioning cache (redis:latest, port 6379)... ✔
▶ Provisioning db (postgres:16, port 5432)... ✔
▶ Provisioning storage (local, ./storage)... ✔
▶ Registering services in container... ✔
▶ Starting app...
Listening on :8080
```

Services are torn down on exit. Uses Docker under the hood (like Testcontainers).

### Production — connect to existing

```bash
$ OPAL_ENV=production opal run app.opl
▶ [prod] Reading infrastructure from infra.opl...
▶ Connecting to cache (cache.prod.internal:6379)... ✔
▶ Connecting to db (db.prod.internal:5432)... ✔
▶ Connecting to storage (myapp-assets, us-east-1)... ✔
▶ Health checks... ✔
▶ Starting app...
Listening on :8080
```

Production endpoints come from config files or environment variables.

### Test — isolated ephemeral services

```bash
$ opal test
▶ [test] Provisioning isolated services...
▶ cache (redis:latest, port 16379)... ✔
▶ db (postgres:16, port 15432)... ✔
▶ Running tests...
  12 passed, 0 failed
▶ Tearing down test services... ✔
```

### Config Priority

Environment configuration extends the `settings model` priority chain:

1. Provider defaults in topology file
2. Config file (`config/{env}.toml`)
3. `.env` file
4. Environment variables (`OPAL_CACHE_HOST`, `OPAL_DB_PORT`, etc.)
5. Explicit overrides

Example production config file:

```toml
# config/production.toml
[cache]
host = "cache.prod.internal"
port = 6379

[db]
host = "db.prod.internal"
port = 5432
database = "myapp_prod"
pool_size = 20
```

---

## 5. Infrastructure Events

The `Platform` module emits events using Opal's existing domain event system. Application code listens with `on` — the same pattern used for domain events.

### Event Models

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

### Listening and Reacting

```opal
from Platform import ServiceDown, ConnectionLost, ServiceRecovered,
                     HealthCheckFailed, ConnectionRestored

on ServiceDown do |event|
  Logger.warn(f"Service {event.name} is down: {event.reason}")
end

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

on HealthCheckFailed do |event|
  if event.consecutive_failures > 3
    Logger.critical(f"{event.service} failing health checks")
  end
end
```

Events are dispatched asynchronously through the actor-based event bus, just like domain events. Handlers get supervision and fault tolerance for free.

---

## 6. Storage Provider — Environment-Transparent Example

This is the canonical example of how providers swap transparently between dev and prod. Application code depends on a protocol — the runtime provides the correct implementation.

### The Storage Protocol

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

### Local Provider (dev)

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

### S3 Provider (prod)

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
  # ...
end
```

### Application Code

Application code depends on the `Storage` protocol, not any concrete provider:

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

In dev, `AvatarService` gets a `LocalStorage` writing to `./storage/`. In production, it gets an `S3Storage` writing to an S3 bucket. The code is identical — the topology file and `OPAL_ENV` determine which implementation is injected.

---

## 7. CLI Commands

| Command | Description |
|---|---|
| `opal infra status` | Show infrastructure service status (running, stopped, error) |
| `opal infra up` | Provision infrastructure services without running the app |
| `opal infra down` | Tear down all infrastructure services |
| `opal infra health` | Run health checks on all infrastructure services |

---

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

Platform integration is built entirely on existing Opal primitives: protocols for abstraction, `needs` for dependency injection, `Container` for auto-wiring, `settings model` for configuration, and domain events for observability. No new syntax or keywords are introduced.
