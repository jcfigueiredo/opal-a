# Cache `error_class_id` on Interpreter — Design Spec

**Date:** 2026-03-16
**Status:** Draft
**Scope:** Replace stringly-typed `"Error"` class lookups with cached `ClassId`

## Problem

The `Error` class is identified at runtime by the string `"Error"` in 5 identity-check locations across `eval.rs`, plus 2 registration sites that write the name. This has three consequences:

1. **Production panic risk:** `make_error_instance()` uses `.expect("Error class not registered")` after a linear scan. If the invariant breaks, the interpreter panics with no recovery.
2. **O(n) lookup per error:** `make_error_instance()` scans the entire `classes` Vec on every call. In error-heavy code (e.g., `?` operator), this is O(n_classes) per error.
3. **Stringly-typed identity:** The `"Error"` string is scattered across 5 identity-check call sites. A typo or rename would silently break error handling with no compiler help.

## Design

Add an `error_class_id: ClassId` field to `Interpreter<W>`.

### Changes

**Struct field:**
- Add `error_class_id: ClassId` to `Interpreter<W>` (after `container_registrations`)
- Initialize to `ClassId(usize::MAX)` in both constructors as an intentionally invalid sentinel — would cause an obvious OOB panic if used before `register_error_class()` runs. Overwritten immediately during construction.

**Registration:**
- In `register_error_class()`, after pushing the `StoredClass`, set `self.error_class_id = class_id`

**5 identity-check sites replaced:**

| Location | Before | After |
|---|---|---|
| `make_error_instance` (line ~4824) | Linear scan + `.expect()` | `self.error_class_id` directly |
| `is_error_instance` (line ~4846) | `c.name == "Error"` | `inst.class_id == self.error_class_id` |
| `match_pattern` (line ~2713) | `c.name == "Error"` | `inst.class_id == self.error_class_id` |
| `format_value_display` (line ~5009) | `class.name == "Error"` | `inst.class_id == self.error_class_id` |
| `format_value` (line ~5049) | `class.name == "Error"` | `inst.class_id == self.error_class_id` |

**Intentionally unchanged `"Error"` strings (3 sites):**

| Location | Why unchanged |
|---|---|
| `register_error_class` (line ~712) | `StoredClass.name` — needed for display (`<Error instance>` formatting) |
| `register_error_class` (line ~718) | `env.set("Error", ...)` — needed for user-level `Error(...)` calls |
| `eval_call` (line ~2949) | `"Error" \| "Err"` name-based dispatch — matches what the user typed, not class identity |

Note: The `StoredEnumVariant` named `"Error"` inside `register_builtin_enums()` (line ~312) is the legacy `Result` enum variant name, not a class identity check. It is unrelated and unchanged.

### Testing

Pure internal refactor — all existing tests cover the affected paths:
- Error construction: `error_class_has_message_and_cause`
- Auto-throw: `auto_throw_error_*` tests
- Pattern matching: `catch_error_*` tests
- Formatting: `error_display_*` tests
- Spec tests: error handling suite in `tests/spec/`

No new tests needed. Verify with `cargo test --all` + `./tests/run_spec.sh`.

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Storage location | Field on `Interpreter` | Explicit, follows existing field patterns, no ordering assumptions |
| Scope | All 5 identity checks | Consistency prevents drift; 3 name-based sites intentionally unchanged |
| Sentinel value | `ClassId(usize::MAX)` | Obvious OOB if used before registration, unlike `ClassId(0)` which silently aliases the real ID |
| New tests | None | All paths already tested; pure refactor |
