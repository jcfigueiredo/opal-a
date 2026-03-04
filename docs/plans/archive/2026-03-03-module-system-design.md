# Module System Redesign

## Goal

Replace the Python-style `from X import Y` with a Gleam-inspired `import X.{Y}` syntax. Add file-based module loading, configurable resolution, and flexible export control.

## Import Syntax

Single `import` keyword with braces for selective imports:

```opal
import Math                          # whole module → Math.abs()
import Math.{abs, max}               # selective → abs(), max()
import Math.Vector as Vec            # aliased → Vec.dot()
import Math.{abs, max as maximum}    # selective + alias
```

Multi-line:
```opal
import Math.{
  abs,
  max,
  Vector as Vec
}
```

## File-to-Module Mapping

- `math.opl` → `Math` module
- `math/vector.opl` → `Math.Vector` module
- `module` blocks inside files create nested sub-modules
- Filename convention: `snake_case.opl` → `SnakeCase`

## File Resolution Order

1. Relative to the importing file's directory
2. Project root `src/` directory (if `opal.toml` exists)
3. `OPAL_PATH` environment variable (colon-separated directories)

## Visibility / Export Control

Three tiers, progressively more explicit:

### Tier 1: Context-sensitive defaults (no annotations)
- Scripts (no `opal.toml`): everything public
- Packages (`opal.toml`): everything private (Phase 2)
- Phase 1: public by default everywhere

### Tier 2: `pub` annotation on items
```opal
pub def abs(x) ...       # exported
pub class Vector ...     # exported
def helper(x) ...        # private
```

### Tier 3: `export {}` block (authoritative list)
```opal
export {abs, Vector}

def abs(x) ...           # exported (in list)
class Vector ...         # exported (in list)
def helper(x) ...        # private (not in list)
```

**Rules:**
- `export {}` takes precedence over everything — only listed names are public
- Without `export {}`, `pub` controls visibility
- Without either, defaults apply

### Re-exports
```opal
pub import Math.Trig.{sin, cos}   # re-exports sin, cos from this module
```

## Circular Dependencies

Track "currently loading" set. If a module is already being loaded when encountered again → runtime error with clear message.

## Migration

- `from X import Y` → `import X.{Y}` (syntax change)
- Existing `from Shapes import Circle` in spec tests updated
- Parser accepts both forms during transition (old form deprecated)

## Phase 1 Scope

- New `import X.{Y}` syntax (parser + interpreter)
- File-based module loading (resolve .opl files)
- `pub` keyword on definitions
- `export {}` blocks
- Update existing tests to new syntax
- Keep `from X import Y` working (deprecated but functional)

## Deferred

- Package mode (opal.toml → private by default)
- Package manager / dependency resolution
- `OPAL_PATH` search (just relative + project root for now)
