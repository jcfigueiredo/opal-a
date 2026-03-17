# Future Improvements

Ice box of ideas and deferred decisions. Items here are not planned — they're captured so we don't lose them.

---

## CI / Coverage

- **Coverage ratchet gate**: Fail CI if coverage *decreases* from the previous run. Prevents regression without picking an arbitrary threshold. Add once baseline is stable.
- **Hard coverage threshold**: Set a floor (e.g., 70%) and fail CI if it drops below. Better suited after a round of targeted coverage improvements.
- **Codecov.io integration**: PR comments with diff coverage, historical trend tracking, README badge. Requires `CODECOV_TOKEN` secret. Worth adding if/when collaborators join.

## Structural Refactoring

- ~~**Decompose `call_method`** (1,400 → 178 lines): Extracted into `string_methods.rs`, `dict_methods.rs`, `list_methods.rs`, `instance_methods.rs`, `class_methods.rs`.~~ DONE
- ~~**Separate `ControlFlow` from `EvalError`**: Nested `FlowSignal` enum inside `EvalError::Flow(...)`.~~ DONE
- ~~**Cache `error_class_id`**: Cached as `ClassId` field on `Interpreter`, set during registration.~~ DONE
- ~~**Remove `.expect()` in `make_error_instance`**: Replaced with cached `error_class_id` field.~~ DONE
- **Continue decomposing `eval.rs`** (~6,300 lines remaining): Extract `eval_stmt` (~1,000 lines) and `eval_expr` (~734 lines) into separate modules, following the same split-`impl`-block pattern used for `call_method`.
- **Extract `StoredFunction` construction helper**: Copy-pasted 4 times across `eval_stmt` arms. A `funcdef_to_stored()` helper would eliminate duplication.
- ~~**Fix `call_closure` arity validation**: Added argument count check; fixed HTTP handler to match closure param counts.~~ DONE
- **Move stored types to `opal-runtime`**: `StoredFunction`, `StoredClass`, `StoredInstance`, etc. are logically part of the interpreter's heap and could live alongside `Value`.
- ~~**Re-export `PanicKind` from crate root**: Now re-exported from `opal-interp` crate root.~~ DONE
- ~~**Remove `Value::is_truthy` magic enum index**: Replaced with named constants `RESULT_ENUM_ID`, `RESULT_ERROR_VARIANT`, `OPTION_ENUM_ID`.~~ DONE

## Performance

- **Closure environment cloning**: `env.snapshot()` clones the entire scope stack on every closure creation *and* call. Reference-counted shared scopes or capture lists would reduce this dramatically.
- **`Dict` is `Vec<(String, Value)>`**: O(n) key lookup. `IndexMap` would give O(1) lookup with insertion-order preservation.
- **`items.clone()` in list methods**: `map`, `filter`, `reduce`, `sort`, `reverse` all clone the list before iterating. Restructure pattern match to move `obj` instead of borrowing.

## LSP

- **Parse-result caching**: LSP re-parses on every `goto_definition` and `document_symbol` request. Cache the parse result per document version.
- **LSP integration tests actually test LSP**: Current tests test the parser, not `goto_def::goto_definition` or symbol extraction.
- ~~**Deduplicate `span_to_range`**: Extracted to shared `utils.rs` module.~~ DONE

## Test Infrastructure

- **Property-based tests**: `proptest` is declared as a workspace dep but unused. Either use it or remove it.
- **Unit tests for private functions**: `match_pattern`, `dispatch_multi`, `call_function`, `maybe_auto_throw`, `value_matches_type`, `eval_binary_op` have no isolated tests.
- **Deduplicate test helpers**: `temp_dir`/`write_file` duplicated between `eval.rs` and `loader.rs`.
- **Port spec tests to Rust integration test harness**: Write a `tests/spec_runner.rs` that reads `# expect:` headers and runs via `cargo test`. Gets parallel execution, better failure diagnostics, and automatic inclusion in coverage instrumentation. Currently spec tests run via `run_spec.sh` and are not counted in coverage.
