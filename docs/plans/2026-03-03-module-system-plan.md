# Module System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `from X import Y` with Gleam-style `import X.{Y}`, add file-based module loading, `pub` keyword for export control, and `export {}` blocks.

**Architecture:** New `StmtKind::Import` with rich import spec (whole module, selective with braces, aliased). File resolver finds `.opl` files relative to the importing file, then project root. `pub` keyword is parsed as a flag on `def`/`class` statements (ignored in Phase 1 scripts — public by default). `export {}` block parsed as a top-level statement. Old `from X import Y` syntax kept as deprecated (parser rewrites to new form internally).

**Tech Stack:** Same as existing — no new deps.

**Design doc:** `docs/plans/2026-03-03-module-system-design.md`

---

### Task 1: Add new import AST nodes

**Files:**
- Modify: `crates/opal-parser/src/ast.rs`

Replace `StmtKind::FromImport` with a richer `StmtKind::Import`:

```rust
/// Import statement (Gleam-style)
Import(ImportStmt),

/// Export block: `export {name1, name2}`
ExportBlock(Vec<String>),
```

Add import types:
```rust
#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub path: Vec<String>,        // ["Math", "Vector"]
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    /// import Math — whole module
    Module,
    /// import Math as M — aliased module
    ModuleAlias(String),
    /// import Math.{abs, max} — selective
    Selective(Vec<ImportItem>),
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}
```

Keep `FromImport` temporarily for backwards compat — mark it deprecated with a comment.

**Step 1:** Add all types. `cargo build --package opal-parser`.
**Step 2:** Commit.

---

### Task 2: Parse new import syntax

**Files:**
- Modify: `crates/opal-parser/src/parser.rs`

Add `Token::Import` check in `parse_statement`. Currently `import` is a keyword token used inside `from X import Y`. Now it becomes a standalone statement starter.

**`parse_import`:**
```
import IDENT                        → Module import
import IDENT as IDENT               → Aliased module
import IDENT.IDENT                  → Nested module
import IDENT.{IDENT, IDENT}         → Selective
import IDENT.{IDENT as IDENT, ...}  → Selective with aliases
```

The parser:
1. Consume `import`
2. Parse module path: `IDENT (.IDENT)*`
3. Check what follows:
   - `.{` → selective import (parse items until `}`)
   - `as IDENT` → aliased module
   - newline/EOF → whole module

Keep the `from X import Y` parser but have it produce `StmtKind::Import` internally (backwards compat).

**Parse `export {names}`:**
When `Token::Export` is encountered in `parse_statement`:
1. Consume `export`
2. Expect `{`
3. Parse comma-separated identifiers
4. Expect `}`

**Tests:**
```rust
#[test]
fn parse_import_module() {
    let prog = parse("import Math");
    // verify StmtKind::Import with path=["Math"], kind=Module
}

#[test]
fn parse_import_selective() {
    let prog = parse("import Math.{abs, max}");
    // verify Selective with 2 items
}

#[test]
fn parse_import_alias() {
    let prog = parse("import Math.Vector as Vec");
    // verify ModuleAlias("Vec")
}

#[test]
fn parse_import_selective_alias() {
    let prog = parse("import Math.{abs, max as maximum}");
    // verify item alias
}

#[test]
fn parse_export_block() {
    let prog = parse("export {abs, Vector}");
    // verify ExportBlock
}

#[test]
fn parse_from_import_compat() {
    let prog = parse("from Shapes import Circle, Rectangle");
    // verify still works, produces Import
}
```

**Step 1:** Implement parser + tests. `cargo test --package opal-parser`.
**Step 2:** Commit.

---

### Task 3: Implement file-based module loading

**Files:**
- Create: `crates/opal-interp/src/loader.rs`
- Modify: `crates/opal-interp/src/lib.rs`

**`loader.rs`** — resolves module paths to files and loads them:

```rust
use std::path::{Path, PathBuf};
use std::collections::HashSet;

pub struct ModuleLoader {
    /// Directories to search for modules
    search_paths: Vec<PathBuf>,
    /// Modules currently being loaded (circular dependency detection)
    loading: HashSet<String>,
    /// Already loaded modules
    loaded: HashSet<String>,
}

impl ModuleLoader {
    pub fn new(base_dir: &Path) -> Self {
        let mut search_paths = vec![base_dir.to_path_buf()];
        // Add OPAL_PATH entries
        if let Ok(opal_path) = std::env::var("OPAL_PATH") {
            for p in opal_path.split(':') {
                search_paths.push(PathBuf::from(p));
            }
        }
        Self { search_paths, loading: HashSet::new(), loaded: HashSet::new() }
    }

    pub fn resolve(&self, module_path: &[String]) -> Option<PathBuf> {
        // Convert ["Math", "Vector"] to "math/vector.opl" or "math_vector.opl"
        let filename = module_path.last()?.to_lowercase();
        let dir_parts: Vec<String> = module_path[..module_path.len()-1].iter()
            .map(|s| s.to_lowercase()).collect();

        for search_dir in &self.search_paths {
            // Try: search_dir/math/vector.opl
            let mut path = search_dir.clone();
            for part in &dir_parts {
                path.push(part);
            }
            path.push(format!("{}.opl", filename));
            if path.exists() {
                return Some(path);
            }

            // Try: search_dir/math.opl (single file module)
            if module_path.len() == 1 {
                let mut path = search_dir.clone();
                path.push(format!("{}.opl", filename));
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }

    pub fn mark_loading(&mut self, key: &str) -> bool {
        // Returns false if circular
        self.loading.insert(key.to_string())
    }

    pub fn mark_loaded(&mut self, key: &str) {
        self.loading.remove(key);
        self.loaded.insert(key.to_string());
    }

    pub fn is_loaded(&self, key: &str) -> bool {
        self.loaded.contains(key)
    }
}
```

**Step 1:** Create loader.rs. `cargo build --package opal-interp`.
**Step 2:** Commit.

---

### Task 4: Handle new Import in interpreter + file loading

**Files:**
- Modify: `crates/opal-interp/src/eval.rs`

**Add `module_loader: Option<ModuleLoader>` to Interpreter.**

Update `Interpreter::new()` — doesn't set loader (no file context).
Add `Interpreter::with_base_dir(dir)` or set loader when CLI provides a file path.

**Handle `StmtKind::Import`:**

```rust
StmtKind::Import(import_stmt) => {
    let module_key = import_stmt.path.join(".");

    // 1. Check if module is already in scope (in-memory module)
    if let Some(val) = self.env.get(&module_key).cloned() {
        self.apply_import(&import_stmt.kind, &module_key, &val)?;
        return Ok(());
    }

    // 2. Try file-based loading
    if let Some(ref mut loader) = self.module_loader {
        if !loader.is_loaded(&module_key) {
            if let Some(file_path) = loader.resolve(&import_stmt.path) {
                if !loader.mark_loading(&module_key) {
                    return Err(EvalError::RuntimeError(
                        format!("circular dependency: {}", module_key)
                    ));
                }
                let source = std::fs::read_to_string(&file_path)
                    .map_err(|e| EvalError::RuntimeError(e.to_string()))?;
                let program = opal_parser::parse(&source)
                    .map_err(|e| EvalError::RuntimeError(e.to_string()))?;

                // Eval in a new scope, capture as module
                self.env.push_scope();
                self.run_inner(&program)?;
                let bindings = self.env.current_scope_bindings();
                self.env.pop_scope();

                let module_id = ModuleId(self.modules.len());
                self.modules.push(StoredModule {
                    name: module_key.clone(),
                    bindings,
                });
                self.env.set(module_key.clone(), Value::Module(module_id));
                loader.mark_loaded(&module_key);
            }
        }

        // Now apply the import
        if let Some(val) = self.env.get(&module_key).cloned() {
            self.apply_import(&import_stmt.kind, &module_key, &val)?;
            return Ok(());
        }
    }

    // 3. Module not found
    return Err(EvalError::UndefinedVariable(module_key));
}
```

`apply_import` handles the different import kinds:
- `Module` → bind the module value to the last path component
- `ModuleAlias(alias)` → bind to alias
- `Selective(items)` → copy each named binding from the module into current scope

**Handle `StmtKind::ExportBlock`:** Store the export list on the interpreter. When other modules import from this one, only export-listed names are visible. (Phase 1 simplification: parse and store, but don't enforce — everything still public.)

**Update CLI** to pass the file's directory to the interpreter so the loader knows the base path.

**Tests:**
```rust
#[test]
fn import_in_memory_module() {
    let output = run(r#"
module Utils
  def double(x)
    x * 2
  end
end
import Utils.{double}
print(double(5))
"#).unwrap();
    assert_eq!(output, "10");
}

#[test]
fn import_whole_module() {
    let output = run(r#"
module Math
  def abs(x)
    if x < 0 then -x else x end
  end
end
import Math
print(Math.abs(-5))
"#).unwrap();
    assert_eq!(output, "5");
}
```

**Step 1:** Implement Import handling + loader integration + tests.
**Step 2:** Commit.

---

### Task 5: Update existing tests and spec files

**Files:**
- Modify: all spec tests using `from X import Y` → `import X.{Y}`
- Modify: interpreter tests using `from X import Y`

Key files to update:
- `tests/spec/04-classes/shapes.opl` — `from Shapes import Rectangle` → `import Shapes.{Rectangle}`
- `crates/opal-interp/src/eval.rs` tests — `module_and_import` test

Keep the old `from X import Y` parser working (it internally produces `StmtKind::Import` now), so nothing breaks even if we don't update all tests immediately.

**Step 1:** Update spec tests to new syntax.
**Step 2:** Run `bash tests/run_spec.sh` — all pass.
**Step 3:** Commit.

---

### Task 6: Add file-based module spec test

**Files:**
- Create: `tests/spec/09-modules/main.opl`
- Create: `tests/spec/09-modules/utils.opl`

**`tests/spec/09-modules/utils.opl`:**
```opal
pub def double(x)
  x * 2
end

def helper()
  "private"
end
```

**`tests/spec/09-modules/main.opl`:**
```opal
# expect: 10
import Utils.{double}
print(double(5))
```

This test verifies file-based module loading end-to-end.

**Step 1:** Create files.
**Step 2:** Run `bash tests/run_spec.sh` — verify it passes.
**Step 3:** Run full `cargo test --all`, `cargo clippy`, `cargo fmt`.
**Step 4:** Commit.

---

## Dependency Graph

```
Task 1 (AST) → Task 2 (parser) → Task 3 (loader) → Task 4 (interpreter) → Task 5 (update tests) → Task 6 (file module test)
```
