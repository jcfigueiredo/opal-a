# Future Improvements

Ice box of ideas and deferred decisions. Items here are not planned — they're captured so we don't lose them.

---

## CI / Coverage

- **Coverage ratchet gate**: Fail CI if coverage *decreases* from the previous run. Prevents regression without picking an arbitrary threshold. Add once baseline is stable.
- **Hard coverage threshold**: Set a floor (e.g., 70%) and fail CI if it drops below. Better suited after a round of targeted coverage improvements.
- **Codecov.io integration**: PR comments with diff coverage, historical trend tracking, README badge. Requires `CODECOV_TOKEN` secret. Worth adding if/when collaborators join.

## Structural Refactoring

- **Decompose `eval.rs`** (~7,500 lines): Extract `call_method` (1,400 lines), `eval_stmt` (1,000 lines), `eval_expr` (734 lines) into separate modules under `eval/`.
- **Separate `ControlFlow` from `EvalError`**: `Return`, `Break`, `Next` are control flow signals, not errors. A dedicated enum would make the distinction explicit and prevent silent propagation bugs.
- **Cache `error_class_id`**: The `Error` class is identified by string `"Error"` in 3 places with a linear scan. Cache `ClassId` as a field on `Interpreter` during `register_error_class()`.
- **Remove `.expect()` in `make_error_instance`**: Production code path at `eval.rs:4828`. Replace with cached `error_class_id` field.
- **Extract `StoredFunction` construction helper**: Copy-pasted 4 times across `eval_stmt` arms. A `funcdef_to_stored()` helper would eliminate duplication.
- **Fix `call_closure` default params**: `call_function` handles default params; `call_closure` silently ignores missing args. Behavioral inconsistency.
- **Move stored types to `opal-runtime`**: `StoredFunction`, `StoredClass`, `StoredInstance`, etc. are logically part of the interpreter's heap and could live alongside `Value`.
- **Re-export `PanicKind` from crate root**: Consumers can't match on `EvalError::Panic(PanicKind::TypeError, _)` without reaching into `eval::PanicKind`.
- **Remove `Value::is_truthy` magic enum index**: `EnumId(0), variant_index: 1` in `opal-runtime` encodes knowledge about the interpreter's builtin enum registration order. Leaky abstraction.

## Performance

- **Closure environment cloning**: `env.snapshot()` clones the entire scope stack on every closure creation *and* call. Reference-counted shared scopes or capture lists would reduce this dramatically.
- **`Dict` is `Vec<(String, Value)>`**: O(n) key lookup. `IndexMap` would give O(1) lookup with insertion-order preservation.
- **`items.clone()` in list methods**: `map`, `filter`, `reduce`, `sort`, `reverse` all clone the list before iterating. Restructure pattern match to move `obj` instead of borrowing.

## LSP

- **Parse-result caching**: LSP re-parses on every `goto_definition` and `document_symbol` request. Cache the parse result per document version.
- **LSP integration tests actually test LSP**: Current tests test the parser, not `goto_def::goto_definition` or symbol extraction.
- **Deduplicate `span_to_range`**: Identical implementations in `symbols.rs` and `goto_def.rs`.

## Test Infrastructure

- **Property-based tests**: `proptest` is declared as a workspace dep but unused. Either use it or remove it.
- **Unit tests for private functions**: `match_pattern`, `dispatch_multi`, `call_function`, `maybe_auto_throw`, `value_matches_type`, `eval_binary_op` have no isolated tests.
- **Deduplicate test helpers**: `temp_dir`/`write_file` duplicated between `eval.rs` and `loader.rs`.
- **Port spec tests to Rust integration test harness**: Write a `tests/spec_runner.rs` that reads `# expect:` headers and runs via `cargo test`. Gets parallel execution, better failure diagnostics, and automatic inclusion in coverage instrumentation. Currently spec tests run via `run_spec.sh` and are not counted in coverage.
