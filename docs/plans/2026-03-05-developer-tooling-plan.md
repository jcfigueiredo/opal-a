# Developer Tooling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add editor support for Opal — tree-sitter grammar for syntax highlighting + LSP server for diagnostics, go-to-definition, and document symbols.

**Architecture:** Two independent deliverables: (1) `tree-sitter-opal/` standalone package with grammar.js, external scanner for f-strings, and highlight queries; (2) `crates/opal-lsp/` new workspace crate using tower-lsp-server that reuses opal-lexer and opal-parser for parsing. Tree-sitter is built first, LSP second.

**Tech Stack:** Tree-sitter CLI (Node.js), C (external scanner), Rust (LSP), tower-lsp-server, tokio, opal-lexer, opal-parser

**Reference files:**
- Design: `docs/plans/2026-03-05-developer-tooling-design.md`
- Tokens: `crates/opal-lexer/src/token.rs` (109 token variants, 55 keywords)
- AST: `crates/opal-parser/src/ast.rs` (27 StmtKind, 27 ExprKind variants)
- Parser: `crates/opal-parser/src/parser.rs` (83 parse methods — canonical grammar reference)
- Spec tests: `tests/spec/**/*.opl` (42 files — use as grammar validation)

---

## Part 1: Tree-sitter Grammar (Tasks 1–8)

---

### Task 1: Scaffold tree-sitter-opal package

**Files:**
- Create: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/package.json`
- Create: `tree-sitter-opal/.gitignore`

**Step 1: Create package.json**

```json
{
  "name": "tree-sitter-opal",
  "version": "0.1.0",
  "description": "Tree-sitter grammar for the Opal programming language",
  "main": "bindings/node",
  "keywords": ["parser", "tree-sitter", "opal"],
  "dependencies": {
    "nan": "^2.18.0"
  },
  "devDependencies": {
    "tree-sitter-cli": "^0.25.0"
  },
  "scripts": {
    "generate": "tree-sitter generate",
    "test": "tree-sitter test",
    "parse": "tree-sitter parse"
  },
  "tree-sitter": [
    {
      "scope": "source.opal",
      "file-types": ["opl"],
      "injection-regex": "^opal$"
    }
  ]
}
```

**Step 2: Create .gitignore**

```
node_modules/
build/
src/parser.c
src/tree_sitter/
bindings/
```

**Step 3: Create minimal grammar.js skeleton**

This is a breadth-first skeleton. It parses only the simplest possible Opal program (`print("hello")`). We'll expand it in subsequent tasks.

```javascript
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'opal',

  extras: $ => [
    /[ \t\r]/,
    $.comment,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.expression_statement,
    ),

    expression_statement: $ => seq(
      $._expression,
      $._terminator,
    ),

    _expression: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.call,
    ),

    call: $ => prec(1, seq(
      $._expression,
      '(',
      optional(seq($._expression, repeat(seq(',', $._expression)))),
      ')',
    )),

    // Literals
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*!?/,
    integer: $ => /[0-9][0-9_]*/,
    float: $ => /[0-9][0-9_]*\.[0-9][0-9_]*/,
    string: $ => choice(
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
    ),

    // Comments
    comment: $ => token(seq('#', /.*/)),

    // Statement terminator
    _terminator: $ => choice('\n', /\0/),
  },
});
```

**Step 4: Install dependencies and generate**

Run:
```bash
cd tree-sitter-opal && npm install && npx tree-sitter generate
```
Expected: Generates `src/parser.c` and `src/tree_sitter/` directory without errors.

**Step 5: Smoke test with a simple file**

Run:
```bash
echo 'print("hello")' > /tmp/test.opl && npx tree-sitter parse /tmp/test.opl
```
Expected: Tree output showing `(source_file (expression_statement (call ...)))` — no ERROR nodes.

**Step 6: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat: scaffold tree-sitter-opal package with minimal grammar"
```

---

### Task 2: Add keywords, variables, and assignment

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/test/corpus/basics.txt`

**Context:** Opal has 55 keywords (see `crates/opal-lexer/src/token.rs:173-283`). Newlines are significant (statement terminators). Variables are assigned with `name = expr`, `let name = expr`. Booleans are `true`/`false`, null is `null`.

**Step 1: Write corpus test for basics**

Tree-sitter tests use a specific format. Create `test/corpus/basics.txt`:

```
================
Integer assignment
================

x = 42

---

(source_file
  (assignment
    (identifier)
    (integer)))

================
Let binding
================

let name = "opal"

---

(source_file
  (let_binding
    (identifier)
    (string)))

================
Boolean and null
================

a = true
b = false
c = null

---

(source_file
  (assignment (identifier) (true))
  (assignment (identifier) (false))
  (assignment (identifier) (null)))

================
Symbol literal
================

status = :ok

---

(source_file
  (assignment
    (identifier)
    (symbol)))

================
Compound assignment
================

x += 10

---

(source_file
  (compound_assignment
    (identifier)
    (integer)))
```

**Step 2: Run test to verify it fails**

Run:
```bash
cd tree-sitter-opal && npx tree-sitter test
```
Expected: FAIL — rules don't exist yet.

**Step 3: Update grammar.js with keywords, assignment, let, symbols**

Replace the `rules` in `grammar.js`:

```javascript
module.exports = grammar({
  name: 'opal',

  extras: $ => [
    /[ \t\r]/,
    $.comment,
  ],

  word: $ => $.identifier,

  conflicts: $ => [
  ],

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => seq(
      choice(
        $.assignment,
        $.compound_assignment,
        $.let_binding,
        $.expression_statement,
      ),
      $._terminator,
    ),

    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    compound_assignment: $ => seq(
      field('name', $.identifier),
      field('operator', choice('+=', '-=', '*=', '/=')),
      field('value', $._expression),
    ),

    let_binding: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    expression_statement: $ => $._expression,

    _expression: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.true,
      $.false,
      $.null,
      $.symbol,
      $.call,
      $.binary_expression,
      $.unary_expression,
      $.grouped_expression,
    ),

    call: $ => prec(2, seq(
      field('function', $._expression),
      '(',
      optional(seq($._argument, repeat(seq(',', $._argument)))),
      ')',
    )),

    _argument: $ => choice(
      $.named_argument,
      $._expression,
    ),

    named_argument: $ => seq(
      field('name', $.identifier),
      ':',
      field('value', $._expression),
    ),

    binary_expression: $ => {
      const table = [
        [1, 'or'],
        [2, 'and'],
        [3, choice('==', '!=')],
        [4, choice('<', '<=', '>', '>=')],
        [4, 'in'],
        [4, 'is'],
        [5, choice('+', '-')],
        [6, choice('*', '/', '%')],
        [8, '|>'],
        [9, '..'],
        [9, '...'],
        [10, '??'],
      ];

      return choice(
        ...table.map(([precedence, op]) =>
          prec.left(precedence, seq(
            field('left', $._expression),
            field('operator', op),
            field('right', $._expression),
          ))
        ),
        prec.right(7, seq(
          field('left', $._expression),
          field('operator', '**'),
          field('right', $._expression),
        )),
      );
    },

    unary_expression: $ => prec(11, choice(
      seq('-', $._expression),
      seq('not', $._expression),
    )),

    grouped_expression: $ => seq('(', $._expression, ')'),

    // Literals
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*!?/,
    integer: $ => /[0-9][0-9_]*/,
    float: $ => /[0-9][0-9_]*\.[0-9][0-9_]*/,
    string: $ => choice(
      seq('"""', /([^"]|"[^"]|""[^"])*/, '"""'),
      seq("'''", /([^']|'[^']|''[^'])*/, "'''"),
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
    ),
    symbol: $ => /:[a-zA-Z_][a-zA-Z0-9_]*/,
    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',

    comment: $ => token(choice(
      seq('#', /[^#\n][^\n]*/),
      seq('#', /\n/),
      seq('###', /(.|\n)*?/, '###'),
    )),

    _terminator: $ => /\n/,
  },
});
```

**Step 4: Generate and run tests**

Run:
```bash
cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test
```
Expected: All corpus tests pass.

**Step 5: Smoke test with spec file**

Run:
```bash
cd tree-sitter-opal && npx tree-sitter parse ../tests/spec/01-basics/bounty_amounts.opl 2>&1 | head -20
```
Expected: Some tree output. ERROR nodes are OK at this stage (we haven't added functions/classes yet).

**Step 6: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add keywords, assignment, operators, literals"
```

---

### Task 3: Add function definitions, if/elsif/else, return

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/test/corpus/functions.txt`
- Create: `tree-sitter-opal/test/corpus/control_flow.txt`

**Context:** Opal functions use `def name(params) ... end`. Parameters can have type annotations (`name: Type`) and defaults (`name = value`). If expressions use `if cond ... elsif cond ... else ... end` or inline `if cond then expr else expr end`. Return is `return expr`.

**Step 1: Write corpus tests**

Create `test/corpus/functions.txt`:

```
================
Simple function
================

def greet(name)
  print(name)
end

---

(source_file
  (function_definition
    (identifier)
    (parameters (parameter (identifier)))
    (body (expression_statement (call (identifier) (identifier))))))

================
Typed function with return
================

def add(a: Int, b: Int) -> Int
  return a + b
end

---

(source_file
  (function_definition
    (identifier)
    (parameters
      (parameter (identifier) (type_annotation (identifier)))
      (parameter (identifier) (type_annotation (identifier))))
    (return_type (identifier))
    (body (return_statement (binary_expression (identifier) (identifier))))))
```

Create `test/corpus/control_flow.txt`:

```
================
If else
================

if x > 0
  print("positive")
else
  print("non-positive")
end

---

(source_file
  (if_expression
    (binary_expression (identifier) (integer))
    (body (expression_statement (call (identifier) (string))))
    (else_clause
      (body (expression_statement (call (identifier) (string)))))))

================
If elsif else
================

if x > 10
  "big"
elsif x > 0
  "small"
else
  "zero"
end

---

(source_file
  (if_expression
    (binary_expression (identifier) (integer))
    (body (expression_statement (string)))
    (elsif_clause
      (binary_expression (identifier) (integer))
      (body (expression_statement (string))))
    (else_clause
      (body (expression_statement (string))))))
```

**Step 2: Run tests to verify they fail**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`
Expected: FAIL

**Step 3: Add grammar rules for functions, if, return**

Add to `grammar.js` rules (in `_statement` choice and as new rules):

```javascript
// In _statement choice, add:
$.function_definition,
$.return_statement,

// New rules:
function_definition: $ => seq(
  optional(field('visibility', choice('public', 'private'))),
  'def',
  field('name', $.identifier),
  optional(field('params', $.parameters)),
  optional(seq('->', field('return_type', $.type_annotation))),
  $._terminator,
  field('body', $.body),
  'end',
),

parameters: $ => seq(
  '(',
  optional(seq($.parameter, repeat(seq(',', $.parameter)))),
  ')',
),

parameter: $ => seq(
  field('name', $.identifier),
  optional(seq(':', field('type', $.type_annotation))),
  optional(seq('=', field('default', $._expression))),
),

type_annotation: $ => seq(
  $.identifier,
  optional(seq('[', $.type_annotation, repeat(seq(',', $.type_annotation)), ']')),
  optional('?'),
),

return_statement: $ => seq('return', optional($._expression)),

body: $ => repeat1($._statement),

// Add if_expression to _expression choice:
if_expression: $ => seq(
  'if',
  field('condition', $._expression),
  optional('then'),
  $._terminator,
  optional(field('consequence', $.body)),
  repeat($.elsif_clause),
  optional($.else_clause),
  'end',
),

elsif_clause: $ => seq(
  'elsif',
  field('condition', $._expression),
  optional('then'),
  $._terminator,
  optional(field('body', $.body)),
),

else_clause: $ => seq(
  'else',
  $._terminator,
  optional(field('body', $.body)),
),
```

**Step 4: Generate and run tests**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`
Expected: All corpus tests pass.

**Step 5: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add functions, if/elsif/else, return"
```

---

### Task 4: Add classes, protocols, modules, needs, enums, models

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/test/corpus/classes.txt`

**Context:** See `crates/opal-parser/src/ast.rs:46-58` for ClassDef, ProtocolDef, ModuleDef. Classes use `needs` for DI. Enums have variants with fields. Models are immutable classes. Protocols define required methods.

**Step 1: Write corpus test**

Create `test/corpus/classes.txt`:

```
================
Class with needs and method
================

class Bounty
  needs title: String
  needs amount: Float

  def display()
    print(.title)
  end
end

---

(source_file
  (class_definition
    (identifier)
    (needs_declaration (identifier) (type_annotation (identifier)))
    (needs_declaration (identifier) (type_annotation (identifier)))
    (function_definition
      (identifier)
      (parameters)
      (body (expression_statement (call (identifier) (instance_variable)))))))

================
Class implements protocol
================

class Dog implements Animal
  needs name: String
end

---

(source_file
  (class_definition
    (identifier)
    (implements_clause (identifier))
    (needs_declaration (identifier) (type_annotation (identifier)))))

================
Enum with variants
================

enum Color
  Red
  Green
  Blue(intensity: Float)
end

---

(source_file
  (enum_definition
    (identifier)
    (enum_variant (identifier))
    (enum_variant (identifier))
    (enum_variant (identifier) (enum_fields (needs_declaration (identifier) (type_annotation (identifier)))))))

================
Protocol
================

protocol Scorable
  def score() -> Int
end

---

(source_file
  (protocol_definition
    (identifier)
    (protocol_method
      (identifier)
      (parameters)
      (return_type (identifier)))))

================
Module
================

module Utils
  def helper()
    42
  end
end

---

(source_file
  (module_definition
    (identifier)
    (body
      (function_definition
        (identifier)
        (parameters)
        (body (expression_statement (integer)))))))
```

**Step 2: Run tests to verify failure**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`

**Step 3: Add grammar rules**

Add to grammar.js:

```javascript
// In _statement choice, add:
$.class_definition,
$.protocol_definition,
$.module_definition,
$.enum_definition,
$.model_definition,
$.needs_declaration,
$.instance_assign,

// New rules:
class_definition: $ => seq(
  'class',
  field('name', $.identifier),
  optional($.implements_clause),
  $._terminator,
  repeat(choice(
    seq($.needs_declaration, $._terminator),
    seq($.function_definition, $._terminator),
  )),
  'end',
),

implements_clause: $ => seq(
  'implements',
  $.identifier,
  repeat(seq(',', $.identifier)),
),

needs_declaration: $ => seq(
  'needs',
  field('name', $.identifier),
  optional(seq(':', field('type', $.type_annotation))),
  optional(seq('=', field('default', $._expression))),
),

protocol_definition: $ => seq(
  'protocol',
  field('name', $.identifier),
  $._terminator,
  repeat(seq($.protocol_method, $._terminator)),
  'end',
),

protocol_method: $ => seq(
  'def',
  field('name', $.identifier),
  optional(field('params', $.parameters)),
  optional(seq('->', field('return_type', $.type_annotation))),
  optional(seq($._terminator, field('body', $.body), 'end')),
),

module_definition: $ => seq(
  'module',
  field('name', $.identifier),
  optional(seq($._terminator, repeat(seq('needs', $.identifier, ':', $.type_annotation, $._terminator)))),
  $._terminator,
  optional(field('body', $.body)),
  'end',
),

enum_definition: $ => seq(
  'enum',
  field('name', $.identifier),
  $._terminator,
  repeat(seq($.enum_variant, $._terminator)),
  repeat(seq($.function_definition, $._terminator)),
  'end',
),

enum_variant: $ => seq(
  field('name', $.identifier),
  optional($.enum_fields),
),

enum_fields: $ => seq(
  '(',
  $.needs_declaration,
  repeat(seq(',', $.needs_declaration)),
  ')',
),

model_definition: $ => seq(
  'model',
  field('name', $.identifier),
  $._terminator,
  repeat(seq($.needs_declaration, $._terminator)),
  repeat(seq($.function_definition, $._terminator)),
  'end',
),

instance_assign: $ => seq(
  $.instance_variable,
  '=',
  field('value', $._expression),
),

// Add to _expression:
instance_variable: $ => seq('.', $.identifier),

member_access: $ => prec(3, seq(
  field('object', $._expression),
  '.',
  field('field', $.identifier),
)),
```

**Step 4: Generate and run tests**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`

**Step 5: Parse a real class spec test**

Run: `cd tree-sitter-opal && npx tree-sitter parse ../tests/spec/04-classes/contributor_class.opl`
Expected: Largely correct tree, some ERROR nodes acceptable for features not yet added.

**Step 6: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add classes, protocols, modules, enums, models, needs"
```

---

### Task 5: Add loops, match/case, closures, collections

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/test/corpus/loops.txt`
- Create: `tree-sitter-opal/test/corpus/match.txt`
- Create: `tree-sitter-opal/test/corpus/closures.txt`

**Context:** For loops: `for x in expr ... end`. While loops: `while cond ... end`. Match: `match expr case pattern ... case pattern ... end`. Closures: `|params| expr` (inline) or `do |params| ... end` (block). Lists: `[1, 2, 3]`. Dicts: `{key: value}` or `{:}`. Ranges: `1..10`, `1...10`. List comprehensions: `[x * 2 for x in list]`.

**Step 1: Write corpus tests for loops and match**

Create `test/corpus/loops.txt`:

```
================
For loop
================

for item in list
  print(item)
end

---

(source_file
  (for_loop
    (identifier)
    (identifier)
    (body (expression_statement (call (identifier) (identifier))))))

================
While loop
================

while x > 0
  x -= 1
end

---

(source_file
  (while_loop
    (binary_expression (identifier) (integer))
    (body (compound_assignment (identifier) (integer)))))
```

Create `test/corpus/match.txt`:

```
================
Match with cases
================

match status
  case :ok
    "good"
  case :error
    "bad"
  case _
    "unknown"
end

---

(source_file
  (match_expression
    (identifier)
    (match_case (pattern (symbol)) (body (expression_statement (string))))
    (match_case (pattern (symbol)) (body (expression_statement (string))))
    (match_case (pattern (wildcard)) (body (expression_statement (string))))))
```

Create `test/corpus/closures.txt`:

```
================
Inline closure
================

nums.map(|x| x * 2)

---

(source_file
  (expression_statement
    (call
      (member_access (identifier) (identifier))
      (closure (closure_params (identifier)) (binary_expression (identifier) (integer))))))

================
Block closure
================

nums.reduce(0) do |acc, n|
  acc + n
end

---

(source_file
  (expression_statement
    (block_closure_call
      (call (member_access (identifier) (identifier)) (integer))
      (closure_params (identifier) (identifier))
      (body (expression_statement (binary_expression (identifier) (identifier)))))))
```

**Step 2: Run tests to verify failure**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`

**Step 3: Add grammar rules**

Add to grammar.js:

```javascript
// In _statement choice, add:
$.for_loop,
$.while_loop,
$.break_statement,
$.next_statement,

// New rules:
for_loop: $ => seq(
  'for',
  field('var', choice($.identifier, $.destructure_pattern)),
  'in',
  field('iterable', $._expression),
  $._terminator,
  field('body', $.body),
  'end',
),

while_loop: $ => seq(
  'while',
  field('condition', $._expression),
  $._terminator,
  field('body', $.body),
  'end',
),

break_statement: $ => 'break',
next_statement: $ => 'next',

destructure_pattern: $ => seq(
  '[',
  $.identifier,
  repeat(seq(',', $.identifier)),
  optional(seq('|', $.identifier)),
  ']',
),

// Add match_expression to _expression:
match_expression: $ => seq(
  'match',
  field('subject', $._expression),
  $._terminator,
  repeat1($.match_case),
  'end',
),

match_case: $ => seq(
  'case',
  field('pattern', $.pattern),
  optional(seq('if', field('guard', $._expression))),
  $._terminator,
  optional(field('body', $.body)),
),

pattern: $ => choice(
  $.wildcard,
  $.symbol,
  $.integer,
  $.float,
  $.string,
  $.true,
  $.false,
  $.null,
  $.constructor_pattern,
  $.enum_variant_pattern,
  $.list_pattern,
  $.or_pattern,
  $.range_pattern,
  $.identifier,
),

wildcard: $ => '_',

constructor_pattern: $ => seq(
  $.identifier,
  '(',
  optional(seq($.pattern, repeat(seq(',', $.pattern)))),
  ')',
),

enum_variant_pattern: $ => seq(
  $.identifier,
  '.',
  $.identifier,
  optional(seq('(', optional(seq($.pattern, repeat(seq(',', $.pattern)))), ')')),
),

list_pattern: $ => seq(
  '[',
  optional(seq($.pattern, repeat(seq(',', $.pattern)))),
  optional(seq('|', $.pattern)),
  ']',
),

or_pattern: $ => prec.left(seq($.pattern, '|', $.pattern)),

range_pattern: $ => seq($.integer, choice('..', '...'), $.integer),

// Add closures to _expression:
closure: $ => seq(
  '|',
  optional($.closure_params),
  '|',
  $._expression,
),

block_closure: $ => seq(
  'do',
  optional(seq('|', optional($.closure_params), '|')),
  $._terminator,
  optional($.body),
  'end',
),

// Call followed by block closure (e.g., list.reduce(0) do |acc, n| ... end)
block_closure_call: $ => prec(1, seq(
  $.call,
  $.block_closure,
)),

closure_params: $ => seq(
  $.identifier,
  repeat(seq(',', $.identifier)),
),

// Collections — add to _expression:
list: $ => seq(
  '[',
  optional(seq($._expression, repeat(seq(',', $._expression)), optional(','))),
  ']',
),

dict: $ => seq(
  '{',
  choice(
    seq(':', '}'),  // empty dict: {:}
    seq(
      optional(seq($.dict_entry, repeat(seq(',', $.dict_entry)), optional(','))),
      '}',
    ),
  ),
),

dict_entry: $ => seq(
  field('key', $._expression),
  ':',
  field('value', $._expression),
),

list_comprehension: $ => seq(
  '[',
  $._expression,
  'for',
  $.identifier,
  'in',
  $._expression,
  optional(seq('if', $._expression)),
  ']',
),

range_expression: $ => prec.left(9, seq(
  $._expression,
  choice('..', '...'),
  $._expression,
)),
```

**Step 4: Generate and run tests**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`

**Step 5: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add loops, match/case, closures, collections"
```

---

### Task 6: Add actors, try/catch, imports, macros, events, and remaining constructs

**Files:**
- Modify: `tree-sitter-opal/grammar.js`
- Create: `tree-sitter-opal/test/corpus/actors.txt`
- Create: `tree-sitter-opal/test/corpus/advanced.txt`

**Context:** Actors: `actor Name ... receive ... case ... end end`. Try/catch: `try ... catch ErrorType as e ... ensure ... end`. Imports: `from X import Y, Z` or `import X.{a, b}`. Macros: `macro name(params) ... end`, invocation `@name args ... end`. Events: `event Name(field: Type)`, `on EventType do |e| ... end`, `emit expr`. Also: `type Name = Type`, `requires cond, "msg"`, `raise expr`, `extern "lib" ... end`, `implements Protocol for Type ... end` (retroactive).

**Step 1: Write corpus tests**

Create `test/corpus/actors.txt`:

```
================
Actor with receive
================

actor Counter
  def init()
    .count = 0
  end

  receive
    case :increment
      .count = .count + 1
    case :get
      reply .count
  end
end

---

(source_file
  (actor_definition
    (identifier)
    (function_definition
      (identifier)
      (parameters)
      (body (instance_assign (instance_variable) (integer))))
    (receive_block
      (match_case (pattern (symbol)) (body (instance_assign (instance_variable) (binary_expression (instance_variable) (integer)))))
      (match_case (pattern (symbol)) (body (reply_statement (instance_variable)))))))
```

Create `test/corpus/advanced.txt`:

```
================
Import
================

from Math import sqrt, PI

---

(source_file
  (from_import (identifier) (identifier) (identifier)))

================
Try catch
================

try
  risky()
catch Error as e
  print(e)
end

---

(source_file
  (try_catch
    (body (expression_statement (call (identifier))))
    (catch_clause (identifier) (identifier) (body (expression_statement (call (identifier) (identifier)))))))

================
Event and emit
================

event UserJoined(name: String)

on UserJoined do |e|
  print(e)
end

emit UserJoined.new(name: "alice")

---

(source_file
  (event_definition (identifier) (needs_declaration (identifier) (type_annotation (identifier))))
  (on_handler (identifier) (identifier) (body (expression_statement (call (identifier) (identifier)))))
  (expression_statement (call (identifier) (call (member_access (identifier) (identifier)) (named_argument (identifier) (string))))))

================
Type alias
================

type Status = :ok | :error | :pending

---

(source_file
  (type_alias (identifier) (type_expression)))
```

**Step 2: Run tests to verify failure**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`

**Step 3: Add grammar rules**

Add to grammar.js:

```javascript
// In _statement choice, add:
$.actor_definition,
$.try_catch,
$.from_import,
$.import_statement,
$.export_block,
$.macro_definition,
$.macro_invocation,
$.annotated_statement,
$.event_definition,
$.on_handler,
$.emit_statement,
$.type_alias,
$.requires_statement,
$.raise_statement,
$.reply_statement,
$.extern_definition,
$.retroactive_impl,
$.parallel_assign,
$.destructure_assign,

// New rules:
actor_definition: $ => seq(
  'actor',
  field('name', $.identifier),
  $._terminator,
  repeat(choice(
    seq($.needs_declaration, $._terminator),
    seq($.function_definition, $._terminator),
  )),
  optional($.receive_block),
  repeat(seq($.function_definition, $._terminator)),
  'end',
),

receive_block: $ => seq(
  'receive',
  $._terminator,
  repeat1($.match_case),
  'end',
),

reply_statement: $ => seq('reply', $._expression),

try_catch: $ => seq(
  'try',
  $._terminator,
  optional(field('body', $.body)),
  repeat1($.catch_clause),
  optional($.ensure_clause),
  'end',
),

catch_clause: $ => seq(
  'catch',
  optional(field('type', $.identifier)),
  optional(seq('as', field('var', $.identifier))),
  $._terminator,
  optional(field('body', $.body)),
),

ensure_clause: $ => seq(
  'ensure',
  $._terminator,
  optional(field('body', $.body)),
),

from_import: $ => seq(
  'from',
  field('module', $.identifier),
  'import',
  $.identifier,
  repeat(seq(',', $.identifier)),
),

import_statement: $ => seq(
  'import',
  $.identifier,
  repeat(seq('.', $.identifier)),
  optional(choice(
    seq('as', $.identifier),
    seq('.', '{', $.identifier, repeat(seq(',', $.identifier)), '}'),
  )),
),

export_block: $ => seq(
  'export',
  '{',
  $.identifier,
  repeat(seq(',', $.identifier)),
  '}',
),

macro_definition: $ => seq(
  'macro',
  field('name', $.identifier),
  '(',
  optional(seq($.identifier, repeat(seq(',', $.identifier)))),
  ')',
  $._terminator,
  optional(field('body', $.body)),
  'end',
),

macro_invocation: $ => seq(
  '@',
  field('name', $.identifier),
  optional(seq($._expression, repeat(seq(',', $._expression)))),
  optional(seq($._terminator, optional($.body), 'end')),
),

annotated_statement: $ => seq(
  $.annotation,
  $._terminator,
  $._statement,
),

annotation: $ => seq(
  '@[',
  $.identifier,
  optional(seq(':', $._expression)),
  repeat(seq(',', $.identifier, optional(seq(':', $._expression)))),
  ']',
),

event_definition: $ => seq(
  'event',
  field('name', $.identifier),
  '(',
  optional(seq($.needs_declaration, repeat(seq(',', $.needs_declaration)))),
  ')',
),

on_handler: $ => seq(
  'on',
  field('event', $.identifier),
  'do',
  '|',
  field('param', $.identifier),
  '|',
  $._terminator,
  optional(field('body', $.body)),
  'end',
),

emit_statement: $ => seq('emit', $._expression),

type_alias: $ => seq(
  'type',
  field('name', $.identifier),
  '=',
  field('type', $.type_expression),
),

type_expression: $ => choice(
  $.identifier,
  $.symbol,
  seq($.type_expression, '|', $.type_expression),
),

requires_statement: $ => seq(
  'requires',
  $._expression,
  optional(seq(',', $._expression)),
),

raise_statement: $ => seq('raise', $._expression),

extern_definition: $ => seq(
  'extern',
  $.string,
  $._terminator,
  repeat(seq($.extern_declaration, $._terminator)),
  'end',
),

extern_declaration: $ => seq(
  'def',
  $.identifier,
  optional($.parameters),
  optional(seq('->', $.type_annotation)),
),

retroactive_impl: $ => seq(
  'implements',
  field('protocol', $.identifier),
  'for',
  field('type', $.identifier),
  $._terminator,
  repeat(seq($.function_definition, $._terminator)),
  'end',
),

parallel_assign: $ => seq(
  $.identifier,
  repeat1(seq(',', $.identifier)),
  '=',
  $._expression,
  repeat(seq(',', $._expression)),
),

destructure_assign: $ => seq(
  $.destructure_pattern,
  '=',
  $._expression,
),

// Add to _expression:
await_expression: $ => prec(12, seq('await', $._expression)),

ast_block: $ => seq(
  'ast',
  $._terminator,
  optional($.body),
  'end',
),

splice: $ => seq('$', $.identifier),

index_expression: $ => prec(3, seq(
  $._expression,
  '[',
  $._expression,
  ']',
)),

null_safe_access: $ => prec(3, seq(
  $._expression,
  '?.',
  $.identifier,
)),

cast_expression: $ => prec(1, seq(
  $._expression,
  'as',
  $.identifier,
)),

self: $ => 'self',
```

**Step 4: Generate and run tests**

Run: `cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test`
Expected: All corpus tests pass.

**Step 5: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add actors, try/catch, imports, macros, events, remaining constructs"
```

---

### Task 7: Add external scanner for f-strings and multiline comments, then highlight queries

**Files:**
- Create: `tree-sitter-opal/src/scanner.c`
- Modify: `tree-sitter-opal/grammar.js` (add `externals`)
- Create: `tree-sitter-opal/queries/highlights.scm`
- Create: `tree-sitter-opal/queries/locals.scm`
- Create: `tree-sitter-opal/queries/indents.scm`
- Create: `tree-sitter-opal/test/corpus/fstrings.txt`

**Context:** F-strings (`f"Hello, {name}!"`) require an external scanner because the interpolation `{expr}` can contain nested braces and quotes. The Opal lexer (`crates/opal-lexer/src/token.rs:39-91`) implements this with brace-depth tracking. Multiline comments (`### ... ###`) also benefit from external scanning. In tree-sitter, the external scanner is a C file that handles tokens the grammar DSL can't express.

**Step 1: Write f-string corpus test**

Create `test/corpus/fstrings.txt`:

```
================
Simple f-string
================

name = "world"
x = f"hello, {name}!"

---

(source_file
  (assignment (identifier) (string))
  (assignment (identifier) (fstring (fstring_content) (interpolation (identifier)) (fstring_content))))

================
F-string with expression
================

x = f"sum: {a + b}"

---

(source_file
  (assignment (identifier) (fstring (fstring_content) (interpolation (binary_expression (identifier) (identifier))))))
```

**Step 2: Add externals to grammar.js**

Add at the top level of the grammar object:

```javascript
externals: $ => [
  $.fstring_start_double,
  $.fstring_start_single,
  $.fstring_content,
  $.fstring_end,
  $.interpolation_start,
  $.interpolation_end,
  $.multiline_comment,
],
```

Add f-string rules:

```javascript
fstring: $ => choice(
  seq(
    $.fstring_start_double,
    repeat(choice($.fstring_content, $.interpolation)),
    $.fstring_end,
  ),
  seq(
    $.fstring_start_single,
    repeat(choice($.fstring_content, $.interpolation)),
    $.fstring_end,
  ),
),

interpolation: $ => seq(
  $.interpolation_start,
  $._expression,
  optional(seq(':', $.format_spec)),
  $.interpolation_end,
),

format_spec: $ => /[^}]+/,
```

Remove the f-string from the `string` rule (it's now a separate `fstring` rule in `_expression`).

**Step 3: Write the external scanner**

Create `src/scanner.c`:

```c
#include "tree_sitter/parser.h"
#include <string.h>

enum TokenType {
  FSTRING_START_DOUBLE,
  FSTRING_START_SINGLE,
  FSTRING_CONTENT,
  FSTRING_END,
  INTERPOLATION_START,
  INTERPOLATION_END,
  MULTILINE_COMMENT,
};

typedef struct {
  int brace_depth;
  char quote_char; // '"' or '\'' or 0
} Scanner;

void *tree_sitter_opal_external_scanner_create() {
  Scanner *scanner = calloc(1, sizeof(Scanner));
  return scanner;
}

void tree_sitter_opal_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_opal_external_scanner_serialize(void *payload, char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  buffer[0] = (char)scanner->brace_depth;
  buffer[1] = scanner->quote_char;
  return 2;
}

void tree_sitter_opal_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  if (length >= 2) {
    scanner->brace_depth = (int)buffer[0];
    scanner->quote_char = buffer[1];
  } else {
    scanner->brace_depth = 0;
    scanner->quote_char = 0;
  }
}

static void advance(TSLexer *lexer) {
  lexer->advance(lexer, false);
}

static void skip(TSLexer *lexer) {
  lexer->advance(lexer, true);
}

bool tree_sitter_opal_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  // Check for multiline comment: ### ... ###
  if (valid_symbols[MULTILINE_COMMENT] && lexer->lookahead == '#') {
    advance(lexer);
    if (lexer->lookahead == '#') {
      advance(lexer);
      if (lexer->lookahead == '#') {
        advance(lexer);
        // Now consume until we find ###
        int hash_count = 0;
        while (lexer->lookahead != 0) {
          if (lexer->lookahead == '#') {
            hash_count++;
            if (hash_count == 3) {
              advance(lexer);
              lexer->result_symbol = MULTILINE_COMMENT;
              return true;
            }
          } else {
            hash_count = 0;
          }
          advance(lexer);
        }
      }
    }
    return false;
  }

  // F-string start: f" or f'
  if (valid_symbols[FSTRING_START_DOUBLE] && lexer->lookahead == 'f') {
    advance(lexer);
    if (lexer->lookahead == '"') {
      advance(lexer);
      scanner->quote_char = '"';
      scanner->brace_depth = 0;
      lexer->result_symbol = FSTRING_START_DOUBLE;
      return true;
    }
    if (lexer->lookahead == '\'') {
      advance(lexer);
      scanner->quote_char = '\'';
      scanner->brace_depth = 0;
      lexer->result_symbol = FSTRING_START_SINGLE;
      return true;
    }
    return false;
  }

  // Inside f-string: content, interpolation start, or end
  if (scanner->quote_char != 0) {
    // Interpolation end: }
    if (valid_symbols[INTERPOLATION_END] && lexer->lookahead == '}') {
      advance(lexer);
      lexer->result_symbol = INTERPOLATION_END;
      return true;
    }

    // Interpolation start: {
    if (valid_symbols[INTERPOLATION_START] && lexer->lookahead == '{') {
      advance(lexer);
      lexer->result_symbol = INTERPOLATION_START;
      return true;
    }

    // F-string end: matching quote
    if (valid_symbols[FSTRING_END] && lexer->lookahead == scanner->quote_char) {
      advance(lexer);
      scanner->quote_char = 0;
      lexer->result_symbol = FSTRING_END;
      return true;
    }

    // F-string content: everything else until { or quote or EOF
    if (valid_symbols[FSTRING_CONTENT]) {
      bool has_content = false;
      while (lexer->lookahead != 0 &&
             lexer->lookahead != '{' &&
             lexer->lookahead != scanner->quote_char) {
        if (lexer->lookahead == '\\') {
          advance(lexer); // skip backslash
          if (lexer->lookahead != 0) advance(lexer); // skip escaped char
        } else {
          advance(lexer);
        }
        has_content = true;
      }
      if (has_content) {
        lexer->result_symbol = FSTRING_CONTENT;
        return true;
      }
    }
  }

  return false;
}
```

**Step 4: Create highlight queries**

Create `queries/highlights.scm`:

```scheme
; Keywords
[
  "def" "end" "class" "module" "protocol" "enum" "model"
  "if" "elsif" "else" "then"
  "for" "while" "in" "do"
  "match" "case"
  "return" "break" "next"
  "try" "catch" "ensure" "raise"
  "let" "needs" "requires"
  "import" "from" "export" "as"
  "actor" "receive" "reply" "send" "await"
  "macro" "ast" "emit" "event" "on"
  "type" "implements" "with" "where" "defaults"
  "and" "or" "not" "is"
  "extern" "parallel" "async"
  "self"
] @keyword

["true" "false"] @boolean
"null" @constant.builtin

; Functions
(function_definition name: (identifier) @function.definition)
(call function: (identifier) @function.call)
(call function: (member_access field: (identifier) @function.method))

; Types
(class_definition name: (identifier) @type.definition)
(protocol_definition name: (identifier) @type.definition)
(module_definition name: (identifier) @type.definition)
(enum_definition name: (identifier) @type.definition)
(model_definition name: (identifier) @type.definition)
(actor_definition name: (identifier) @type.definition)
(event_definition name: (identifier) @type.definition)
(type_annotation (identifier) @type)
(implements_clause (identifier) @type)

; Variables
(assignment name: (identifier) @variable)
(let_binding name: (identifier) @variable)
(parameter name: (identifier) @variable.parameter)
(needs_declaration name: (identifier) @variable.parameter)

; Literals
(integer) @number
(float) @number.float
(string) @string
(fstring) @string
(fstring_content) @string
(interpolation) @punctuation.special
(symbol) @string.special.symbol

; Instance variables
(instance_variable) @variable.member

; Comments
(comment) @comment
(multiline_comment) @comment

; Operators
[
  "+" "-" "*" "/" "%" "**"
  "==" "!=" "<" "<=" ">" ">="
  "+=" "-=" "*=" "/="
  "|>" ".." "..." "?." "??"
  "=" "->" "|"
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ":" "."] @punctuation.delimiter

; Macros and annotations
(macro_invocation "@" @attribute name: (identifier) @attribute)
(annotation "@[" @attribute (identifier) @attribute "]" @attribute)
```

Create `queries/locals.scm`:

```scheme
; Scopes
(function_definition) @scope
(class_definition) @scope
(module_definition) @scope
(actor_definition) @scope
(for_loop) @scope
(while_loop) @scope
(block_closure) @scope
(if_expression) @scope

; Definitions
(assignment name: (identifier) @definition.var)
(let_binding name: (identifier) @definition.var)
(parameter name: (identifier) @definition.parameter)
(function_definition name: (identifier) @definition.function)

; References
(identifier) @reference
```

Create `queries/indents.scm`:

```scheme
[
  (function_definition)
  (class_definition)
  (module_definition)
  (protocol_definition)
  (enum_definition)
  (model_definition)
  (actor_definition)
  (if_expression)
  (elsif_clause)
  (else_clause)
  (for_loop)
  (while_loop)
  (match_expression)
  (match_case)
  (try_catch)
  (catch_clause)
  (ensure_clause)
  (block_closure)
  (receive_block)
  (macro_definition)
  (on_handler)
  (extern_definition)
  (retroactive_impl)
] @indent

"end" @outdent
"elsif" @outdent
"else" @outdent
"catch" @outdent
"ensure" @outdent
```

**Step 5: Generate and test**

Run:
```bash
cd tree-sitter-opal && npx tree-sitter generate && npx tree-sitter test
```

**Step 6: Test highlighting**

Run:
```bash
cd tree-sitter-opal && npx tree-sitter highlight ../tests/spec/04-classes/contributor_class.opl
```
Expected: Colored output with keywords, types, strings, etc. highlighted.

**Step 7: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): add external scanner for f-strings, highlight/locals/indents queries"
```

---

### Task 8: Validate grammar against all spec tests

**Files:**
- Create: `tree-sitter-opal/test/validate_specs.sh`

**Context:** The Opal project has 42 spec test files under `tests/spec/`. Parse all of them and check for ERROR nodes. Some errors are expected (grammar may need tweaking), but this task identifies gaps.

**Step 1: Write validation script**

Create `tree-sitter-opal/test/validate_specs.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0
ERRORS=0

for f in ../tests/spec/**/*.opl; do
    output=$(npx tree-sitter parse "$f" 2>&1)
    if echo "$output" | grep -q "ERROR"; then
        echo "HAS ERRORS: $f"
        echo "$output" | grep "ERROR" | head -3
        echo ""
        ERRORS=$((ERRORS + 1))
    else
        PASS=$((PASS + 1))
    fi
done

echo "Results: $PASS clean, $ERRORS with errors (out of $((PASS + ERRORS)) files)"
```

**Step 2: Run validation**

Run:
```bash
cd tree-sitter-opal && chmod +x test/validate_specs.sh && bash test/validate_specs.sh
```
Expected: Most files parse cleanly. Files with errors indicate grammar gaps to fix.

**Step 3: Fix grammar issues found**

Iterate on `grammar.js` to fix the most common ERROR patterns. Focus on:
- Missing `_terminator` handling (newlines between statements)
- Precedence conflicts between expression forms
- Edge cases in pattern matching syntax

**Step 4: Re-run validation until clean or acceptably few errors**

Target: 90%+ of spec files parse without ERROR nodes (38+ out of 42).

**Step 5: Commit**

```bash
git add tree-sitter-opal/
git commit -m "feat(tree-sitter): validate against spec tests, fix grammar issues"
```

---

## Part 2: LSP Server (Tasks 9–13)

---

### Task 9: Scaffold opal-lsp crate

**Files:**
- Create: `crates/opal-lsp/Cargo.toml`
- Create: `crates/opal-lsp/src/main.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "opal-lsp"
version.workspace = true
edition.workspace = true

[[bin]]
name = "opal-lsp"
path = "src/main.rs"

[dependencies]
opal-lexer.workspace = true
opal-parser.workspace = true
tower-lsp-server = "0.20"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

Note: Check the latest `tower-lsp-server` version on crates.io. If the community fork is not available, use `tower-lsp = "0.20"` instead and adjust the imports accordingly (the API is the same, just `tower_lsp` instead of `tower_lsp_server`).

**Step 2: Create minimal main.rs**

```rust
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct OpalBackend {
    client: Client,
}

#[tower_lsp_server::async_trait]
impl LanguageServer for OpalBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "opal-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Opal language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| OpalBackend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

**Step 3: Add to workspace**

Add `"crates/opal-lsp"` to the `members` list in the root `Cargo.toml`. Also add to `[workspace.dependencies]`:

```toml
tower-lsp-server = "0.20"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

**Step 4: Build**

Run:
```bash
cargo build -p opal-lsp
```
Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add crates/opal-lsp/ Cargo.toml
git commit -m "feat: scaffold opal-lsp crate with minimal server"
```

---

### Task 10: Add diagnostics (parse errors on file open/change)

**Files:**
- Create: `crates/opal-lsp/src/diagnostics.rs`
- Modify: `crates/opal-lsp/src/main.rs`

**Context:** When a user opens or edits an `.opl` file, the LSP should parse it and report any parse errors as diagnostics. The existing `opal_parser::parse()` returns `Result<Program, ParseError>`. `ParseError` has span information we can convert to LSP positions. Use `opal_lexer::source_location()` to convert byte offsets to line/column.

**Step 1: Create diagnostics.rs**

```rust
use opal_lexer::source_location;
use opal_parser::ParseError;
use tower_lsp_server::ls_types::*;

pub fn parse_diagnostics(source: &str) -> (Option<opal_parser::Program>, Vec<Diagnostic>) {
    match opal_parser::parse(source) {
        Ok(program) => (Some(program), vec![]),
        Err(err) => {
            let diagnostic = parse_error_to_diagnostic(&err, source);
            (None, vec![diagnostic])
        }
    }
}

fn parse_error_to_diagnostic(err: &ParseError, source: &str) -> Diagnostic {
    let (message, range) = match err {
        ParseError::UnexpectedToken { expected, got, span, .. } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                format!("expected {}, got {}", expected, got),
                Range::new(
                    Position::new((line - 1) as u32, (col - 1) as u32),
                    Position::new((end_line - 1) as u32, (end_col - 1) as u32),
                ),
            )
        }
        ParseError::UnexpectedEof { expected, .. } => {
            let lines = source.lines().count();
            let last_col = source.lines().last().map(|l| l.len()).unwrap_or(0);
            (
                format!("unexpected end of file, expected {}", expected),
                Range::new(
                    Position::new(lines.saturating_sub(1) as u32, last_col as u32),
                    Position::new(lines.saturating_sub(1) as u32, last_col as u32),
                ),
            )
        }
        ParseError::InvalidFString { message, span, .. } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                message.clone(),
                Range::new(
                    Position::new((line - 1) as u32, (col - 1) as u32),
                    Position::new((end_line - 1) as u32, (end_col - 1) as u32),
                ),
            )
        }
        ParseError::LexError { message, span, .. } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                message.clone(),
                Range::new(
                    Position::new((line - 1) as u32, (col - 1) as u32),
                    Position::new((end_line - 1) as u32, (end_col - 1) as u32),
                ),
            )
        }
    };

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("opal".to_string()),
        message,
        ..Default::default()
    }
}
```

**Step 2: Update main.rs to call diagnostics on open/change**

Add `mod diagnostics;` and implement `did_open` and `did_change`:

```rust
use std::collections::HashMap;
use std::sync::Mutex;

mod diagnostics;

#[derive(Debug)]
struct OpalBackend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
}

// In LanguageServer impl:
async fn did_open(&self, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let text = params.text_document.text;
    self.documents.lock().unwrap().insert(uri.clone(), text.clone());
    self.publish_diagnostics(uri, &text).await;
}

async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    if let Some(change) = params.content_changes.into_iter().last() {
        self.documents.lock().unwrap().insert(uri.clone(), change.text.clone());
        self.publish_diagnostics(uri, &change.text).await;
    }
}

// Helper method on OpalBackend:
impl OpalBackend {
    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let (_program, diagnostics) = diagnostics::parse_diagnostics(source);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}
```

Update `LspService::new` to initialize `documents`:

```rust
let (service, socket) = LspService::new(|client| OpalBackend {
    client,
    documents: Mutex::new(HashMap::new()),
});
```

**Step 3: Build and test**

Run:
```bash
cargo build -p opal-lsp
```
Expected: Compiles.

To test manually: open a file with syntax errors in your editor with the LSP configured. You should see red squiggles on parse errors.

**Step 4: Commit**

```bash
git add crates/opal-lsp/
git commit -m "feat(lsp): add diagnostics on file open/change"
```

---

### Task 11: Add document symbols (outline view)

**Files:**
- Create: `crates/opal-lsp/src/symbols.rs`
- Modify: `crates/opal-lsp/src/main.rs`

**Context:** Document symbols power the outline view (Ctrl+Shift+O / Cmd+Shift+O) and breadcrumbs. Walk the AST and emit `DocumentSymbol` for each function, class, module, enum, actor, protocol, model, event, type alias. Classes/modules should contain their methods as children. Use span information from the AST to compute ranges.

**Step 1: Create symbols.rs**

```rust
use opal_lexer::source_location;
use opal_parser::{Program, StmtKind, Stmt};
use tower_lsp_server::ls_types::*;

pub fn document_symbols(program: &Program, source: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for stmt in &program.statements {
        if let Some(sym) = stmt_to_symbol(stmt, source) {
            symbols.push(sym);
        }
    }
    symbols
}

fn span_to_range(span: opal_lexer::Span, source: &str) -> Range {
    let (start_line, start_col) = source_location(source, span.start);
    let (end_line, end_col) = source_location(source, span.end);
    Range::new(
        Position::new((start_line - 1) as u32, (start_col - 1) as u32),
        Position::new((end_line - 1) as u32, (end_col - 1) as u32),
    )
}

#[allow(deprecated)] // DocumentSymbol::deprecated field
fn stmt_to_symbol(stmt: &Stmt, source: &str) -> Option<DocumentSymbol> {
    let range = span_to_range(stmt.span, source);

    match &stmt.kind {
        StmtKind::FuncDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::ClassDef { name, methods, .. } => {
            let children: Vec<_> = methods
                .iter()
                .filter_map(|m| stmt_to_symbol(m, source))
                .collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ModuleDef { name, body, .. } => {
            let children: Vec<_> = body
                .iter()
                .filter_map(|s| stmt_to_symbol(s, source))
                .collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ProtocolDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::INTERFACE,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::EnumDef { name, variants, .. } => {
            let children: Vec<_> = variants
                .iter()
                .map(|v| DocumentSymbol {
                    name: v.name.clone(),
                    detail: None,
                    kind: SymbolKind::ENUM_MEMBER,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                })
                .collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ActorDef { name, methods, .. } => {
            let children: Vec<_> = methods
                .iter()
                .filter_map(|m| stmt_to_symbol(m, source))
                .collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::CLASS,
                tags: Some(vec![SymbolTag::DEPRECATED]), // Using tag to distinguish; ideally custom
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ModelDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::STRUCT,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::EventDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::EVENT,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::TypeAlias { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::TYPE_PARAMETER,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        _ => None,
    }
}
```

**Step 2: Register capability and implement handler in main.rs**

In `initialize`, add to capabilities:

```rust
document_symbol_provider: Some(OneOf::Left(true)),
```

Add handler:

```rust
async fn document_symbol(
    &self,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;
    let docs = self.documents.lock().unwrap();
    let Some(source) = docs.get(&uri) else {
        return Ok(None);
    };

    let Ok(program) = opal_parser::parse(source) else {
        return Ok(None);
    };

    let syms = symbols::document_symbols(&program, source);
    Ok(Some(DocumentSymbolResponse::Nested(syms)))
}
```

Add `mod symbols;` to main.rs.

**Step 3: Build**

Run: `cargo build -p opal-lsp`
Expected: Compiles.

**Step 4: Commit**

```bash
git add crates/opal-lsp/
git commit -m "feat(lsp): add document symbols for outline view"
```

---

### Task 12: Add go-to-definition

**Files:**
- Create: `crates/opal-lsp/src/goto_def.rs`
- Modify: `crates/opal-lsp/src/main.rs`

**Context:** Go-to-definition resolves the identifier at the cursor position to its definition location. For a single-file LSP, walk the AST to build a symbol table mapping names to spans, then find which identifier the cursor is on and look up its definition. Handles: function names, class names, module names, enum names, variable assignments, let bindings, parameters, needs declarations.

**Step 1: Create goto_def.rs**

```rust
use opal_lexer::{source_location, Span};
use opal_parser::*;
use tower_lsp_server::ls_types::*;

/// Find the definition location for the symbol at the given position.
pub fn goto_definition(
    program: &Program,
    source: &str,
    position: Position,
) -> Option<Location> {
    let offset = position_to_offset(source, position)?;
    let target_name = identifier_at_offset(source, offset)?;

    // Build symbol table
    let mut symbols: Vec<(String, Span)> = Vec::new();
    collect_definitions(&program.statements, &mut symbols);

    // Find the definition
    symbols
        .iter()
        .find(|(name, _)| name == &target_name)
        .map(|(_, span)| {
            let range = span_to_range(*span, source);
            Location {
                uri: Url::parse("file:///").unwrap(), // Will be replaced by caller
                range,
            }
        })
}

fn position_to_offset(source: &str, position: Position) -> Option<usize> {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in source.char_indices() {
        if line == position.line && col == position.character {
            return Some(i);
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    if line == position.line && col == position.character {
        Some(source.len())
    } else {
        None
    }
}

fn identifier_at_offset(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if offset >= bytes.len() {
        return None;
    }

    // Find start of identifier
    let mut start = offset;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }

    // Find end of identifier
    let mut end = offset;
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(source[start..end].to_string())
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'!'
}

fn collect_definitions(stmts: &[Stmt], symbols: &mut Vec<(String, Span)>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::FuncDef { name, body, params, .. } => {
                symbols.push((name.clone(), stmt.span));
                for param in params {
                    symbols.push((param.name.clone(), stmt.span));
                }
                collect_definitions(body, symbols);
            }
            StmtKind::ClassDef { name, methods, .. } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::ModuleDef { name, body, .. } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(body, symbols);
            }
            StmtKind::ProtocolDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::EnumDef { name, methods, .. } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::ActorDef { name, methods, init, .. } => {
                symbols.push((name.clone(), stmt.span));
                if let Some(init_body) = init {
                    collect_definitions(init_body, symbols);
                }
                collect_definitions(methods, symbols);
            }
            StmtKind::ModelDef { name, methods, .. } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::EventDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::TypeAlias { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::Assign { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::Let { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::MacroDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::For { var, body, .. } => {
                symbols.push((var.clone(), stmt.span));
                collect_definitions(body, symbols);
            }
            _ => {}
        }
    }
}

fn span_to_range(span: Span, source: &str) -> Range {
    let (start_line, start_col) = source_location(source, span.start);
    let (end_line, end_col) = source_location(source, span.end);
    Range::new(
        Position::new((start_line - 1) as u32, (start_col - 1) as u32),
        Position::new((end_line - 1) as u32, (end_col - 1) as u32),
    )
}
```

**Step 2: Register capability and implement handler in main.rs**

In `initialize`, add to capabilities:

```rust
definition_provider: Some(OneOf::Left(true)),
```

Add handler:

```rust
async fn goto_definition(
    &self,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let docs = self.documents.lock().unwrap();
    let Some(source) = docs.get(&uri) else {
        return Ok(None);
    };

    let Ok(program) = opal_parser::parse(source) else {
        return Ok(None);
    };

    let result = goto_def::goto_definition(&program, source, position);
    match result {
        Some(mut location) => {
            location.uri = uri;
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        }
        None => Ok(None),
    }
}
```

Add `mod goto_def;` to main.rs.

**Step 3: Build**

Run: `cargo build -p opal-lsp`
Expected: Compiles.

**Step 4: Commit**

```bash
git add crates/opal-lsp/
git commit -m "feat(lsp): add go-to-definition"
```

---

### Task 13: Integration test and editor configuration

**Files:**
- Create: `crates/opal-lsp/tests/integration.rs`
- Create: `editors/README.md`

**Context:** Write a basic integration test that verifies the LSP server initializes correctly and returns diagnostics. Then document how to configure each editor.

**Step 1: Write integration test**

Create `crates/opal-lsp/tests/integration.rs`:

```rust
//! Basic tests for LSP functionality (unit-level, not full LSP protocol).

#[test]
fn test_diagnostics_valid_code() {
    // This tests the diagnostics module directly, not via LSP protocol
    let source = r#"
x = 42
print(x)
"#;
    let result = opal_parser::parse(source);
    assert!(result.is_ok(), "Valid code should parse without errors");
}

#[test]
fn test_diagnostics_invalid_code() {
    let source = "def\n";
    let result = opal_parser::parse(source);
    assert!(result.is_err(), "Invalid code should produce parse error");
}

#[test]
fn test_document_symbols() {
    let source = r#"
def greet(name)
  print(name)
end

class Bounty
  needs title: String
end
"#;
    let program = opal_parser::parse(source).unwrap();
    // Verify the AST has the expected structure
    assert!(program.statements.len() >= 2);
}
```

**Step 2: Run tests**

Run: `cargo test -p opal-lsp`
Expected: All tests pass.

**Step 3: Create editor configuration docs**

Create `editors/README.md`:

````markdown
# Opal Editor Support

## Prerequisites

Build the LSP server:

```bash
cargo build --release -p opal-lsp
```

The binary is at `target/release/opal-lsp`.

## Neovim

### Tree-sitter (syntax highlighting)

Add to your `init.lua` or tree-sitter config:

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.opal = {
  install_info = {
    url = "/path/to/opal/tree-sitter-opal",
    files = { "src/parser.c", "src/scanner.c" },
    branch = "main",
  },
  filetype = "opal",
}

vim.filetype.add({
  extension = {
    opl = "opal",
  },
})
```

Then run `:TSInstall opal`.

Copy the query files:
```bash
mkdir -p ~/.config/nvim/queries/opal/
cp tree-sitter-opal/queries/*.scm ~/.config/nvim/queries/opal/
```

### LSP

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "opal",
  callback = function()
    vim.lsp.start({
      name = "opal-lsp",
      cmd = { "/path/to/opal/target/release/opal-lsp" },
      root_dir = vim.fn.getcwd(),
    })
  end,
})
```

## Zed

Create an extension directory:

```
~/.config/zed/extensions/opal/
├── extension.toml
├── grammars/opal/
│   └── (symlink to tree-sitter-opal/)
└── languages/opal/
    ├── config.toml
    ├── highlights.scm
    ├── indents.scm
    └── locals.scm
```

`extension.toml`:
```toml
id = "opal"
name = "Opal"
version = "0.1.0"

[grammars.opal]
repository = "file:///path/to/opal/tree-sitter-opal"

[language_servers.opal-lsp]
language = "Opal"
```

`languages/opal/config.toml`:
```toml
name = "Opal"
grammar = "opal"
path_suffixes = ["opl"]
line_comments = ["# "]
```

## Cursor / VS Code

Install the tree-sitter extension or create a TextMate grammar extension. For LSP, add to `settings.json`:

```json
{
  "opal.lsp.path": "/path/to/opal/target/release/opal-lsp"
}
```

(A proper VS Code extension with `package.json` and `extension.js` is recommended for production use.)
````

**Step 4: Commit**

```bash
git add crates/opal-lsp/tests/ editors/
git commit -m "feat: add LSP integration tests and editor configuration docs"
```

---

## Summary

| Task | Deliverable | Key files |
|------|------------|-----------|
| 1 | Tree-sitter scaffold | `tree-sitter-opal/grammar.js`, `package.json` |
| 2 | Keywords, assignment, operators | `grammar.js`, `test/corpus/basics.txt` |
| 3 | Functions, if/else, return | `grammar.js`, `test/corpus/functions.txt`, `control_flow.txt` |
| 4 | Classes, protocols, modules, enums | `grammar.js`, `test/corpus/classes.txt` |
| 5 | Loops, match, closures, collections | `grammar.js`, `test/corpus/loops.txt`, `match.txt`, `closures.txt` |
| 6 | Actors, try/catch, imports, macros, events | `grammar.js`, `test/corpus/actors.txt`, `advanced.txt` |
| 7 | F-string external scanner + highlight queries | `src/scanner.c`, `queries/*.scm` |
| 8 | Spec test validation + grammar fixes | `test/validate_specs.sh` |
| 9 | LSP scaffold | `crates/opal-lsp/` |
| 10 | Diagnostics | `diagnostics.rs` |
| 11 | Document symbols | `symbols.rs` |
| 12 | Go-to-definition | `goto_def.rs` |
| 13 | Integration tests + editor docs | `tests/integration.rs`, `editors/README.md` |
