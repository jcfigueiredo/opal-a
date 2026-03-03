# Quote Syntax Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the `quote` keyword with `ast` across the entire Opal spec, adding an inline `ast(expr)` form for single-expression quoting.

**Architecture:** Two tasks: (1) update the BNF and hub in Opal.md, (2) update the two metaprogramming docs. The change is mechanical — rename `quote` to `ast` in code blocks and add the inline parenthesized form — but requires care to not touch string-related "quote" references in prose.

**Tech Stack:** Markdown files only. No code, no tests.

---

## Context

**Design doc:** `docs/plans/2026-03-03-quote-syntax-design.md`

**The change:** `quote expr end` → `ast(expr)` for inline, `quote...end` → `ast...end` for multi-line blocks. Everything else (interpolation `$`, splats `$...`, `macro`, `esc`, `macroexpand`, `Expr` type) stays the same.

**Files to modify:** 3 total — `Opal.md`, `docs/07-metaprogramming/metaprogramming.md`, `docs/appendix/self-hosting.md`

**Files NOT to modify:** `CLAUDE.md` — its "quote" references are about string quotes, not the metaprogramming keyword.

**Critical caution:** The word "quote" appears in prose about string quoting (e.g., "single/double quotes"). Only replace `quote` when it refers to the **metaprogramming keyword** — in code blocks, BNF rules, and prose describing AST quoting. Do NOT change:
- "Both quote styles for strings" (CLAUDE.md)
- "triple-quoted" (Opal.md line 318)
- "single/double quotes" in any prose

---

### Task 1: Update BNF and hub in Opal.md

**Files:**
- Modify: `Opal.md`

**What to change:**

**Change 1 — BNF rule (lines 59, 223-224):**

Line 59 in `<expression>` alternatives: change `<quote_expr>` to `<ast_expr>`

Lines 223-224: replace the rule definition:

Before:
```bnf
<quote_expr>    ::= "quote" <expression> "end"
                   | "quote" NEWLINE <block> "end"
```

After:
```bnf
<ast_expr>      ::= "ast" "(" <expression> ")"
                   | "ast" NEWLINE <block> "end"
```

Note the inline form uses parentheses, not `"ast" <expression> "end"`.

**Change 2 — Hub metaprogramming section (lines 647-661):**

Line 647 prose: change `quote...end captures code as Expr AST nodes` to `ast(...) captures code as Expr AST nodes`

Lines 651-661: update the `@memoize` example. Replace `quote` with `ast` — and convert inline uses to the parenthesized form where appropriate. Read the current example first to determine which `quote` usages are inline vs multi-line.

**Change 3 — Parser core list:**

If the metaprogramming hub section mentions `quote` in a keyword list (e.g., "quote, macro, $"), change to `ast, macro, $`.

**How to verify:** Search Opal.md for any remaining `quote` that refers to the metaprogramming keyword (ignore string-related "quote" in prose). Should find zero.

**Commit:** `"Rename quote to ast in BNF and Opal.md hub"`

---

### Task 2: Update metaprogramming.md and self-hosting.md

**Files:**
- Modify: `docs/07-metaprogramming/metaprogramming.md` (~30 occurrences)
- Modify: `docs/appendix/self-hosting.md` (~15 occurrences)

**What to change in `metaprogramming.md`:**

This file has `quote` in three contexts:
1. **Code blocks** — replace `quote ... end` with `ast(...)` for inline, `ast...end` for multi-line
2. **Prose** — replace "quoting" / "quote" when referring to the keyword (e.g., "Code is captured using `quote ... end`" → "Code is captured using `ast(...)` for inline or `ast...end` for blocks")
3. **Comparison table** — update the Julia comparison table row that mentions `quote`

**Specific sections to update:**

- **Quoting section title** (line ~14): Consider renaming from "Quoting – Code as Data" to "AST Literals – Code as Data"
- **Quoting intro** (line ~16): Update prose about `quote ... end`
- **All code examples** in the quoting section (lines ~22-70): Replace `quote` with `ast`, converting inline cases to `ast(expr)` form
- **Bullet points** (line ~70-71): Update `quote ... end` references
- **Macro examples** (lines ~86-162): Replace `quote` in macro bodies
- **Hygiene section** (lines ~108-135): Replace `quote` in examples
- **Practical examples** (lines ~332-443): Replace `quote` in `@json_serializable`, `@test`, `@debug`, `@memoize` examples
- **Self-Hosting section** (lines ~447-471): Update keyword lists — change "quote, macro, $" to "ast, macro, $"
- **Subdomain examples** (lines ~474-627): Replace `quote` in OpalWeb macro examples
- **Julia comparison table** (line ~648): Update `quote ... end` row
- **New Keywords table** (line ~668-669): Change `quote ... end` to `ast(expr)` / `ast...end`

**What to change in `self-hosting.md`:**

- **Parser core diagram** (~line 15-18): Change `quote, macro, $, @, @[...]` to `ast, macro, $, @, @[...]`
- **All macro code blocks**: Replace `quote` with `ast`, converting inline `quote expr end` to `ast(expr)` and multi-line `quote...end` to `ast...end`
- **Closing prose** (~line 492): If it mentions `quote`/`$`/`macro` primitives, update to `ast`/`$`/`macro`
- **macroexpand section** in the `needs` feature: ensure the example still works (just rename `quote` to `ast` in the context around it)

**How to distinguish inline vs multi-line:**
- If the current code has `quote <single-expression> end` on one line → replace with `ast(<single-expression>)`
- If the current code has `quote` on its own line followed by a block → replace with `ast` (same line, no parens, block follows)

**How to verify:**
```bash
grep -n 'quote' docs/07-metaprogramming/metaprogramming.md | grep -v 'docs/plans/'
grep -n 'quote' docs/appendix/self-hosting.md
```
Both should return zero matches (or only matches in string-quoting prose if any exist).

**Commit:** `"Rename quote to ast in metaprogramming docs"`

---

### Task 3: Final audit

**Files:** All 3 modified files + CLAUDE.md (verify no accidental changes)

**Step 1:** Search for any remaining `quote` that refers to the metaprogramming keyword:
```bash
grep -n '\bquote\b' Opal.md docs/07-metaprogramming/metaprogramming.md docs/appendix/self-hosting.md | grep -v 'docs/plans/' | grep -v 'triple-quoted' | grep -v 'quote styles' | grep -v 'single.*quote' | grep -v 'double.*quote'
```
Expected: zero matches for the metaprogramming keyword.

**Step 2:** Verify all `ast(` inline forms have matching closing `)`:
```bash
grep -n 'ast(' docs/07-metaprogramming/metaprogramming.md docs/appendix/self-hosting.md | head -20
```
Expected: every `ast(` has a matching `)` on the same line.

**Step 3:** Verify BNF consistency — the rule name should be `<ast_expr>` and referenced in the `<expression>` alternatives:
```bash
grep -n 'ast_expr\|quote_expr' Opal.md
```
Expected: only `ast_expr`, no `quote_expr`.

**Step 4:** Verify CLAUDE.md was NOT accidentally modified:
```bash
git diff CLAUDE.md
```
Expected: no changes.

Fix any issues found.

**Commit:** `"Audit and fix remaining quote-to-ast instances"` (only if fixes needed)

---

## Dependency Graph

```
Task 1 (BNF + Opal.md hub)
  └─> Task 2 (metaprogramming.md + self-hosting.md)
        └─> Task 3 (audit)
```

Tasks are sequential — the BNF change goes first as the canonical source, then the docs update to match.
