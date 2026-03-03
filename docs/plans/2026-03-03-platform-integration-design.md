# Platform Integration Design

## Goal

Make infrastructure services (Redis, Postgres, S3, Docker) feel like native Opal modules. Developers import a client, declare it as a dependency via `needs`, and the runtime handles provisioning (dev), connecting (prod), and lifecycle management — zero new syntax, built entirely on existing Opal primitives.

## Approach

Convention over configuration using existing Opal constructs: `Platform` stdlib module, `ServiceProvider[C]` protocol, DI via `needs` + `Container`, `settings model` for config, domain events for infrastructure observability. No new keywords, no BNF changes.

## Architecture

### Provider / Client Separation

Every infrastructure package ships two things:

1. **Provider** — implements `ServiceProvider[C]`, handles lifecycle (provision, connect, health check, shutdown). Used by the `Platform` runtime.
2. **Client** — the object application code uses (`Redis`, `Postgres`, `S3Storage`). Implements a domain protocol (`Cache`, `Database`, `Storage`).

```opal
# In Platform stdlib
protocol ServiceProvider[C]
  def name() -> String
  def provision(config: Settings) -> C
  def connect(config: Settings) -> C
  def health_check(client: C) -> Bool
  def shutdown(client: C) -> Null
end
```

Application code only sees clients:

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

### Topology File

A regular `.opl` file that declares infrastructure using `Platform.define`. Referenced in `opal.toml`.

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

Discovery via `opal.toml`:

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

The topology is importable from any file for metadata access:

```opal
from Infra import infrastructure

infrastructure.services()        # => [:cache, :db, :storage]
infrastructure.config(:cache)    # => RedisProvider config
infrastructure.status(:db)       # => :running | :stopped | :error
```

### DI Auto-Registration

When the runtime starts:

1. Reads `opal.toml` → finds `infra.opl`
2. Evaluates the `Platform.define` block → gets the list of providers
3. Based on `OPAL_ENV`:
   - `dev` (default): calls `provider.provision()` (spins up containers)
   - `production`: calls `provider.connect()` (connects to existing)
   - `test`: calls `provider.provision()` with isolated random ports
4. Registers each resulting **client** in the global DI container
5. Any class with `needs cache: Redis` gets the live client injected

### Environment Handling

**Dev (default) — auto-provision:**

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

**Production — connect to existing:**

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

**Test — isolated ephemeral services:**

```bash
$ opal test
▶ [test] Provisioning isolated services...
▶ cache (redis:latest, port 16379)... ✔
▶ db (postgres:16, port 15432)... ✔
▶ Running tests...
  12 passed, 0 failed
▶ Tearing down test services... ✔
```

**Environment config priority** (extends `settings model` priority):

1. Provider defaults in topology file
2. Config file (`config/{env}.toml`)
3. `.env` file
4. Environment variables (`OPAL_CACHE_HOST`, `OPAL_DB_PORT`, etc.)
5. Explicit overrides

### Infrastructure Events

The `Platform` module emits events using Opal's existing domain event system. Application code listens with `on`:

```opal
from Platform import ServiceDown, ServiceRecovered,
                     HealthCheckFailed, ConnectionLost,
                     ConnectionRestored

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

Event models shipped with the `Platform` stdlib:

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

### Storage Provider — Canonical Environment-Transparent Example

Demonstrates the pattern of local dev → cloud production with zero application code changes.

**The protocol:**

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

**Local provider (dev):**

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

**S3 provider (prod):**

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

**Application code uses the protocol:**

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

### CLI Commands

```bash
opal infra status          # show service status
opal infra up              # provision without running app
opal infra down            # tear down services
opal infra health          # run health checks
```

## Spec Impact

**New file:** `docs/06-patterns/platform-integration.md`

**Updates to existing files:**

| File | Change |
|---|---|
| `Opal.md` | Add summary + link in Software Engineering Patterns section |
| `docs/06-patterns/dependency-injection.md` | Add section on Platform auto-registration into the DI container |
| `docs/03-functions-and-types/modules-and-imports.md` | Mention infrastructure modules as regular imports |
| `docs/08-stdlib/stdlib.md` | Add `Platform` module to stdlib table |
| `docs/09-tooling/tooling.md` | Add `opal infra` CLI commands |
| `CLAUDE.md` | Add platform integration to key rules |

**No changes to:** BNF grammar, type system, keywords, or DI semantics.

## Kept As-Is

- DI via `needs` — unchanged, Platform just auto-registers into the container
- Domain events via `emit`/`on` — unchanged, Platform emits standard event models
- `settings model` config loading — unchanged, Platform extends the same priority chain
- Protocols for service abstraction — unchanged, `Storage`, `Cache`, `Database` are regular protocols
