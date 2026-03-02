# Validation & Settings (Models)

---

## Overview

The `model` keyword defines validated, immutable data structures — Opal's equivalent of Pydantic's BaseModel. Models validate on construction, serialize to/from dicts and JSON, and can serve as application settings with environment/config file loading.

---

## 1. The `model` Keyword

A `model` is a dedicated construct for validated, immutable data. It differs from `class` in semantics: data with constraints vs behavior with state.

```opal
model User
  needs name::String where |v| v.length > 0
  needs email::String where is_email
  needs age::Int32 where |v| v >= 0
  needs role::String = "member"
end

# Construction — validates all fields
user = User.new(
  name: "claudio",
  email: "c@test.com",
  age: 15
)

# Immutable — cannot be modified
user.name         # => "claudio"
user.name = "x"   # COMPILE ERROR — models are immutable

# Create modified copy (also validated)
updated = user.copy(age: 16)
```

### How `model` Differs from `class`

| | `model` | `class` |
|---|---|---|
| Mutability | Immutable | Mutable |
| Validation | On construction, automatic | Manual (guards) |
| Serialization | Built-in | None |
| Purpose | Data with constraints | Behavior with state |
| `needs` | Validated fields | Dependencies |

### Rules

- All fields declared with `needs` — same syntax as classes.
- Validation runs on construction (`.new()`), deserialization (`.from_dict()`, `.from_json()`), and `.copy()`.
- Failed validation raises `ValidationError` with field name and reason.
- Models can have methods (computed properties, formatting) but cannot mutate fields.

---

## 2. Field Validation

Simple constraints go inline with `where`. Complex cross-field validation uses a `validate` block. Named guards from Opal's guard system work directly in `where` clauses.

### Three Forms of `where`

```opal
# Define reusable guards
guard is_email(value) fails :invalid_email
  return /^[^@]+@[^@]+\.[^@]+$/.match?(value)
end

guard is_required(value) fails :required
  return value != null and value.to_string().length > 0
end

guard min_length(value, n) fails :too_short
  return value.length >= n
end

model User
  # Named guard — value passed automatically
  needs email::String where is_email

  # Inline closure
  needs age::Int32 where |v| v >= 0

  # Named guard with partial application
  needs username::String where min_length(3)

  # Multiple constraints (comma-separated, all must pass)
  needs password::String where is_required, |v| v.length >= 8

  needs role::String = "member"
end
```

### Cross-Field Validation

```opal
model Order
  needs quantity::Int32 where |v| v > 0
  needs price::Float64 where |v| v > 0.0
  needs discount::Float64 = 0.0

  validate do
    if .discount > .price
      fail ValidationError.new(
        field: "discount",
        reason: "cannot exceed price"
      )
    end
  end
end
```

### Validation Order

1. **Type checking** — field types verified first.
2. **Inline `where`** — per-field constraints run next.
3. **`validate` blocks** — cross-field validation runs last (all fields are populated).

### Rules

- `where guard_name` — named guard, value passed automatically.
- `where |v| expr` — inline closure, must return Bool.
- `where guard_name(args)` — partial application, value is the first argument.
- Comma-separated to combine: `where is_required, is_email`.
- If a `where` returns false, raises `ValidationError` with the field name.
- `validate do ... end` runs after all fields pass individual checks.
- Multiple `validate` blocks allowed — run in declaration order.

---

## 3. Serialization

Models automatically get `to_dict`/`from_dict` and `to_json`/`from_json`. Nested models serialize recursively. Deserialization validates on load.

```opal
model Address
  needs street::String
  needs city::String
  needs zip::String where |v| /^\d{5}$/.match?(v)
end

model User
  needs name::String where |v| v.length > 0
  needs email::String where is_email
  needs age::Int32 where |v| v >= 0
  needs address::Address
  needs tags::List(String) = []
end

# Serialize
user = User.new(
  name: "claudio",
  email: "c@test.com",
  age: 15,
  address: Address.new(street: "123 Main", city: "Springfield", zip: "62704")
)

user.to_dict()
# => {"name": "claudio", "email": "c@test.com", "age": 15,
#     "address": {"street": "123 Main", "city": "Springfield", "zip": "62704"},
#     "tags": []}

user.to_json()
# => '{"name": "claudio", ...}'

# Deserialize — validates on load
user = User.from_dict({
  "name": "claudio",
  "email": "c@test.com",
  "age": 15,
  "address": {"street": "123 Main", "city": "Springfield", "zip": "62704"}
})

user = User.from_json('{"name": "claudio", ...}')
```

### Rules

- `to_dict()` returns `Dict(String, Any)` — field names as keys, values serialized recursively.
- `to_json()` returns a JSON string.
- `from_dict(dict)` and `from_json(json)` are static methods that construct and validate.
- Nested models are serialized/deserialized recursively.
- Enum fields serialize to their variant name (simple) or a dict (data-carrying).
- Fields with defaults can be omitted in input — defaults apply.

---

## 4. Settings Model

`Settings` is a built-in base that adds configuration loading from environment variables and config files. Only the root model is `Settings` — nested groups are regular `model`s.

### Defining Settings

```opal
# Nested groups are regular models
model DatabaseSettings
  needs host::String = "localhost"
  needs port::Int32 = 5432
  needs name::String = "opal_dev"
  needs pool_size::Int32 = 5 where |v| v > 0
end

model CacheSettings
  needs host::String = "localhost"
  needs port::Int32 = 6379
  needs ttl::Int32 = 3600 where |v| v > 0
end

# Only the root is Settings
model AppSettings as Settings
  needs debug::Bool = false
  needs secret_key::String
  needs log_level::String = "info" where |v| v in ["debug", "info", "warn", "error"]
  needs db::DatabaseSettings         # group
  needs cache::CacheSettings         # group
end
```

### Loading Settings

```opal
# Load from env with prefix
settings = AppSettings.load(env_prefix: "OPAL_")

# Load from env + config file
settings = AppSettings.load(
  env_prefix: "OPAL_",
  config: "config.toml"
)

# Explicit args override everything
settings = AppSettings.load(
  env_prefix: "OPAL_",
  config: "config.toml",
  debug: true
)

# Custom delimiter (default is __)
settings = AppSettings.load(
  env_prefix: "OPAL_",
  env_delimiter: "_"     # OPAL_DB_HOST instead of OPAL_DB__HOST
)

settings.db.host          # => from OPAL_DB__HOST or config or "localhost"
settings.cache.ttl        # => from OPAL_CACHE__TTL or config or 3600
settings.secret_key       # => required — raises SettingsError if missing everywhere
```

### How Groups Map to Sources

| Source | Flat field `debug` | Group field `db.host` |
|---|---|---|
| Env var | `OPAL_DEBUG` | `OPAL_DB__HOST` (default delimiter) |
| TOML | `debug = true` | `[db]` section, `host = "..."` |
| JSON | `{"debug": true}` | `{"db": {"host": "..."}}` |
| Explicit arg | `debug: true` | `db: DatabaseSettings.new(host: "...")` |

### Config File Example

```toml
# config.toml
debug = false
secret_key = "abc123"
log_level = "warn"

[db]
host = "db.prod.example.com"
port = 5432
name = "opal_prod"
pool_size = 10

[cache]
host = "cache.prod.example.com"
ttl = 7200
```

### Source Priority (Lowest to Highest)

1. Field defaults in model definition
2. Config file (TOML, JSON)
3. `.env` file
4. Environment variables
5. Explicit keyword arguments to `.load()`

### Type Coercion from Environment Variables

Environment variables are strings. Settings automatically coerces:
- `"true"` / `"false"` -> `Bool`
- `"5432"` -> `Int32`
- `"a,b,c"` -> `List(String)` (comma-separated)
- `"3.14"` -> `Float64`

### Rules

- `model X as Settings` makes the root a settings model with `.load()`.
- Nested groups are regular `model` — only the root loads from sources.
- Env delimiter defaults to `__`, configurable via `env_delimiter:`.
- Required fields (no default) raise `SettingsError` if missing from all sources.
- All validation runs after merging — same as regular models.
- Settings are immutable after loading, like all models.
- Supported config formats: TOML, JSON.

---

## Summary

| Feature | Decision |
|---|---|
| Keyword | `model` — dedicated construct, distinct from `class` |
| Immutability | Models are immutable, `.copy()` for modified copies |
| Inline validation | `where |v| expr` or `where guard_name` |
| Named guards | `where is_email`, `where min_length(3)` — reuses guard system |
| Multiple constraints | Comma-separated: `where is_required, is_email` |
| Cross-field validation | `validate do ... end` blocks |
| Validation order | Type check -> inline `where` -> `validate` blocks |
| Serialization | Built-in `to_dict`/`from_dict`, `to_json`/`from_json`, recursive |
| Settings | `model X as Settings` with `.load()` |
| Settings groups | Nested regular `model`s, root distributes values |
| Env delimiter | Configurable, defaults to `__` |
| Source priority | defaults < config file < .env < env vars < explicit args |

### New Keywords

| Keyword | Purpose |
|---|---|
| `model` | Define a validated, immutable data structure |
| `as Settings` | Make a model load from env/config sources |
| `where` (on `needs`) | Inline field validation |
| `validate` | Cross-field validation block |
