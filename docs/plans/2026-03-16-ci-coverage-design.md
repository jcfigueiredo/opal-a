# CI Hardening + Code Coverage — Design Spec

**Date:** 2026-03-16
**Status:** Draft
**Scope:** CI pipeline restructuring, code coverage enablement, test suite integration

## Problem

The CI pipeline has three gaps:

1. **118 spec tests are not run in CI.** They pass locally but CI doesn't execute `tests/run_spec.sh`. A parser regression could merge undetected.
2. **34 tree-sitter corpus tests are not run in CI.** Grammar regressions are invisible until someone manually runs `pnpm test` in `tree-sitter-opal/`.
3. **No code coverage.** There's no way to identify untested code paths. The codebase has ~234 unit tests but no coverage baseline to measure against.

## Design

### Architecture: Three Parallel CI Jobs

```
ci.yml
├── rust                          (existing, enhanced)
│   ├── cargo fmt --check
│   ├── cargo clippy -D warnings
│   ├── cargo build --all-targets
│   ├── cargo test --all
│   └── ./tests/run_spec.sh      ← NEW
│
├── coverage                      ← NEW
│   ├── rust-toolchain (stable + llvm-tools-preview)
│   ├── install cargo-llvm-cov
│   ├── cargo llvm-cov --no-report  (run tests once)
│   ├── cargo llvm-cov report --html / --lcov
│   └── upload artifacts (HTML report + lcov.info)
│
└── tree-sitter                   ← NEW
    ├── setup Node.js 22 + pnpm
    └── cd tree-sitter-opal && pnpm install && pnpm test
```

All three jobs run in parallel. No job depends on another.

### Job Details

#### `rust` job (enhanced)

Add two steps after `cargo test --all`:

1. `cargo build --release --bin opal` — spec tests need the release binary for speed
2. `OPAL_BIN="./target/release/opal" ./tests/run_spec.sh` — run all 118 spec tests using the release binary

The `OPAL_BIN` env var overrides the script's default (`cargo run --quiet --bin opal --`) so the pre-built release binary is used. The script exits non-zero on failure, so CI will catch regressions.

#### `coverage` job (new)

- **Toolchain:** `dtolnay/rust-toolchain@stable` with `components: llvm-tools-preview`
- **Cache:** `Swatinem/rust-cache@v2`
- **Tool:** `cargo-llvm-cov` (installed via `taiki-e/install-action@cargo-llvm-cov`)
- **Commands (single test run, two report passes):**
  - `cargo llvm-cov --all-features --workspace --no-report` — runs instrumented tests, caches coverage data
  - `cargo llvm-cov report --html` — generates HTML report at `target/llvm-cov/html/`
  - `cargo llvm-cov report --lcov --output-path lcov.info` — generates machine-readable lcov
  - Note: `--html` and `--lcov` cannot be combined in one invocation; the `--no-report` + `report` pattern avoids running tests twice
- **Artifacts:** Upload both HTML report directory and `lcov.info` as GitHub Actions artifacts (retained 30 days)
- **No gate:** Coverage is report-only. No threshold enforcement.

Note: Coverage captures unit tests only (the `cargo test` suite). Spec tests run via shell script are not instrumented. Porting spec tests to a Rust harness (which would include them in coverage) is deferred to `future-improvements.md`.

#### `tree-sitter` job (new)

- **Environment:** Node.js 22 (via `actions/setup-node@v4`), pnpm (via `pnpm/action-setup`)
- **Steps:** `cd tree-sitter-opal && pnpm install && pnpm run test`
- **Isolation:** Runs independently, no Rust toolchain needed
- **Note:** `tree-sitter-cli` compiles a native binary during `pnpm install`; `ubuntu-latest` includes `gcc` so no additional system deps are needed

### What Changes

| File | Change |
|---|---|
| `.github/workflows/ci.yml` | Restructure into 3 parallel jobs; add spec test, coverage, tree-sitter steps |
| `docs/future-improvements.md` | Add note about porting spec tests to Rust integration test harness |

### What Doesn't Change

- No code changes to any crate
- No new dependencies in `Cargo.toml`
- No coverage thresholds or gates
- Spec test format (`# expect:` headers) stays the same
- Tree-sitter corpus test format stays the same

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Coverage tool | `cargo-llvm-cov` | LLVM-native instrumentation, accurate, runs on macOS locally |
| Coverage reporting | HTML artifact only | Solo developer, no need for Codecov/Coveralls integration |
| Coverage gate | None (report only) | Active language design phase; gates would add friction |
| Spec test integration | Run `run_spec.sh` as-is | Zero-risk, uses existing infrastructure |
| Tree-sitter CI | Separate parallel job | Independent of Rust build, no added wall-clock time |

## Deferred

These are captured in `docs/future-improvements.md`:

- Coverage ratchet gate (fail on decrease)
- Hard coverage threshold
- Codecov.io integration (when collaborators join)
- Port spec tests to Rust integration test harness (enables coverage instrumentation)
