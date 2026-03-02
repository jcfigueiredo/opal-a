# Testing Framework

---

## Overview

Opal ships with a built-in testing framework as part of its "batteries included" philosophy. Tests use `.topl` files, are discovered automatically by `opal test`, and use `@describe`/`@test` macros (sugar for the `Test` module API). Mocking combines DI-based implementation swapping with a `Mock` helper for quick stubs.

---

## 1. Test Files

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

Convention: mirror the `src/` directory structure in `tests/`. This is a convention, not enforced — any `.topl` file in the project is a test file.

---

## 2. Test Structure

Tests use `@describe` and `@test` macros from the `OpalTest` subdomain. These are sugar for the underlying `Test` module API.

### Idiomatic Way (Macros)

```opal
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

### Nested Describe Blocks

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

### Equivalent Module API

The macros expand to `Test` module calls. The module API is available for programmatic test generation:

```opal
import Test

Test.describe("Math")
  Test.it("addition")
    Test.assert_eq(2 + 2, 4)
  end
end
```

---

## 3. Assertions

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

---

## 4. Lifecycle Hooks

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

### Hook Execution Order

For each test in the group:
1. `@before_all` — once before all tests in the group
2. `@before_each` — before each test
3. Test body runs
4. `@after_each` — after each test (runs even if test fails)
5. `@after_all` — once after all tests in the group

Nested `@describe` blocks inherit parent hooks. Inner `@before_each` runs after outer `@before_each`.

---

## 5. Mocking

Two approaches, used together:

### DI-Based (Primary)

Swap implementations via `needs` — this works naturally with Opal's DI system:

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

### Mock Helper (Quick Stubs)

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

### Mock API

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

Use the `@[test_only]` annotation to mark helpers that should only be available in test code — they won't be available in production builds:

```opal
# Mark helpers as test-only — they won't be available in production code
@[test_only]
def create_test_user(name::String) -> User
  User.new(name: name, email: f"{name}@test.com", age: 25)
end
```

---

## 6. Test Runner

### CLI

```
$ opal test                           # run all .topl files
$ opal test tests/math.topl           # run one file
$ opal test --filter "addition"       # filter by test name
$ opal test --filter "OrderService"   # filter by describe name
```

### Output

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

### Failure Output

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

---

## 7. Complete Example

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
