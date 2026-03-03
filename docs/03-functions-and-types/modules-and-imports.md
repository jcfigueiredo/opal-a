# Modules & Imports

---

## Overview

Opal uses a hybrid file-to-module mapping: each `.opl` file implicitly defines a module matching its filename, and `module` blocks inside a file create nested namespaces. Imports are always absolute from the project root. Circular dependencies are compile-time errors.

---

## 1. File-to-Module Mapping

Each `.opl` file implicitly defines a module with a name derived from its filename (PascalCase). Subdirectories create module hierarchies. `module` blocks inside a file create nested sub-modules.

```
src/
  app.opl           -> App
  math.opl          -> Math
  math/
    vector.opl      -> Math.Vector
    matrix.opl      -> Math.Matrix
```

```opal
# file: src/math.opl
# Implicitly the Math module

PI = 3.14159

def abs(x: Number)
  if x < 0 then -x else x end
end

def max(a, b)
  if a > b then a else b end
end

module Trig           # Math.Trig (nested)
  def sin(x: Float64) -> Float64
    # ...
  end
end

Math.abs(-5)    # => 5
Math.PI         # => 3.14159
Math.Trig.sin(1.0)
```

Modules can contain classes:

```opal
# file: src/geometry.opl
# Implicitly the Geometry module

class Circle
  def init(radius: Float32)
    .radius = radius
  end

  def area()
    Math.PI * .radius ** 2
  end
end

c = Geometry.Circle.new(radius: 5.0)
c.area()  # => 78.539...
```

### Rules

- One top-level module per file, name derived from filename.
- `module` blocks inside a file create nested sub-modules.
- Subdirectories also create nested modules (both approaches work).
- If `src/math.opl` exists AND `src/math/` directory exists, the file defines the parent module and the directory holds child modules.

---

## 2. Import Syntax

Five forms, all using absolute paths from the project root:

```opal
import Math                          # whole module -- access as Math.abs(), Math.PI
import Math.Vector                   # nested module -- access as Math.Vector.dot()
from Math import abs, max            # selective -- abs() and max() available directly
import Math.Vector as Vec            # aliased -- access as Vec.dot()
from Math import abs, max as maximum # selective + alias -- abs() and maximum()
```

Multi-line selective imports use parentheses:

```opal
from Math import (
  sin, cos, tan,
  sqrt, abs, max,
  PI, E
)
```

### What "import" Does

- `import Module` loads the module and makes its public symbols accessible via `Module.name`.
- `from Module import name` brings `name` directly into the current scope (no prefix needed).
- `as` renames for the current scope only.
- All paths are absolute from the project root -- no relative imports.

### Collision Rule

If two selective imports bring the same name into scope, it's a compile-time error. Fix by using aliased import.

```opal
from Math import max
from Stats import max        # COMPILE ERROR -- 'max' already imported

from Math import max as math_max
from Stats import max as stats_max  # ok
```

---

## 3. Re-exports

Modules can re-export symbols from their imports using `export`. This lets library authors expose a clean API without leaking internal structure.

```opal
# file: src/opal_web.opl
import OpalWeb.Router
import OpalWeb.Middleware
import OpalWeb.Response

# Re-export specific symbols
export get, post, put, delete from Router
export use from Middleware
export json, html, redirect from Response
```

```opal
# Consumer just imports the top-level module
import OpalWeb
OpalWeb.get("/", handler)     # works -- re-exported from Router
OpalWeb.json({status: "ok"})  # works -- re-exported from Response
```

### Rules

- `export names from Module` re-exports specific symbols from an imported module.
- Re-exported symbols appear as if defined in the exporting module.
- Only public symbols can be re-exported.
- By default, all top-level `def`, `class`, `module`, `enum`, `model`, `protocol` in a file are public. Use `private` to hide.

---

## 4. Circular Dependencies

Circular imports are a compile-time error. If A imports B and B imports A (directly or transitively), the compiler rejects the program.

```opal
# a.opl
import B        # B imports A -> COMPILE ERROR: circular dependency A <-> B

# Fix: extract shared types into a third module
# shared.opl -- types both need
# a.opl imports Shared
# b.opl imports Shared
# No cycle
```

### Loading Order

- The compiler builds a dependency graph from imports.
- Modules are loaded in topological order (dependencies before dependents).
- Circular dependency = no valid topological order = compile-time error with a clear message showing the cycle.

### Error Message Example

```
Compile error: circular dependency detected
  A imports B (a.opl:1)
  B imports A (b.opl:3)
Extract shared definitions into a separate module to break the cycle.
```

---

## 5. Packages

A package is a distributable unit -- a directory with a manifest file. A module is a namespace within a package. Every package has a root module matching the package name.

```
my_web_app/
  opal.toml              # package manifest
  src/
    my_web_app.opl       # root module (MyWebApp)
    routes.opl            # MyWebApp.Routes
    models/
      user.opl            # MyWebApp.Models.User
```

```toml
# opal.toml
[package]
name = "my_web_app"
version = "0.1.0"

[dependencies]
opal_web = "1.2"
opal_db = "0.5"
```

### Rules

- A package = directory with `opal.toml` + `src/` directory.
- External packages are imported by their package name: `import OpalWeb`.
- The package manager resolves package name to installed source.
- Within a package, all imports are absolute from that package's root: `import MyWebApp.Routes`.
- Cross-package imports use the dependency's package name: `import OpalWeb.Router`.

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

---

## 7. Design Rationale

### Why Absolute-Only Imports?

Relative imports (like `from . import foo`) create fragile code that breaks when files move and makes dependency graphs harder to reason about. Absolute imports from the project root are unambiguous -- you always know where a symbol comes from. This is the same choice made by Go and Rust.

### Why No Circular Dependencies?

Circular dependencies make compilation order undefined and create tight coupling between modules. Banning them forces cleaner architecture: extract shared types into a separate module. The compiler provides a clear error message showing the cycle, making it easy to fix.

### Why Hybrid File-Module Mapping?

Pure file-based mapping (one module per file, no nesting) is too rigid for large modules. Pure block-based mapping (all modules declared explicitly) adds boilerplate. The hybrid approach uses files for the common case and `module` blocks for nesting within a file, giving flexibility without ceremony.

---

## Summary

| Feature | Decision |
|---|---|
| File mapping | Hybrid -- file = module, `module` blocks for nesting |
| Import forms | Whole, selective, aliased, selective+alias |
| Paths | Absolute only -- no relative imports |
| Collisions | Compile-time error on duplicate names in scope |
| Re-exports | `export names from Module` for clean public APIs |
| Circular deps | Compile-time error -- extract shared code to break cycles |
| Packages | Directory with `opal.toml` + `src/`, root module matches package name |
| Infrastructure | Regular packages with `ServiceProvider[C]`, used in topology files |

### New Keywords

| Keyword | Purpose |
|---|---|
| `import` | Load a module and bring symbols into scope |
| `from` (in import) | Selective import: `from Module import name` |
| `as` (in import) | Alias an imported module or symbol |
| `export` | Re-export symbols from an imported module |
