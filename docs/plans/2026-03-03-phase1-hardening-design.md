# Phase 1 Hardening Design

## Goal

Make Opal usable and debuggable before moving to module system redesign and developer tooling. Six independent improvements that fill gaps exposed during Slices 0-8.

## 1. REPL (`opal repl`)

Interactive read-eval-print loop using stdin. Reads a line, parses, evaluates, prints non-null results. Multi-line support: if a line ends mid-block (unclosed `def`, `if`, `class`, etc.), keep reading until balanced. Same interpreter instance across lines so variables persist. No line-editing library — zero new deps.

## 2. Error Messages with Line Numbers

Convert `Span` byte offsets to `line:col` using a `source_location(span, source)` helper. Update `EvalError` to carry optional `Span`. CLI formats errors as `file.opl:3:5: NameError: undefined variable 'x'`.

## 3. Dicts and Ranges

- **Dicts:** `{key: value}`, empty `{:}`. `Value::Dict`. Methods: `.get()`, `.keys()`, `.values()`, `.length()`, `.set()`.
- **Ranges:** `1..5` (exclusive), `1...5` (inclusive). `Value::Range`. Iterable in `for` loops. Method: `.to_list()`.
- **Tuples skipped** — `(expr)` grouping conflict needs careful disambiguation, deferred.

## 4. String Methods

`.split(sep)`, `.trim()`, `.contains(sub)`, `.replace(old, new)`, `.starts_with(s)`, `.ends_with(s)`, `.to_upper()`, `.to_lower()`, `.chars()`.

## 5. Closure Environment Capture

Snapshot the environment (clone scope chain) at closure creation. Call closures using captured env as base + new scope for params. Proper lexical closure semantics.

## 6. Actor Refactor

Replace `ClassId >= 1000` hack with `Value::ActorClass(ActorDefId)`. Clean separation between classes and actors in dispatch.
