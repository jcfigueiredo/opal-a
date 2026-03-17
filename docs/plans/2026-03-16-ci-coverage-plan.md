# CI Hardening + Code Coverage Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the CI pipeline by integrating all test suites and enabling code coverage reporting.

**Architecture:** Restructure the single CI job into three parallel jobs — `rust` (enhanced with spec tests), `coverage` (cargo-llvm-cov with HTML artifact), and `tree-sitter` (Node.js-based corpus tests). No code changes to any crate.

**Tech Stack:** GitHub Actions, cargo-llvm-cov, Node.js 22, pnpm, tree-sitter-cli

**Spec:** `docs/plans/2026-03-16-ci-coverage-design.md`

---

## Chunk 1: Enhance the Rust CI Job with Spec Tests

### Task 1: Add spec tests to the `rust` CI job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Rename the job and add spec test steps**

Replace the entire contents of `.github/workflows/ci.yml` with this — it keeps the existing `check` job intact and adds spec test steps at the end:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  rust:
    name: Rust Check & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features

      - name: Build
        run: cargo build --all-targets

      - name: Unit tests
        run: cargo test --all

      - name: Build release binary
        run: cargo build --release --bin opal

      - name: Spec tests
        run: OPAL_BIN="./target/release/opal" ./tests/run_spec.sh
```

- [ ] **Step 2: Verify YAML syntax is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No output (valid YAML)

If `pyyaml` is not available, alternatively: `cat .github/workflows/ci.yml | head -50` and visually confirm indentation.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add spec tests to rust CI job"
```

---

## Chunk 2: Add Coverage Job

### Task 2: Add the `coverage` job to CI

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Append the coverage job to ci.yml**

Add the following job after the `rust` job in `.github/workflows/ci.yml` (same indentation level as `rust:`):

```yaml
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Run tests with coverage instrumentation
        run: cargo llvm-cov --all-features --workspace --no-report

      - name: Generate HTML report
        run: cargo llvm-cov report --html

      - name: Generate lcov report
        run: cargo llvm-cov report --lcov --output-path lcov.info

      - name: Upload coverage HTML report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-html
          path: target/llvm-cov/html/
          retention-days: 30

      - name: Upload lcov.info
        uses: actions/upload-artifact@v4
        with:
          name: coverage-lcov
          path: lcov.info
          retention-days: 30
```

Note: `--html` and `--lcov` cannot be combined in a single `cargo llvm-cov` invocation. The `--no-report` flag runs the instrumented tests once, then `cargo llvm-cov report` generates each format from the cached data without re-running tests.

- [ ] **Step 2: Verify YAML syntax is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No output (valid YAML)

- [ ] **Step 3: Verify locally that cargo-llvm-cov works**

Run: `cargo install cargo-llvm-cov` (if not already installed), then: `rustup component add llvm-tools-preview`
Then:
```bash
cargo llvm-cov --all-features --workspace --no-report
cargo llvm-cov report --html
cargo llvm-cov report --lcov --output-path lcov.info
```
Expected: Test suite runs once, `target/llvm-cov/html/index.html` and `lcov.info` are generated.

Open `target/llvm-cov/html/index.html` in a browser to confirm the report renders correctly. This establishes the coverage baseline.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add code coverage job with cargo-llvm-cov"
```

---

## Chunk 3: Add Tree-Sitter Job

### Task 3: Add the `tree-sitter` job to CI

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Append the tree-sitter job to ci.yml**

Add the following job after the `coverage` job in `.github/workflows/ci.yml` (same indentation level):

```yaml
  tree-sitter:
    name: Tree-sitter Grammar Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '22'

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Install dependencies
        run: cd tree-sitter-opal && pnpm install

      - name: Run tree-sitter tests
        run: cd tree-sitter-opal && pnpm run test
```

- [ ] **Step 2: Verify YAML syntax is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No output (valid YAML)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add tree-sitter grammar tests job"
```

---

## Chunk 4: Update Future Improvements and Final Verification

### Task 4: Add deferred items to future-improvements.md

**Files:**
- Modify: `docs/future-improvements.md`

- [ ] **Step 1: Add spec test harness porting note**

Under the `## Test Infrastructure` section in `docs/future-improvements.md`, add:

```markdown
- **Port spec tests to Rust integration test harness**: Write a `tests/spec_runner.rs` that reads `# expect:` headers and runs via `cargo test`. Gets parallel execution, better failure diagnostics, and automatic inclusion in coverage instrumentation. Currently spec tests run via `run_spec.sh` and are not counted in coverage.
```

- [ ] **Step 2: Commit**

```bash
git add docs/future-improvements.md
git commit -m "docs: add spec test harness porting to future improvements"
```

### Task 5: Final verification — push and validate CI

- [ ] **Step 1: Review the final ci.yml**

Run: `cat .github/workflows/ci.yml`

Verify the file contains exactly three jobs: `rust`, `coverage`, `tree-sitter`. No `needs:` dependencies between them. All three run on `ubuntu-latest`.

- [ ] **Step 2: Push and monitor CI**

```bash
git push
```

Then check the GitHub Actions tab. All three jobs should appear and run in parallel:
- `Rust Check & Test` — should pass (fmt, clippy, build, unit tests, spec tests)
- `Code Coverage` — should pass and produce two artifacts (coverage-html, coverage-lcov)
- `Tree-sitter Grammar Tests` — should pass (34 tests across 12 corpus files)

- [ ] **Step 3: Download and inspect the coverage artifact**

From the GitHub Actions run, download the `coverage-html` artifact. Open `index.html` and review:
- Overall line coverage percentage (this is the baseline)
- Per-crate breakdown
- Identify any crates with surprisingly low coverage

This is informational — no action needed, but the numbers inform what to work on next.
