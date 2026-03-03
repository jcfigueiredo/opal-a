# Standard Library

Opal ships with a standard library organized into modules:

| Module | Purpose |
|---|---|
| `IO` | Standard input/output: `print()`, `println()`, `read_line()`, `read_all()` |
| `File` | File operations: `read()`, `write()`, `exists?()`, `delete()`, `list_dir()` |
| `Net` | HTTP client/server, TCP/UDP sockets |
| `Math` | Mathematical functions: `abs()`, `max()`, `min()`, `sqrt()`, `sin()`, `cos()`, constants (`PI`, `E`) |
| `Collections` | Advanced data structures: Set, Queue, Stack, PriorityQueue |
| `String` | String manipulation: `split()`, `join()`, `trim()`, `replace()`, `upper()`, `lower()` |
| `Time` | Date, time, duration: `Time.now()`, `Time.parse()`, `Duration`, formatting |
| `JSON` | JSON parsing and generation: `JSON.parse()`, `JSON.generate()`, streaming |
| `Test` | Built-in test framework — `@describe`, `@test`, assertions, lifecycle hooks |
| `Mock` | Mock creation for tests — `Mock.new(Protocol)`, stubs, call verification |
| `Spec` | Specification pattern base classes |
| `Container` | Optional dependency injection container for large apps |
| `Iter` | `Iterable` and `Iterator(T)` protocols, lazy sequences |
| `Option` | `Option(T)` enum — `Some(value)` or `None` for explicit nullable handling; used by `Iterator(T)` |
| `Result` | `Result(T, E)` enum — `Ok(value)` or `Err(error)` for error handling |
| `Settings` | Base for `settings model` definitions — env/config/file loading with source priority |
| `Reflect` | Runtime introspection: `annotations()`, `field_annotations()`, `typeof()`, `methods()` |

## Usage Examples

```opal
import IO
import File
import JSON

# Read a JSON config file
content = File.read("config.json")
config = JSON.parse(content)
IO.print(f"Loaded {config.keys().length} settings")
```

```opal
import Test

Test.describe("Math operations")
  Test.it("adds two numbers")
    Test.assert_eq(2 + 2, 4)
  end

  Test.it("handles negative numbers")
    Test.assert_eq(-1 + 1, 0)
  end
end
```
