# Iterators

---

## Overview

Two protocols -- `Iterable` (the thing you iterate over) and `Iterator` (the cursor) -- form the foundation for `for ... in` loops and collection methods like `map`, `filter`, and `reduce`. Any class implementing `Iterable` plugs into the entire collection pipeline.

---

## 1. The Iterator Protocols

```opal
protocol Iterable
  def iter() -> Iterator
end

protocol Iterator[T]
  def next() -> Option[T]
end
```

`Iterable` returns an `Iterator`, and `Iterator.next()` returns `Option[T]` -- `Some(value)` for the next element, `None` when exhausted.

---

## 2. Custom Collections

Any class can become iterable by implementing the protocol pair. Here, `FileLines` reads lines lazily from a file:

```opal
class FileLines implements Iterable
  needs path: String

  def iter()
    FileLinesIterator.new(file: File.open(.path))
  end
end

class FileLinesIterator implements Iterator[String]
  needs file: File

  def next() -> Option[String]
    line = .file.read_line()
    if line == null
      Option.None
    else
      Option.Some(line)
    end
  end
end

# Works with for-in
for line in FileLines.new(path: "data.txt")
  print(line)
end

# Works with collection methods
FileLines.new(path: "data.txt")
  .map(|line| line.trim())
  .filter(|line| line.length > 0)
```

---

## 3. Lazy Infinite Sequences

Because `Iterator.next()` returns `Option[T]`, iterators can be infinite -- they never return `None`. Consumers use methods like `take` to limit output.

```opal
class Counter implements Iterable
  needs start: Int32

  def iter()
    CounterIterator.new(current: .start)
  end
end

class CounterIterator implements Iterator[Int32]
  needs current: Int32

  def next() -> Option[Int32]
    value = .current
    .current += 1
    Option.Some(value)  # never exhausted
  end
end

for n in Counter.new(start: 0).take(5)
  print(n)  # 0, 1, 2, 3, 4
end
```

---

## 4. Design Rationale

### Why Two Protocols?

Separating `Iterable` and `Iterator` lets one collection produce multiple independent iterators. A `List` can be iterated by two nested loops simultaneously because each `iter()` call returns a fresh cursor. This mirrors the design in Java, Python, and Rust, adapted to Opal's protocol system.

### Why `Option[T]` for `next()`?

Using `Option[T]` instead of a sentinel value or exception:
- Makes exhaustion explicit in the type system.
- Enables exhaustive matching on iterator results.
- Composes naturally with Opal's `Option`/`Result` ecosystem.

### What the Iterator Protocol Unlocks

| What You Can Write | How |
|---|---|
| Custom collections (trees, graphs) | Implement `Iterable` + `Iterator` |
| File/network streaming | Lazy iterators that read on demand |
| Infinite sequences | Iterators that never return `None` |
| Collection pipeline (`map`, `filter`, `reduce`) | Works on any `Iterable` automatically |

---

## 5. Rules

- Any class implementing `Iterable` works with `for ... in`.
- `Iterator.next()` returns `Option[T]` -- `Some(value)` for the next element, `None` when exhausted.
- Built-in types (`List`, `Dict`, `Range`, `String`) all implement `Iterable`.
- Collection methods (`map`, `filter`, `reduce`, `take`, `zip`) work on any `Iterable`.
- `iter()` is called automatically by `for ... in`.

---

## Summary

| Feature | Decision |
|---|---|
| Iterable protocol | `def iter() -> Iterator` -- returns a cursor |
| Iterator protocol | `def next() -> Option[T]` -- returns next element or `None` |
| For-in integration | Any `Iterable` works with `for ... in` automatically |
| Collection methods | `map`, `filter`, `reduce`, `take`, `zip` work on any `Iterable` |
| Built-in support | `List`, `Dict`, `Range`, `String` all implement `Iterable` |
| Lazy sequences | Iterators can be infinite; consumers use `take` to limit |
