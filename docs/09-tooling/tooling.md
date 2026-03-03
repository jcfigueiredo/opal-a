# Tooling

## 1. Overview

Opal provides a unified CLI (`opal`) for running, testing, formatting, linting, documenting, and managing packages. All tooling is built-in and follows the "batteries included" philosophy.

---

## 2. Running Programs

```
$ opal run src/app.opl
  Hello, world!

$ opal run src/app.opl --arg1 value1
  # command-line arguments passed to the program
```

---

## 3. Testing

Opal ships with a built-in testing framework. Tests use `.topl` files, are discovered automatically by `opal test`, and use `@describe`/`@test` macros (sugar for the `Test` module API). Mocking combines DI-based implementation swapping with a `Mock` helper for quick stubs.

### Test Files

Test files use the `.topl` extension and are discovered automatically by `opal test`.

```
my_app/
  opal.toml
  src/
    math.opl
    order_service.opl
  tests/
    math.topl
    order_service.topl
```

Convention: mirror the `src/` directory structure in `tests/`. This is a convention, not enforced -- any `.topl` file in the project is a test file.

### Test Structure

Tests use `@describe` and `@test` macros from the `OpalTest` subdomain. These are sugar for the underlying `Test` module API.

#### Idiomatic Way (Macros)

```opal
# tests/math.topl
import OpalTest

@describe "Math" do
  @test "addition" do
    assert_eq(2 + 2, 4)
  end

  @test "negative numbers" do
    assert_eq(-1 + 1, 0)
  end
end
```

#### Nested Describe Blocks

```opal
@describe "OrderService" do
  @describe "place_order" do
    @test "saves to database" do
      # ...
    end

    @test "sends confirmation email" do
      # ...
    end
  end

  @describe "cancel_order" do
    @test "refunds payment" do
      # ...
    end
  end
end
```

#### Equivalent Module API

The macros expand to `Test` module calls. The module API is available for programmatic test generation:

```opal
import Test

Test.describe("Math")
  Test.it("addition")
    Test.assert_eq(2 + 2, 4)
  end
end
```

### Assertions

```opal
# Equality
assert_eq(actual, expected)         # passes if actual == expected
assert_ne(actual, expected)         # passes if actual != expected

# Boolean
assert_true(expr)                   # passes if expr is true
assert_false(expr)                  # passes if expr is false

# Exceptions
assert_raises(ValidationError) do   # passes if block raises ValidationError
  validate(bad_input)
end

# Pattern matching
assert_match(Result.Ok(_), result)  # passes if result matches the pattern
```

All assertions produce clear failure messages with expected vs actual values:

```
FAIL: addition (tests/math.topl:5)
  assert_eq failed:
    expected: 5
    actual:   4
```

### Lifecycle Hooks

Four hooks for setup and teardown, at both group and per-test levels:

```opal
@describe "Database" do
  @before_all do
    .db = TestDB.create()
  end

  @after_all do
    .db.destroy()
  end

  @before_each do
    .db.begin_transaction()
  end

  @after_each do
    .db.rollback()
  end

  @test "insert" do
    .db.insert({name: "test"})
    assert_eq(.db.count(), 1)
  end

  @test "delete" do
    .db.insert({name: "test"})
    .db.delete(1)
    assert_eq(.db.count(), 0)
  end
end
```

#### Hook Execution Order

For each test in the group:
1. `@before_all` -- once before all tests in the group
2. `@before_each` -- before each test
3. Test body runs
4. `@after_each` -- after each test (runs even if test fails)
5. `@after_all` -- once after all tests in the group

Nested `@describe` blocks inherit parent hooks. Inner `@before_each` runs after outer `@before_each`.

### Mocking

Two approaches, used together:

#### DI-Based (Primary)

Swap implementations via `needs` -- this works naturally with Opal's DI system:

```opal
# Hand-written mock
class FakeDB implements Database
  def init()
    .saved = []
  end

  def save(record) -> Bool
    .saved.push(record)
    true
  end

  def find(id::Int32) -> Record?
    null
  end
end

test_service = OrderService.new(
  db: FakeDB.new(),
  mailer: FakeMailer.new()
)
```

#### Mock Helper (Quick Stubs)

The `Mock` module creates protocol-conforming stubs without writing a full class:

```opal
import Mock

mock_db = Mock.new(Database)
mock_db.stub(:save, true)            # save() always returns true
mock_db.stub(:find, |id| null)       # find() uses a closure

service = OrderService.new(db: mock_db, mailer: mock_mailer)
service.place_order(order)

# Verify calls
assert_true(mock_db.called?(:save))
assert_eq(mock_db.call_count(:save), 1)
assert_eq(mock_db.last_args(:save), [order])
```

#### Mock API

```opal
mock = Mock.new(Protocol)            # create mock conforming to Protocol
mock.stub(:method, return_value)     # stub with fixed return value
mock.stub(:method, |args| expr)      # stub with closure
mock.called?(:method) -> Bool        # was method called?
mock.call_count(:method) -> Int32    # how many times?
mock.last_args(:method) -> List      # arguments of last call
mock.reset!()                        # clear all call records
```

### Test-Only Helpers

Use the `@[test_only]` annotation to mark helpers that should only be available in test code -- they won't be available in production builds:

```opal
# Mark helpers as test-only -- they won't be available in production code
@[test_only]
def create_test_user(name::String) -> User
  User.new(name: name, email: f"{name}@test.com", age: 25)
end
```

### Test Runner

#### CLI

```
$ opal test                           # run all .topl files
$ opal test tests/math.topl           # run one file
$ opal test --filter "addition"       # filter by test name
$ opal test --filter "OrderService"   # filter by describe name
```

#### Output

```
$ opal test
  Math
    ✓ addition
    ✓ negative numbers
  OrderService
    place_order
      ✓ saves to database
      ✓ sends confirmation email
    cancel_order
      ✓ refunds payment

5 tests, 5 passed, 0 failed (0.03s)
```

#### Failure Output

```
$ opal test
  Math
    ✓ addition
    ✗ subtraction
      FAIL: subtraction (tests/math.topl:9)
        assert_eq failed:
          expected: 3
          actual:   2

2 tests, 1 passed, 1 failed (0.02s)
```

### Complete Test Example

```opal
# tests/order_service.topl
import OpalTest
import Mock

@describe "OrderService" do
  @before_each do
    .mock_db = Mock.new(Database)
    .mock_db.stub(:save, true)
    .mock_mailer = Mock.new(Mailer)
    .mock_mailer.stub(:send_confirmation, :ok)
    .service = OrderService.new(db: .mock_db, mailer: .mock_mailer)
  end

  @describe "place_order" do
    @test "saves the order" do
      order = Order.new(id: 1, total: 99.99)
      .service.place_order(order)
      assert_true(.mock_db.called?(:save))
      assert_eq(.mock_db.call_count(:save), 1)
    end

    @test "sends confirmation email" do
      order = Order.new(id: 1, total: 99.99)
      .service.place_order(order)
      assert_true(.mock_mailer.called?(:send_confirmation))
    end

    @test "fails on invalid order" do
      assert_raises(ValidationError) do
        .service.place_order(null)
      end
    end
  end
end
```

---

## 4. Project Scaffolding

```
$ opal init --type lib MyCoolProject
       create  MyCoolProject/.gitignore
       create  MyCoolProject/LICENSE
       create  MyCoolProject/README.md
       create  MyCoolProject/src/MyCoolProject.opl
       create  MyCoolProject/tests/MyCoolProject.topl
Initialized empty Git repository in ~/MyCoolProject/.git/
```

Templates are supported: `opal init --type web MyWebApp`, `opal init --type cli MyCLI`.

---

## 5. Documentation Generation

```
$ opal docs MyCoolProject
       created  docs/libs/MyCoolProject.md
Documentation created in docs/ [should I publish to GHPages?].
```

---

## 6. Linter

```
$ opal lint src/
  src/main.opl:12:5  warning  unused variable 'temp'
  src/main.opl:28:1  error    unreachable code after return
  2 issues (1 error, 1 warning)
```

---

## 7. Formatter

Opinionated, zero-configuration formatter. One canonical style -- no debates.

```
$ opal fmt src/
  Formatted 12 files

$ opal fmt --check src/
  src/math.opl: needs formatting
  1 file needs formatting
```

`opal fmt` enforces a single consistent style. There are no configuration options -- this aligns with Opal's "one explicit way" philosophy.

---

## 8. Package Manager

Integrated package management, inspired by Poetry/Cargo.

### Package Manifest (`opal.toml`)

Every package has an `opal.toml` at its root:

```toml
# opal.toml
[package]
name = "my_app"
version = "1.0.0"
author = "claudio"
license = "MIT"
opal = ">=0.1.0"       # minimum Opal version

[dependencies]
opal_web = "~1.2"       # compatible with 1.2.x
opal_db = "^0.5"        # compatible with 0.x

[dev-dependencies]
opal_bench = "0.3"      # test/bench only
```

### Version Specifiers

| Specifier | Meaning | Example |
|---|---|---|
| `"1.2.3"` | Exact version | Only 1.2.3 |
| `"~1.2"` | Compatible minor | >=1.2.0, <1.3.0 |
| `"^1.2"` | Compatible major | >=1.2.0, <2.0.0 |
| `">=1.0"` | Minimum version | 1.0.0 or higher |

### Lock File

`opal.lock` is auto-generated by `opal pkg install` and records exact resolved versions. Commit it to version control for reproducible builds.

### Commands

```
$ opal pkg add http_server@1.2
  Added http_server 1.2.0 to dependencies
$ opal pkg install
  Installing 3 packages...
  Done.
$ opal pkg remove opal_web
  Removed opal_web from dependencies
```

---

## 9. DSL Configuration with `with`

```opal
# Using with for DSL-style configuration blocks
import Nginx

my_site = Nginx.create with {
  user:              "www www",
  worker_processes:  5,
  error_log:         "logs/error.log",
  pid:               "logs/nginx.pid"
}

my_site.http with {
  index:        "index.html index.htm index.php",
  default_type: "application/octet-stream"
}

my_site.http.server with {
  listen:      80,
  server_name: "domain.com",
  access_log:  "logs/domain.access.log main"
}

my_site.serve!
```

The `with` keyword is syntactic sugar for named argument passing. `expr with { key: value }` is equivalent to `expr(key: value)`. The dict keys become parameter names, the values become argument values. This provides a visual block style that reads better for configuration-heavy calls. Object creation uses `.new()` with named arguments; string interpolation uses f-strings (or t-strings for safe templating).

---

## 10. Design Rationale

### Why Built-in Testing?

Opal includes testing as a first-class tool rather than deferring to third-party frameworks. This ensures consistent conventions across the ecosystem and removes the decision fatigue of choosing a test library.

### Why `.topl` Extension?

The `.topl` extension (Test Opal) makes test files immediately identifiable and allows the toolchain to auto-discover them without configuration. This mirrors the convention used by other languages with dedicated test file patterns.

### Why Macros for Test Structure?

`@describe` and `@test` are macros rather than language keywords. This keeps the core language small while providing ergonomic test syntax. The underlying `Test` module API remains available for programmatic test generation or custom frameworks.

### Why DI-Based Mocking?

Opal's `needs` keyword makes dependency injection natural. Since services already declare their dependencies explicitly, swapping implementations for testing requires no special mocking framework -- just pass a different implementation. The `Mock` helper supplements this for cases where writing a full fake class is overkill.

---

## 11. CLI Summary

| Command | Purpose |
|---|---|
| `opal run` | Run a program |
| `opal test` | Run tests (`.topl` files) |
| `opal fmt` | Format code (opinionated, zero-config) |
| `opal lint` | Lint code for errors and warnings |
| `opal docs` | Generate documentation |
| `opal init` | Scaffold a new project |
| `opal pkg add` | Add a dependency |
| `opal pkg install` | Install all dependencies |
| `opal pkg remove` | Remove a dependency |

---

## Summary

| Feature | Decision |
|---|---|
| Test files | `.topl` extension, auto-discovered by `opal test` |
| Structure | `@describe`/`@test` macros (sugar for `Test` module API) |
| Assertions | `assert_eq`, `assert_ne`, `assert_true`, `assert_false`, `assert_raises`, `assert_match` |
| Lifecycle | `@before_all`, `@after_all`, `@before_each`, `@after_each` |
| Mocking | DI-based swap + `Mock.new(Protocol)` helper for quick stubs |
| Runner | `opal test`, `--filter` for name filtering |
| Nesting | `@describe` blocks nest, hooks inherit |
| Formatter | Zero-config, one canonical style |
| Package manager | `opal.toml` manifest, `opal.lock` for reproducibility |
