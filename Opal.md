# Opal — Opinionated Programming Algorithmic Language

[...towards a better programming...](http://www.chris-granger.com/2014/03/27/toward-a-better-programming/)

Opal is a dynamic, interpreted, object-oriented language with first-class functions, multiple dispatch, an actor-based concurrency model, and a gradual type system. It prioritizes readability, explicitness, and demonstrating sound software engineering concepts.

---

## 1. Design Philosophy

- **Readability is paramount.** Code is read far more than it is written.
- **One explicit way.** There should be one obvious way to do something — no alternative syntax for the same operation.
- **Software engineering concepts are first-class.** Dependency injection, domain events, specifications, guards, null objects, validated models, settings, the actor model, and metaprogramming are built into the language, not bolted on.
- **Batteries included.** Built-in testing, mocking, fixtures, documentation generation, project scaffolding, and package management.
- **Gradual typing.** Write quick scripts with no annotations, then add types at module boundaries for safety.

---

## 2. Facts & Semantics

| Question | Answer |
|---|---|
| Direct pointer access? | No. |
| Data types? | Rich: integers, floats, chars, strings, booleans, null, symbols, lists, tuples, dicts, ranges, regex. |
| Static or dynamic? | Dynamic, interpreted. |
| Memory model? | Garbage collected (inherited from host runtime). |
| Concurrency model? | Actor model. |
| Primitives? | The least number of primitives as possible; most functionality comes from the standard library. |
| Paradigm? | Multi-paradigm: object-oriented with functional features, multiple dispatch, and actor concurrency. |
| FFI? | To be determined. |
| Unicode support? | Full. Variable names, string literals, and symbols may contain Unicode characters. |
| File extensions? | `.opl` for source, `.topl` for tests. |

---

## 3. Formal Grammar (BNF Excerpt)

```bnf
<program>       ::= <statement>*

<statement>     ::= <expression> NEWLINE
                   | <assignment>
                   | <conditional>
                   | <loop>
                   | <function_def>
                   | <class_def>
                   | <module_def>
                   | <match_expr>
                   | <try_expr>
                   | <actor_def>
                   | <supervisor_def>
                   | <parallel_expr>
                   | <async_expr>
                   | <event_def>
                   | <emit_expr>
                   | <on_handler>
                   | <macro_def>
                   | <quote_expr>
                   | <type_alias>
                   | <implements_for>
                   | <enum_def>
                   | <model_def>
                   | <import_stmt>
                   | <export_stmt>

<assignment>    ::= IDENTIFIER "=" <expression>

<expression>    ::= <literal>
                   | IDENTIFIER
                   | <expression> <binary_op> <expression>
                   | <unary_op> <expression>
                   | <expression> "." IDENTIFIER
                   | <expression> "." IDENTIFIER "(" <args> ")"
                   | <expression> "[" <expression> "]"
                   | <function_call>
                   | <lambda>
                   | <with_expr>
                   | "(" <expression> ")"

<literal>       ::= INTEGER | FLOAT | CHAR | STRING | BOOL | NULL
                   | SYMBOL | <list> | <tuple> | <dict> | <range> | <regex>
                   | <f_string> | <r_string> | <t_string>

<char>          ::= "'" ( CHAR_CONTENT | ESCAPE_SEQ ) "'"
<string>        ::= '"' ( STRING_CONTENT | ESCAPE_SEQ )* '"'
                   | '"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE )* '"""'
<f_string>      ::= 'f"' ( STRING_CONTENT | ESCAPE_SEQ | "{" <expression> FORMAT_SPEC? "}" )* '"'
                   | 'f"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE | "{" <expression> FORMAT_SPEC? "}" )* '"""'
<r_string>      ::= 'r"' RAW_CONTENT* '"'
                   | 'r"""' RAW_CONTENT* '"""'
<t_string>      ::= 't"' ( STRING_CONTENT | ESCAPE_SEQ | "{" <expression> "}" )* '"'
                   | 't"""' ( STRING_CONTENT | ESCAPE_SEQ | NEWLINE | "{" <expression> "}" )* '"""'
<format_spec>   ::= "=" | ":" FORMAT_CONTENT

<list>          ::= "[" <expression> ("," <expression>)* "]"
                   | "[" "]"
<tuple>         ::= "(" <expression> ("," <expression>)* ")"
                   | "()"
<dict>          ::= "{" <dict_entry> ("," <dict_entry>)* "}"
                   | "{:}"
<dict_entry>    ::= <expression> ":" <expression>
<range>         ::= <expression> ".." <expression>
                   | <expression> "..." <expression>
<regex>         ::= "/" REGEX_CONTENT "/" REGEX_FLAGS?

<symbol>        ::= ":" IDENTIFIER
                   | ":" '"' STRING_CONTENT '"'

<function_call> ::= IDENTIFIER "(" <args> ")"
                   | <expression> "." IDENTIFIER "(" <args> ")"
<args>          ::= <arg> ("," <arg>)*
<arg>           ::= <expression>
                   | IDENTIFIER ":" <expression>

<lambda>        ::= "|" <params> "|" <expression>
                   | "|" <params> "|" NEWLINE <block> "end"

<with_expr>     ::= <expression> "with" <dict>

<function_def>  ::= "def" IDENTIFIER "(" <params> ")" NEWLINE <block> "end"
<params>        ::= <param> ("," <param>)*
<param>         ::= IDENTIFIER
                   | IDENTIFIER "::" TYPE
                   | IDENTIFIER "=" <expression>

<conditional>   ::= "if" <expression> NEWLINE <block> ("else" NEWLINE <block>)? "end"
                   | "unless" <expression> NEWLINE <block> ("else" NEWLINE <block>)? "end"

<loop>          ::= "while" <expression> NEWLINE <block> "end"
                   | "until" <expression> NEWLINE <block> "end"
                   | "for" IDENTIFIER "in" <expression> NEWLINE <block> "end"

<destructure>   ::= "(" <destruct_target> ("," <destruct_target>)* ")" "=" <expression>
                   | "{" <dict_destruct> ("," <dict_destruct>)* "}" "=" <expression>
                   | "[" <destruct_target> ("|" IDENTIFIER)? "]" "=" <expression>
<destruct_target>::= IDENTIFIER | "_" | "(" <destruct_target> ("," <destruct_target>)* ")"
<dict_destruct> ::= IDENTIFIER ":" IDENTIFIER | IDENTIFIER "?:" IDENTIFIER

<operator_def>  ::= "def" OPERATOR "(" <params> ")" NEWLINE <block> "end"

<class_body>    ::= (<needs_decl> | <function_def> | <operator_def> | <assignment>)*

<module_def>    ::= "module" IDENTIFIER NEWLINE <module_body> "end"
<module_body>   ::= (<needs_decl> | <function_def> | <class_def> | <assignment> | <on_handler>)*

<match_expr>    ::= "match" <expression> NEWLINE <case_clause>+ "end"
<case_clause>   ::= "case" <pattern> NEWLINE <block>
<pattern>       ::= <literal>
                   | IDENTIFIER
                   | "_"
                   | IDENTIFIER "::" TYPE
                   | <tuple_pattern>
                   | <list_pattern>
                   | <dict_pattern>
                   | <enum_pattern>
                   | <pattern> "|" <pattern>
                   | <pattern> "as" IDENTIFIER
                   | <pattern> "if" <expression>
<tuple_pattern>  ::= "(" <pattern> ("," <pattern>)* ")"
<list_pattern>   ::= "[" "]"
                    | "[" <pattern> ("," <pattern>)* "]"
                    | "[" <pattern> "|" IDENTIFIER "]"
<dict_pattern>   ::= "{" <dict_pat_entry> ("," <dict_pat_entry>)* "}"
<dict_pat_entry> ::= IDENTIFIER ":" <pattern>
<enum_pattern>   ::= <module_path> "." IDENTIFIER ("(" <pattern> ("," <pattern>)* ")")?

<try_expr>      ::= "try" NEWLINE <block>
                     ("on" "fail" TYPE ("as" IDENTIFIER)? NEWLINE <block>)*
                     ("ensure" NEWLINE <block>)?
                     "end"

<actor_def>     ::= "actor" IDENTIFIER NEWLINE <actor_body> "end"
<actor_body>    ::= (<needs_decl> | <function_def> | <receive_clause>)*
<receive_clause>::= "receive" SYMBOL ("(" <params> ")")? NEWLINE <block> "end"

<supervisor_def>::= "supervisor" IDENTIFIER NEWLINE <supervisor_body> "end"
<supervisor_body>::= ("strategy" SYMBOL NEWLINE)?
                     ("max_restarts" INTEGER "within" INTEGER NEWLINE)?
                     ("supervise" <expression> NEWLINE)*

<parallel_expr> ::= "parallel" NEWLINE <block> "end"
                   | "parallel" ("max:" INTEGER)? "for" IDENTIFIER "in" <expression> NEWLINE <block> "end"

<async_expr>    ::= "async" <expression>
<await_expr>    ::= "await" <expression>

<needs_decl>    ::= "needs" IDENTIFIER "::" TYPE ("=" <expression>)?
<event_def>     ::= "event" IDENTIFIER "(" <params> ")"
<emit_expr>     ::= "emit" <expression> ("await")?
<on_handler>    ::= "on" TYPE "do" "|" IDENTIFIER "|" NEWLINE <block> "end"

<macro_def>     ::= "macro" IDENTIFIER "(" <params> ")" NEWLINE <block> "end"
<macro_invoke>  ::= "@" IDENTIFIER <args>?
<quote_expr>    ::= "quote" <expression> "end"
                   | "quote" NEWLINE <block> "end"

<type_alias>    ::= "type" IDENTIFIER ("(" <type_params> ")")? "=" <type_expr>
<type_expr>     ::= TYPE
                   | <type_expr> "|" <type_expr>
                   | TYPE "(" <type_args> ")"
                   | TYPE "?"
                   | "|" <type_list> "|" "->" <type_expr>
<type_params>   ::= <type_param> ("," <type_param>)*
<type_param>    ::= IDENTIFIER ("implements" TYPE ("," TYPE)*)?
<type_args>     ::= <type_expr> ("," <type_expr>)*
<where_clause>  ::= "where" <constraint> ("," <constraint>)*
<constraint>    ::= IDENTIFIER "implements" TYPE ("," TYPE)*

<implements_for>::= "implements" TYPE "for" TYPE NEWLINE <class_body> "end"

<enum_def>      ::= "enum" IDENTIFIER ("(" <type_params> ")")?
                     ("implements" TYPE ("," TYPE)*)? NEWLINE
                     <variant>+ <function_def>* "end"
<variant>       ::= IDENTIFIER ("(" <params> ")")?

<model_def>     ::= "model" IDENTIFIER ("(" <type_params> ")")?
                     ("as" IDENTIFIER)? NEWLINE
                     (<needs_decl> | <where_field> | <validate_block> | <function_def>)* "end"
<where_field>   ::= "needs" IDENTIFIER "::" TYPE ("=" <expression>)?
                     "where" <field_constraint> ("," <field_constraint>)*
<field_constraint> ::= <lambda> | IDENTIFIER ("(" <args> ")")?
<validate_block>::= "validate" "do" NEWLINE <block> "end"

<import_stmt>   ::= "import" <module_path>
                   | "import" <module_path> "as" IDENTIFIER
                   | "import" <module_path> ".{" <import_list> "}"
<import_list>   ::= <import_item> ("," <import_item>)*
<import_item>   ::= IDENTIFIER
                   | IDENTIFIER "as" IDENTIFIER
<module_path>   ::= IDENTIFIER ("." IDENTIFIER)*
<export_stmt>   ::= "export" <module_path> ".{" <import_list> "}"

<is_expr>       ::= <expression> "is" TYPE
<propagate_expr>::= <expression> "!"

<class_def>     ::= "class" IDENTIFIER ("(" <type_params> ")")? ("<" IDENTIFIER)?
                     (<where_clause>)? NEWLINE <class_body> "end"
<null_object_def> ::= "class" IDENTIFIER "as" IDENTIFIER "defaults" <dict>

<block>         ::= <statement>+

<binary_op>     ::= "+" | "-" | "*" | "/" | "%" | "**"
                   | "==" | "!=" | "<" | ">" | "<=" | ">="
                   | "and" | "or"
                   | ".." | "..."
<unary_op>      ::= "-" | "not"
```

---

## 4. Basics

### 4.1 Comments

Single-line comments begin with `#`. Multiline comments are delimited by `#{` and `}#`.

```opal
# This is a single-line comment

#{
  This is a multiline comment.
  It can span as many lines as needed.
}#

x = 42  # inline comment
```

### 4.2 Variables & Assignment

Variables are dynamically typed and need no declaration keyword. Unicode identifiers are supported and encouraged.

```opal
pi = 3.14
𝛑 = 3.14
alpha = 1

# Parallel assignment
x, y = 1, 2

# Swap
x, y = y, x
```

Variable naming conventions:
- `snake_case` for local variables and functions
- `PascalCase` for classes, modules, and actors
- `SCREAMING_SNAKE` for constants
- `.name` for instance variables (inside classes)
- `:name` for symbols

### 4.3 Literals

#### 4.3.1 Null

```opal
value = null
```

#### 4.3.2 Booleans

```opal
are_you_here = true
are_you_there = false
```

#### 4.3.3 Numbers

```opal
# Integers
total = 10                   # Int32 by default
another_total = 11 as Int16  # explicit type cast
big = 1_000_000              # underscores for readability

# Floats
price = 22.3                 # Float32 by default
another_price = 236.70 as Float64
au = 149.700e9               # scientific notation -> Float64
```

#### 4.3.4 Characters

Characters use single quotes. A char is a single Unicode code point.

```opal
'a'
'ሴ'
'\''       # single quote
'\\'       # backslash
'\n'       # newline
'\t'       # tab
'\r'       # carriage return
'\e'       # escape
'\f'       # form feed
'\v'       # vertical tab

# Octal code point (up to three digits)
'\101'     # == 'A'
'\123'     # == 'S'

# Unicode code point (four hex digits)
'\u0041'   # == 'A'

# Unicode code point (up to six hex digits in braces)
'\u{41}'   # == 'A'
'\u{1F52E}'# == '🔮'
```

#### 4.3.5 Strings

Strings are immutable sequences of characters. They use double quotes. Opal provides several string prefixes for different use cases.

**Regular strings** — double quotes, supports escape sequences:

```opal
name = "claudio"
move_message = "my move is ♘ to ♚"

# Escape sequences: \n, \t, \\, \", etc. (same as chars)
tab_separated = "col1\tcol2\tcol3"
```

**Triple-quoted strings** — multiline without escaping:

```opal
query = """
  SELECT name, age
  FROM users
  WHERE active = true
"""

poem = """
  Roses are red,
  Violets are blue,
  Opal is readable,
  And so are you.
"""
```

Backslash continuation still works for joining lines without newlines:

```opal
hello = "hello \
         world"  # => "hello world"
```

**f-strings** — string interpolation with embedded expressions:

```opal
greeting = f"Hi {name}, welcome!"
result = f"The answer is {40 + 2}."

# Expressions can include method calls and nested quotes
report = f"Found {users.filter(|u| u.active?()).length} active users"

# Debug specifier with = (prints expression and its value)
x = 42
print(f"{x=}")          # => "x=42"
print(f"{x * 2=}")      # => "x * 2=84"
print(f"{name=}")       # => "name=claudio"

# Format specifiers with :
pi = 3.14159
print(f"{pi:.2}")       # => "3.14"
print(f"{amount:>10}")  # => "     42.50"

# Multiline f-strings
summary = f"""
  Name: {person.name}
  Age:  {person.age}
  Role: {person.role}
"""
```

**r-strings** — raw strings, no escape processing:

```opal
# Useful for regex patterns
pattern = r"\d{3}-\d{4}"

# Useful for file paths
path = r"C:\Users\claudio\documents"

# Without r-prefix, you'd need to double-escape
path_escaped = "C:\\Users\\claudio\\documents"  # equivalent

# Multiline raw strings
raw_block = r"""
  No \n escape \t processing here.
  Everything is literal.
"""
```

**t-strings** — template strings for safe interpolation:

```opal
# t-strings return a Template object instead of a string.
# This enables libraries to process interpolations safely.

# Safe HTML (library escapes values before inserting)
username = "<script>alert('xss')</script>"
page = html(t"<p>Hello, {username}</p>")
# => "<p>Hello, &lt;script&gt;alert('xss')&lt;/script&gt;</p>"

# Safe SQL (library uses parameterized queries)
id = 42
query = db.prepare(t"SELECT * FROM users WHERE id = {id}")
# => parameterized query, not string concatenation

# Multiline template
email = mailer.render(t"""
  Dear {customer.name},

  Your order #{order.id} has been shipped.
  Expected delivery: {order.delivery_date}.
""")
```

Template strings give libraries control over how interpolated values are processed — preventing injection vulnerabilities by design.

**String prefix summary:**

| Prefix | Purpose | Returns |
|---|---|---|
| (none) | Regular string with escapes | `String` |
| `f` | Interpolation with expressions | `String` |
| `r` | Raw, no escape processing | `String` |
| `t` | Template for safe interpolation | `Template` |

#### 4.3.6 Symbols

Symbols are self-identifying constants. They do not need to be assigned a value.

```opal
:hi
:bye
:"I have spaces."
:really?
:yes!
```

### 4.4 Operators

> See [Self-Hosting Foundations](docs/features/self-hosting-foundations.md) for the operator overloading design rationale.

#### Arithmetic
| Operator | Description |
|---|---|
| `+` | Addition |
| `-` | Subtraction / Unary negation |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |
| `**` | Exponentiation |

#### Comparison
| Operator | Description |
|---|---|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

#### Logical
| Operator | Description |
|---|---|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT |

#### Assignment
| Operator | Description |
|---|---|
| `=` | Assignment |
| `+=` | Add and assign |
| `-=` | Subtract and assign |
| `*=` | Multiply and assign |
| `/=` | Divide and assign |

```opal
# Arithmetic
2 ** 10          # => 1024
17 % 5           # => 2

# Comparison chaining
1 < x and x < 10

# Logical
ready = loaded and not errored
```

#### Operator Overloading

Operators are methods. The method form (inside a class) is sugar for the standalone form. Both use the same multiple dispatch mechanism.

```opal
class Vector
  needs x::Float64
  needs y::Float64

  # Arithmetic operators as methods
  def +(other::Vector) -> Vector
    Vector.new(x: .x + other.x, y: .y + other.y)
  end

  def -() -> Vector  # unary negation
    Vector.new(x: -.x, y: -.y)
  end

  # Indexing
  def [](index::Int32) -> Float64
    if index == 0 then .x else .y end
  end

  def []=(index::Int32, value::Float64)
    if index == 0 then .x = value else .y = value end
  end

  # Comparison
  def ==(other::Vector) -> Bool
    .x == other.x and .y == other.y
  end

  # String representation (used by f-strings and print)
  def to_string() -> String
    f"({.x}, {.y})"
  end
end

a = Vector.new(x: 1.0, y: 2.0)
b = Vector.new(x: 3.0, y: 4.0)
c = a + b          # => (4.0, 6.0)
a[0]               # => 1.0
print(f"result: {c}")  # => "result: (4.0, 6.0)"
```

```opal
# Standalone form — for cross-type operators and third-party extension
def *(scalar::Float64, v::Vector) -> Vector
  Vector.new(x: scalar * v.x, y: scalar * v.y)
end

def *(v::Vector, scalar::Float64) -> Vector
  scalar * v
end

2.0 * a   # => (2.0, 4.0) — standalone dispatch
```

The method form `def +(other::T)` inside a class is sugar for `def +(self::Self, other::T)`. Same dispatch resolution as regular functions.

**Overloadable operators:**

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `**`, unary `-` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Indexing | `[]`, `[]=` |
| Conversion | `to_string()`, `to_bool()`, `iter()` |

**Not overloadable** (language semantics): `=`, `and`, `or`, `not`, `..`, `...`, `is`, `as`.

### 4.5 Collections

#### 4.5.1 Lists

Lists are ordered, mutable sequences.

```opal
[]                        # empty list
numbers = [1, 2, 3, 4, 5]
names = ["alice", "bob"]
mixed = [1, "hello", :ok] # List(Int32 | String | Symbol)

# Access
numbers[0]                # => 1
numbers[-1]               # => 5

# Common operations
numbers.length            # => 5
numbers.push(6)           # [1, 2, 3, 4, 5, 6]
numbers.map(|x| x * 2)   # [2, 4, 6, 8, 10]
numbers.filter(|x| x > 3)           # [4, 5]
numbers.reduce(0, |acc, x| acc + x)  # 15
```

#### 4.5.2 Tuples

Tuples are ordered, immutable sequences. They use parentheses.

```opal
()                              # empty tuple
point = (10, 20)                # Tuple(Int32, Int32)
record = (:banana, "apple", '🙈')  # Tuple(Symbol, String, Char)

record[0]                       # => :banana
record[1]                       # => "apple"
record[2]                       # => '🙈'
```

#### 4.5.3 Dictionaries

Dictionaries are mutable mappings of key-value pairs. Keys can be any immutable object and must be unique.

```opal
{:}                             # empty dict
{1: 2, 3: 4}                   # Dict(Int32, Int32)
{1: 2, "a": 3}                 # Dict(Int32 | String, Int32)
{"α": "alpha", "β": "beta"}    # Dict(String, String)
{:plane: "✈", :train: "🚂"}    # Dict(Symbol, String)

# Access
ages = {"alice": 30, "bob": 25}
ages["alice"]                   # => 30
ages["carol"] = 28              # insert new entry
```

#### 4.5.4 Ranges

A range is constructed with a range literal. Types on both extremes must be the same.

```opal
1..10       # inclusive range: 1, 2, 3, ..., 10
1...10      # exclusive range: 1, 2, 3, ..., 9
'a'..'z'    # character range

# Ranges are iterable
for i in 1..5
  print(i)
end
```

### 4.6 Regex

Regular expressions use the `Regex` class, typically created with a literal delimited by `/`.

```opal
foo_or_bar = /foo|bar/
heEello    = /h(e+)llo/i
integer    = /\d+/

# Modifiers:
#   i  — ignore case (PCRE_CASELESS)
#   m  — multiline (PCRE_MULTILINE)
#   x  — extended (PCRE_EXTENDED)

# Usage
if "hello" =~ /h(e+)llo/
  print("matched!")
end

"foo bar baz".scan(/\w+/)  # => ["foo", "bar", "baz"]
```

### 4.7 Destructuring Assignment

> See [Self-Hosting Foundations](docs/features/self-hosting-foundations.md) for the destructuring design rationale.

Pattern matching syntax extended to regular assignment, function parameters, `for` loops, and closures. Same patterns as `match` — one way to do it everywhere.

#### Tuples

```opal
(x, y) = get_point()
(status, body) = http_get("/users")

# Ignore with _
(_, y) = get_point()

# Nested
(first, (a, b)) = (1, (2, 3))
# first = 1, a = 2, b = 3
```

#### Dicts

```opal
{name: n, age: a} = {name: "claudio", age: 15, role: "admin"}
# n = "claudio", a = 15 (extra keys ignored)

# Optional keys with ?
{name: n, age?: a} = {name: "claudio"}
# n = "claudio", a = null
```

#### Lists (head/tail)

```opal
[first, second | rest] = [1, 2, 3, 4, 5]
# first = 1, second = 2, rest = [3, 4, 5]

[head | _] = [10, 20, 30]
# head = 10
```

#### In Function Parameters

```opal
def distance((x1, y1), (x2, y2))
  ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
end

distance((0, 0), (3, 4))  # => 5.0
```

#### In For Loops and Closures

```opal
pairs = [("alice", 30), ("bob", 25)]
for (name, age) in pairs
  print(f"{name} is {age}")
end

points.map(|(x, y)| x + y)
```

**Rules:**
- `_` ignores a value.
- `[head | tail]` splits a list into first element(s) and rest.
- Dict destructuring extracts by key; extra keys are ignored. Missing required keys = runtime error.
- `?` suffix on a dict key makes it optional (null if missing).

---

## 5. Control Flow

### 5.1 Conditionals

```opal
# if / else
if a == b
  c = 1
else
  c = 2
end

# unless (inverted if)
unless a != b
  c = 1
else
  c = 2
end

# Suffix form (single expression)
print("even") if n % 2 == 0
print("odd") unless n % 2 == 0

# Ternary-style inline
status = if active then "on" else "off" end
```

### 5.2 Loops & Iteration

```opal
# while
while count < 10
  count += 1
end

# until (inverted while)
until count >= 10
  count += 1
end

# for-in
for item in [1, 2, 3]
  print(item)
end

for char in 'a'..'z'
  print(char)
end

# Loop with index
for item, index in ["a", "b", "c"].with_index()
  print(f"{index}: {item}")
end

# break and next (skip)
for n in 1..100
  next if n % 2 == 0
  break if n > 50
  print(n)
end
```

### 5.3 Pattern Matching

Opal's `match` expression supports a rich set of pattern forms. Patterns are tried top-to-bottom; the first matching case wins. Destructuring in patterns mirrors [4.7 Destructuring Assignment](#47-destructuring-assignment) but within a match context.

#### Literals & Ranges

```opal
match value
  case 0
    "zero"
  case 1..10
    "small"
  case "hello"
    "greeting"
  case :ok
    "symbol"
  case null
    "nothing"
  case true
    "yes"
  case _
    "other"
end
```

#### Variable Binding & Type Matching

```opal
match response
  case s::String
    print(s)
  case n::Int32
    print(f"code: {n}")
  case (status, body)
    print(f"{status}: {body}")
end
```

#### Or-Patterns

Use `|` to match any of several alternatives in a single case arm. Or-patterns cannot bind variables (it would be ambiguous which alternative produced the binding).

```opal
match status_code
  case 200 | 201 | 204
    "success"
  case 301 | 302
    "redirect"
  case 400 | 404 | 422
    "client error"
  case _
    "other"
end

# Works with enums
match direction
  case Direction.North | Direction.South
    "vertical"
  case Direction.East | Direction.West
    "horizontal"
end
```

#### Tuple & Dict Destructuring

Tuple patterns work like [4.7 Destructuring Assignment](#47-destructuring-assignment) inside match arms. Dict patterns extract values by key.

```opal
# Tuples
match point
  case (0, 0)
    "origin"
  case (x, 0)
    f"on x-axis at {x}"
  case (0, y)
    f"on y-axis at {y}"
  case (x, y)
    f"at ({x}, {y})"
end

# Dicts
match config
  case {host: h, port: p}
    connect(h, p)
  case {host: h}
    connect(h, 8080)
end
```

#### List Patterns

Lists can be matched by exact shape or split into head and tail with `|`.

```opal
match items
  case []
    "empty"
  case [only]
    f"just {only}"
  case [first, second]
    f"{first} and {second}"
  case [head | tail]
    f"{head} and {tail.length} more"
end
```

#### Enum Patterns & Nesting

Enum variant patterns extract associated data. Patterns nest arbitrarily. See [6.9 Enums](#69-enums--algebraic-data-types) for enum definitions.

```opal
# Enum variant matching
match shape
  case Shape.Circle(r)
    Math.PI * r ** 2
  case Shape.Rectangle(w, h)
    w * h
  case Shape.Triangle(b, h)
    0.5 * b * h
end

# Nested patterns — arbitrarily deep
match result
  case Result.Ok(Option.Some(value))
    use(value)
  case Result.Ok(Option.None)
    use_default()
  case Result.Err(e)
    handle(e)
end
```

#### Guards

A `case` arm can include an `if` guard — the arm only matches when both the pattern and the condition are true.

```opal
match value
  case x if x > 100
    "large"
  case x if x > 0
    "positive"
  case x
    "non-positive"
end
```

#### As-Bindings

`as name` binds the entire matched value while still destructuring its contents.

```opal
match shape
  case Shape.Circle(r) as original
    log(original)          # the whole Circle value
    Math.PI * r ** 2
  case _ as s
    log(f"unknown: {s}")
    0.0
end
```

#### Pattern Summary

| Pattern | Example | Matches |
|---|---|---|
| Literal | `case 42`, `case "hello"`, `case null` | Exact value |
| Range | `case 1..10` | Value in range |
| Variable | `case x` | Anything, binds to `x` |
| Wildcard | `case _` | Anything, no binding |
| Type | `case s::String` | Value of type, binds |
| Tuple | `case (x, y)` | Tuple destructure |
| List | `case []`, `case [h \| t]` | List destructure |
| Dict | `case {key: v}` | Dict destructure |
| Enum | `case Shape.Circle(r)` | Variant + extract fields |
| Nested | `case Result.Ok(Option.Some(v))` | Arbitrarily nested |
| Or | `case 1 \| 2 \| 3` | Any of listed (no bindings) |
| Guard | `case x if x > 0` | Pattern + condition |
| As-binding | `case Shape.Circle(r) as s` | Destructure + bind whole |

When matching on an `enum` type, the compiler enforces **exhaustive matching** — all variants must be covered or a `case _` catch-all must be present. See [6.9 Enums](#69-enums--algebraic-data-types) for details.

---

## 6. Functions & Types

### 6.1 Functions & Closures

Functions are defined with `def`. They are first-class values.

```opal
# Basic function
def greet(name)
  print(f"Hello, {name}!")
end

# With type annotations
def add(a::Int32, b::Int32) -> Int32
  a + b
end

# Default arguments
def connect(host, port = 8080)
  # ...
end

# Named arguments at call site
connect(host: "localhost", port: 3000)

# Last expression is the return value (explicit return also works)
def square(x)
  x * x
end
```

#### Closures / Lambdas

Closures use the `|params| body` syntax.

```opal
double = |x| x * 2
apply = |fn, value| fn(value)
apply(double, 5)  # => 10

# Multi-line closure
transform = |items, fn|
  items.map(fn)
end

# Closures capture their enclosing scope
multiplier = 3
triple = |x| x * multiplier
triple(10)  # => 30
```

### 6.2 Type System

Opal uses **gradual typing**: unannotated code is fully dynamic, annotated code is checked at boundaries (function entry, return, annotated assignment). Types serve two equal purposes: catching bugs early and documenting intent.

> See [Type System Design](docs/features/type-system.md) for the full rationale.

#### Core Rules

```opal
# No annotations — fully dynamic
def add(a, b)
  a + b
end

# Annotated — type-checked at boundaries
def add(a::Int32, b::Int32) -> Int32
  x = a + b   # x is NOT checked (internal)
  x            # checked against -> Int32 on return
end

add(1, 2)      # checked: args are Int32
add(1, "hi")   # TYPE ERROR at call site

# Annotated assignment — checked
name::String = "claudio"
age::Int32 = 15

# Explicit casting with `as`
x = 3.14 as Int32   # => 3

# Optional types (nullable)
def find(id::Int32) -> Person?
  # may return null — Person? is sugar for Person | Null
end
```

**Core types:** `Int8`, `Int16`, `Int32`, `Int64`, `Float32`, `Float64`, `Bool`, `Char`, `String`, `Template`, `Symbol`, `Null`, `List(T)`, `Tuple(...)`, `Dict(K, V)`, `Range(T)`, `Regex`.

**Boundary checking rules:**
- Unannotated parameters and variables are dynamic — no checking.
- Annotated parameters are checked at call sites.
- Return type annotations are checked at function exit.
- Annotated variable assignments are checked when assigned.
- Internal variables without annotations are unchecked.
- `as` performs explicit type conversion. Raises a runtime error if conversion fails.
- `?` suffix denotes a nullable type (e.g., `String?` means `String | Null`).

#### Generics

Type parameters are declared explicitly on classes, protocols, and type aliases. At call sites, they're inferred from arguments.

```opal
# Define with explicit type parameter
class Stack(T)
  needs items::List(T)

  def push(item::T)
    .items.append(item)
  end

  def pop() -> T?
    .items.pop()
  end
end

# Inferred at call site
s = Stack.new(items: [1, 2, 3])   # T = Int32
s.push(42)    # ok
s.push("hi")  # type error

# Explicit when ambiguous (e.g., empty collection)
s = Stack(Int32).new(items: [])
```

Generic functions infer type parameters from annotated arguments:

```opal
def first(items::List(T)) -> T?
  items[0]
end

first([1, 2, 3])       # T inferred as Int32, returns Int32?
first(["a", "b"])       # T inferred as String, returns String?
```

#### Generic Constraints

Constraints restrict what types can fill a type parameter. Simple constraints go inline, complex ones use a `where` clause.

```opal
# Inline — single constraint
class SortedList(T implements Comparable)
  needs items::List(T)

  def insert(item::T)
    # compare_to guaranteed available
  end
end

# Where clause — multiple constraints
class Cache(K, V)
    where K implements Hashable,
          V implements Printable
  needs store::Dict(K, V)
end

# Functions — where clause
def max(a::T, b::T) -> T
    where T implements Comparable
  if a > b then a else b end
end

# Functions — inline for simple cases
def sort(items::List(T implements Comparable)) -> List(T)
  # ...
end
```

#### Union Types

A value can be one of several types, expressed with `|`. The nullable `?` suffix is sugar for `T | Null`.

```opal
# Union return type
def parse(input::String) -> Int32 | Float64 | Error
  # can return any of these
end

# Pattern match to narrow
match parse("42")
  case n::Int32
    print(f"integer: {n}")
  case f::Float64
    print(f"float: {f}")
  case e::Error
    print(f"error: {e.message}")
end

# Union in parameters
def display(value::String | Int32 | Float64)
  print(f"{value}")
end
```

Union rules:
- `A | B` is a union — the value is one of the listed types.
- `T?` is exactly `T | Null`.
- Unions are unordered — `Int32 | String` is the same type as `String | Int32`.
- Pattern matching with `case x::Type` narrows a union to a specific type.

#### Type Aliases

The `type` keyword names a complex type. Aliases are transparent — the alias and the original type are fully interchangeable.

```opal
# Simple aliases — semantic names for primitives
type UserID = Int64
type Email = String

# Parameterized aliases
type Result(T) = T | Error
type Pair(A, B) = (A, B)

# Function type alias
type Handler = |Request, Response| -> Null

# Usage
def find_user(id::UserID) -> Result(User)
  # returns User | Error
end
```

Type alias rules:
- `type Name = Type` creates a transparent alias.
- `type Name(T) = ...` creates a parameterized alias.
- Aliases are interchangeable with their underlying type — `UserID` and `Int64` are the same type.
- Aliases can reference other aliases, unions, generics, and function types.

#### Runtime Type Introspection

```opal
# Type of a value
typeof(42)          # => Int32
typeof("hello")     # => String
typeof([1, 2, 3])   # => List(Int32)

# Type narrowing with `is`
if value is String
  # value is known to be String here
  print(value.length)
end

# `is` with unions
def handle(result::Int32 | String | Error)
  if result is Error
    print(f"failed: {result.message}")
  else
    print(f"ok: {result}")
  end
end

# `is` with protocols
if shape is Drawable
  shape.draw()
end
```

Introspection rules:
- `typeof(expr)` returns the runtime type as a Type object.
- `is` checks if a value is an instance of a type, protocol, or union member.
- `is` narrows the type in the enclosing branch (flow-sensitive narrowing).

### 6.3 Classes & Methods

> See [Classes & Inheritance](docs/features/classes-and-inheritance.md) for the full design rationale.

Classes use `def :init()` for construction. Instance variables are accessed with the `.` prefix.

```opal
class Person
  def :init(name = "anonymous", age = 0)
    .name = name
    .age = age
    .started = true
  end

  def greet()
    print(f"Hi, my name is {.name}")
  end

  # Names ending in ? are for predicates
  def adult?()
    .age >= 18
  end

  # Names ending in ! are for mutations
  def rename!(new_name)
    .name = new_name
  end

  # Static method (defined with self.)
  def self.species()
    "Homo sapiens"
  end
end

# Object creation with .new() and named arguments
claudio = Person.new(name: "claudio", age: 15)
claudio.greet()         # => "Hi, my name is claudio"
claudio.adult?()        # => false
Person.species()        # => "Homo sapiens"
```

#### Inheritance

```opal
class Animal
  def talk()
    print("...")
  end
end

class Dog < Animal
  def talk()
    print("Woof!")
  end
end

rex = Dog.new()
rex.talk()  # => "Woof!"
```

#### Construction Order

When both `needs` and `:init` are present, they compose in a defined order:

```opal
class OrderService
  needs db::Database
  needs mailer::Mailer

  def :init(retry_count = 3)
    # .db and .mailer already available
    .retry_count = retry_count
    .cache = {:}
  end
end

service = OrderService.new(
  db: PostgresDB.new(),
  mailer: SMTPMailer.new(),
  retry_count: 5
)
```

Construction order:
1. Parent `needs` fields injected (if inheriting)
2. Own `needs` fields injected
3. Parent `:init` runs (if present)
4. Own `:init` runs (if present)

#### Inherited `needs`

Subclasses inherit all `needs` from their parent. Callers must provide every ancestor's `needs` at construction time:

```opal
class Animal
  needs name::String
  needs sound::String

  def speak()
    print(f"{.name} says {.sound}")
  end
end

class Dog < Animal
  needs breed::String
end

# Must provide ALL needs (parent + own)
rex = Dog.new(name: "Rex", sound: "Woof", breed: "Labrador")
rex.name    # => "Rex" (inherited)
rex.breed   # => "Labrador" (own)
```

#### `super`

`super` calls the parent's version of the current method:

```opal
class Dog < Animal
  def speak()
    super()  # calls Animal.speak
    print(f"({.breed})")
  end
end
```

In `:init`, `super()` calls the parent's `:init`.

#### Class Rules

- Single inheritance only: `class Child < Parent`. No multiple inheritance.
- No abstract classes — use protocols for contracts (see 6.6).
- `needs` fields are inherited — subclass `.new()` requires all ancestor `needs`.
- Construction order: parent needs -> own needs -> parent `:init` -> own `:init`.
- `super` calls parent's method. `super()` in `:init` calls parent's `:init`.
- For multiple behaviors, combine `< Parent` with `implements Protocol`.

### 6.4 Modules & Namespaces

> See [Imports & Modules](docs/features/imports-and-modules.md) for the full design rationale.

Modules group related functions, classes, and constants.

#### File-to-Module Mapping

Each `.opl` file implicitly defines a module matching its filename in PascalCase. Directories create module hierarchies:

```
src/
  app.opl           -> App
  math.opl          -> Math
  math/
    vector.opl      -> Math.Vector
    matrix.opl      -> Math.Matrix
```

```opal
# file: src/math.opl
# Implicitly the Math module

PI = 3.14159

def abs(x::Number)
  if x < 0 then -x else x end
end

def max(a, b)
  if a > b then a else b end
end

module Trig           # Math.Trig (nested)
  def sin(x::Float64) -> Float64
    # ...
  end
end

Math.abs(-5)    # => 5
Math.PI         # => 3.14159
Math.Trig.sin(1.0)
```

Modules can contain classes:

```opal
# file: src/geometry.opl
# Implicitly the Geometry module

class Circle
  def :init(radius::Float32)
    .radius = radius
  end

  def area()
    Math.PI * .radius ** 2
  end
end

c = Geometry.Circle.new(radius: 5.0)
c.area()  # => 78.539...
```

#### Import Syntax

```opal
import Math                          # whole module — Math.abs(), Math.PI
import Math.Vector                   # nested — Math.Vector.dot()
import Math.{abs, max}               # selective — abs() and max() directly
import Math.Vector as Vec            # aliased — Vec.dot()
import Math.{abs, max as maximum}    # selective + alias
```

Collision rule — two selective imports with the same name are a compile-time error:

```opal
import Math.{max}
import Stats.{max}       # COMPILE ERROR — 'max' already imported

import Math.{max as math_max}
import Stats.{max as stats_max}  # ok
```

#### Re-exports

```opal
# file: src/opal_web.opl
import OpalWeb.Router
import OpalWeb.Middleware

export Router.{get, post, put, delete}
export Middleware.{use}
```

```opal
# Consumer
import OpalWeb
OpalWeb.get("/", handler)   # works — re-exported from Router
```

#### Module Rules

- Each `.opl` file implicitly defines a module matching its filename (PascalCase).
- `module` blocks inside a file create nested sub-modules.
- Subdirectories create module hierarchies. Both approaches (file-level and directory-level) work.
- All imports are absolute from the project root — no relative imports.
- Circular imports are a compile-time error. Extract shared code to break cycles.
- `export Module.{names}` re-exports symbols for clean public APIs.
- All top-level definitions are public by default. Use `private` to hide.

#### Packages

A package is a directory with `opal.toml` + `src/`. External packages are imported by name:

```opal
# Import from an external package (installed via opal pkg)
import OpalWeb
import OpalWeb.Router

# Within your own package — absolute from package root
import MyApp.Models.User
```

### 6.5 Visibility / Access Control

```opal
class Account
  def :init(owner, balance)
    .owner = owner
    .balance = balance
  end

  public def balance()
    .balance
  end

  public def deposit(amount)
    .balance += amount
  end

  private def calculate_interest()
    .balance * 0.05
  end

  protected def transfer_to(other::Account, amount)
    .balance -= amount
    other.deposit(amount)
  end
end

acct = Account.new(owner: "alice", balance: 1000)
acct.balance()              # => 1000
acct.calculate_interest()   # Error: private method called
```

Default visibility is `public`. Mark methods `private` (accessible only within the class) or `protected` (accessible within the class and subclasses).

### 6.6 Interfaces / Protocols

> See [Self-Hosting Foundations](docs/features/self-hosting-foundations.md) for the protocol defaults design rationale.

Protocols define a contract that classes must fulfill. Methods without a body are **required** — implementors must define them. Methods with a body are **defaults** — inherited automatically, overridable.

```opal
protocol Printable
  # Required
  def to_string() -> String

  # Defaults — derived from to_string
  def print()
    IO.print(.to_string())
  end

  def println()
    IO.println(.to_string())
  end

  def inspect() -> String
    f"<{typeof(self).name}: {.to_string()}>"
  end
end

protocol Comparable
  # Required
  def compare_to(other) -> Int32

  # Defaults — derived from compare_to
  def <(other) -> Bool
    .compare_to(other) < 0
  end

  def >(other) -> Bool
    .compare_to(other) > 0
  end

  def <=(other) -> Bool
    .compare_to(other) <= 0
  end

  def >=(other) -> Bool
    .compare_to(other) >= 0
  end
end

class Person implements Printable
  def :init(name, age)
    .name = name
    .age = age
  end

  def to_string()
    f"{.name}, age {.age}"
  end

  # Override a default
  def inspect()
    f"<Person name={.name} age={.age}>"
  end
end

person = Person.new(name: "claudio", age: 15)
person.println()   # "claudio, age 15" (default, calls to_string)
person.inspect()   # "<Person name=claudio age=15>" (overridden)
```

```opal
# Multiple protocols — implementor gets all defaults
protocol Hashable
  # Required
  def hash_code() -> Int32

  # Default
  def ==(other) -> Bool
    .hash_code() == other.hash_code()
  end
end

class Temperature implements Printable, Comparable, Hashable
  def :init(degrees::Float32)
    .degrees = degrees
  end

  def to_string()
    f"{.degrees}°"
  end

  def compare_to(other::Temperature) -> Int32
    (.degrees - other.degrees) as Int32
  end

  def hash_code() -> Int32
    .degrees as Int32
  end
end

a = Temperature.new(degrees: 20.0)
b = Temperature.new(degrees: 30.0)
a < b     # => true (default from Comparable)
a.println()  # "20.0°" (default from Printable)
```

If two protocols provide conflicting defaults for the same method name, the implementor must explicitly define it (ambiguity = compile-time error).

Opal uses **nominal typing** — a class must declare `implements Protocol` to satisfy it. Having the right methods is not enough:

```opal
protocol Drawable
  def draw() -> String
end

class Circle implements Drawable
  def draw() -> String
    f"circle at ({.x}, {.y})"
  end
end

class Coin
  def draw() -> String  # same shape, but NOT Drawable
    "coin"
  end
end

def render(shape::Drawable)
  shape.draw()
end

render(Circle.new(x: 0, y: 0))  # ok
render(Coin.new())               # TYPE ERROR — Coin doesn't implement Drawable
```

**Retroactive conformance** lets you add protocol conformance to types you don't own:

```opal
implements Drawable for ThirdPartyShape
  def draw() -> String
    .render()  # delegate to existing method
  end
end

render(ThirdPartyShape.new())  # now works
```

Retroactive conformance rules:
- `implements Protocol for Type` adds conformance after the fact.
- Can define new methods or delegate to existing ones.
- Cannot access private fields of the target type.
- If two retroactive conformances conflict, the one in the current module wins.

**Generic protocols** use type parameters like classes:

```opal
protocol Collection(T)
  def add(item::T)
  def contains?(item::T) -> Bool
  def size() -> Int32
end

class Set(T implements Hashable) implements Collection(T)
  def add(item::T)
    # ...
  end

  def contains?(item::T) -> Bool
    # ...
  end

  def size() -> Int32
    .items.length
  end
end
```

### 6.7 Multiple Dispatch

Functions can have multiple definitions that dispatch based on argument types, arity, and guards.

```opal
class Renderer
  # Dispatch by type
  def render(shape::Circle)
    draw_circle(shape.center, shape.radius)
  end

  def render(shape::Rectangle)
    draw_rect(shape.origin, shape.width, shape.height)
  end

  # Dispatch by arity
  def render(shape::Circle, color::Color)
    set_color(color)
    draw_circle(shape.center, shape.radius)
  end
end
```

**Resolution order:**

1. **Exact type match** — argument types match a definition exactly.
2. **Guard-constrained match** — a guard narrows the valid inputs.
3. **Signature arity match** — number of arguments selects among overloads.
4. **Ambiguity = compile-time error** — if two definitions match equally well, the compiler rejects the program.

```opal
# Dispatch with guards
def process(value::Int32)
  print("generic integer")
end

@positive
def process(value::Int32)
  print("positive integer")
end

process(5)   # => "positive integer" (guard match wins)
process(-3)  # => "generic integer"  (guard fails, falls to base)
```

### 6.8 Iterator Protocol

> See [Self-Hosting Foundations](docs/features/self-hosting-foundations.md) for the iterator protocol design rationale.

Two protocols — `Iterable` (the thing you iterate over) and `Iterator` (the cursor). Any class implementing `Iterable` works with `for ... in` and collection methods like `map`, `filter`, `reduce`.

```opal
# Built-in protocols
protocol Iterable
  def iter() -> Iterator
end

protocol Iterator
  def next() -> (value, done::Bool)
end
```

```opal
# Custom collection: iterate lines of a file
class FileLines implements Iterable
  needs path::String

  def iter()
    FileLinesIterator.new(file: File.open(.path))
  end
end

class FileLinesIterator implements Iterator
  needs file::File

  def next()
    line = .file.read_line()
    if line == null
      (null, true)    # done
    else
      (line, false)   # value
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

```opal
# Lazy infinite sequence
class Counter implements Iterable
  needs start::Int32

  def iter()
    CounterIterator.new(current: .start)
  end
end

class CounterIterator implements Iterator
  needs current::Int32

  def next()
    value = .current
    .current += 1
    (value, false)  # never done
  end
end

for n in Counter.new(start: 0).take(5)
  print(n)  # 0, 1, 2, 3, 4
end
```

**Rules:**
- `Iterator.next()` returns a tuple `(value, done::Bool)`.
- Built-in types (`List`, `Dict`, `Range`, `String`) all implement `Iterable`.
- Collection methods (`map`, `filter`, `reduce`, `take`, `zip`) work on any `Iterable`.

### 6.9 Enums & Algebraic Data Types

> See [Enums & Algebraic Data Types](docs/features/enums-and-algebraic-types.md) for the full design rationale.

The `enum` keyword defines a closed set of variants. Variants without fields are simple constants. Variants with fields carry data. One keyword covers both simple enums and full algebraic data types.

```opal
# Simple enum — no data
enum Direction
  North
  South
  East
  West
end

# Data-carrying variants
enum Shape
  Circle(radius::Float64)
  Rectangle(width::Float64, height::Float64)
  Triangle(base::Float64, height::Float64)
end

# Mixed
enum Response
  Success(body::String, headers::Dict(String, String))
  NotFound(path::String)
  ServerError(reason::String, code::Int32)
  Unauthorized
end

# Construction
d = Direction.North
s = Shape.Circle(radius: 5.0)
r = Response.Success(body: "hello", headers: {:})
```

#### Exhaustive Pattern Matching

Matching on an enum must cover all variants or include a `case _` catch-all. Missing a variant is a compile-time error.

```opal
match shape
  case Shape.Circle(r)
    Math.pi * r ** 2
  case Shape.Rectangle(w, h)
    w * h
  case Shape.Triangle(b, h)
    0.5 * b * h
end

# Guards work with enum variants
match response
  case Response.ServerError(reason, code) if code >= 500
    retry()
  case Response.ServerError(reason, code)
    log(f"client error {code}: {reason}")
  case Response.Success(body, _)
    process(body)
  case _
    handle_other()
end
```

#### Methods & Protocols

Enums can have methods and implement protocols. Methods operate on `self` and typically match across variants.

```opal
enum Shape implements Printable
  Circle(radius::Float64)
  Rectangle(width::Float64, height::Float64)
  Triangle(base::Float64, height::Float64)

  def area() -> Float64
    match self
      case Shape.Circle(r)
        Math.pi * r ** 2
      case Shape.Rectangle(w, h)
        w * h
      case Shape.Triangle(b, h)
        0.5 * b * h
    end
  end

  def to_string() -> String
    match self
      case Shape.Circle(r)
        f"Circle(r={r})"
      case Shape.Rectangle(w, h)
        f"Rect({w}x{h})"
      case Shape.Triangle(b, h)
        f"Tri(b={b}, h={h})"
    end
  end
end

s = Shape.Circle(radius: 5.0)
s.area()       # => 78.539...
s.println()    # "Circle(r=5.0)" — default from Printable
```

#### Generic Enums

Enums support type parameters, enabling foundational stdlib types like `Option(T)` and `Result(T, E)`.

```opal
enum Option(T)
  Some(value::T)
  None
end

enum Result(T, E)
  Ok(value::T)
  Err(error::E)
end

# Type inferred from construction
opt = Option.Some(value: 42)       # Option(Int32)
opt = Option(String).None           # explicit when ambiguous

# Result in practice
def parse_int(s::String) -> Result(Int32, String)
  # ...
end

match parse_int("42")
  case Result.Ok(n)
    print(f"parsed: {n}")
  case Result.Err(msg)
    print(f"failed: {msg}")
end
```

**Relationship to `?` nullable:** `T?` stays as `T | Null` — lightweight nullable for everyday use. `Option(T)` is a stdlib enum for explicit `Some`/`None` with exhaustive matching. They are separate types.

**Enum rules:**
- Variants are accessed as `EnumName.VariantName`.
- Variants with fields use named arguments: `Shape.Circle(radius: 5.0)`.
- Variants without fields are singletons.
- Enums are closed and immutable — no variants can be added, values cannot be modified.
- Methods are defined inside the `enum` block, after the variants.
- Enums can `implements` protocols. Retroactive conformance works on enums.
- Enums support type parameters with constraints, same as classes.
- `match` on an enum without covering all variants = compile-time error (unless `case _` is present).

### 6.10 Models (Validated Data)

> See [Validation & Settings](docs/features/validation-and-settings.md) for the full design rationale.

The `model` keyword defines validated, immutable data structures — Opal's equivalent of Pydantic's BaseModel. Models validate on construction, serialize to/from dicts and JSON, and differ from `class` in semantics: data with constraints vs behavior with state.

```opal
model User
  needs name::String where |v| v.length > 0
  needs email::String where is_email
  needs age::Int32 where |v| v >= 0
  needs role::String = "member"
end

# Construction — validates all fields
user = User.new(name: "claudio", email: "c@test.com", age: 15)

# Immutable
user.name         # => "claudio"
user.name = "x"   # COMPILE ERROR — models are immutable

# Modified copy (also validated)
updated = user.copy(age: 16)
```

#### Field Validation

Three forms of `where` — inline closures, named guards, and guards with partial application. Comma-separated to combine.

```opal
# Reusable guards
guard is_email(value) fails :invalid_email
  return /^[^@]+@[^@]+\.[^@]+$/.match?(value)
end

guard min_length(value, n) fails :too_short
  return value.length >= n
end

model Account
  needs email::String where is_email                     # named guard
  needs age::Int32 where |v| v >= 0                      # inline closure
  needs username::String where min_length(3)             # partial application
  needs password::String where is_required, |v| v.length >= 8  # multiple
  needs discount::Float64 = 0.0
  needs price::Float64

  # Cross-field validation
  validate do
    if .discount > .price
      fail ValidationError.new(field: "discount", reason: "cannot exceed price")
    end
  end
end
```

**Validation order:** type checking -> inline `where` per field -> `validate` blocks.

#### Serialization

Models automatically get `to_dict`/`from_dict` and `to_json`/`from_json`. Nested models serialize recursively. Deserialization validates on load.

```opal
model Address
  needs street::String
  needs city::String
  needs zip::String where |v| /^\d{5}$/.match?(v)
end

model User
  needs name::String where |v| v.length > 0
  needs email::String where is_email
  needs address::Address
end

user = User.new(
  name: "claudio", email: "c@test.com",
  address: Address.new(street: "123 Main", city: "Springfield", zip: "62704")
)

user.to_dict()   # => {"name": "claudio", "email": "c@test.com",
                 #     "address": {"street": "123 Main", ...}}
user.to_json()   # => '{"name": "claudio", ...}'

# Deserialize — validates on load
user = User.from_dict({"name": "claudio", "email": "c@test.com",
                        "address": {"street": "123 Main", "city": "Springfield", "zip": "62704"}})
```

**Model rules:**
- All fields declared with `needs` — same syntax as classes.
- `where |v| expr` inline closure, `where guard_name`, or `where guard_name(args)` for partial application.
- Comma-separated `where` constraints: all must pass.
- `validate do ... end` for cross-field validation, runs after all fields pass.
- Validation runs on `.new()`, `.from_dict()`, `.from_json()`, and `.copy()`.
- Failed validation raises `ValidationError`.
- `to_dict()` / `to_json()` serialize recursively. `from_dict()` / `from_json()` deserialize and validate.
- Models can have methods but cannot mutate fields.

---

## 7. Error Handling & Safety

### 7.1 Error Handling

> See [Error Handling Design](docs/features/error-handling.md) for the full design rationale.
> See [Self-Hosting Foundations](docs/features/self-hosting-foundations.md) for the custom error types design.

Opal has two error handling mechanisms for different situations:

- **Exceptions** (`fail` / `try` / `on fail`) — for truly exceptional, unrecoverable, or unexpected errors. Propagate implicitly up the call stack.
- **Result types** (`Result(T, E)`) — for expected, recoverable errors. Explicit return values that force the caller to handle both cases.

**When to use which:** If the caller should *always* handle it, use `Result`. If the caller *shouldn't need to know* how to handle it, use exceptions.

#### Exceptions

Errors are classes inheriting from `Error`. `fail` raises, `try`/`on fail` catches, `ensure` always runs.

```opal
# Custom error types — classes with needs fields
class FileNotFound < Error
  needs path::String

  def :init(path)
    .path = path
    super(message: f"File not found: {path}")
  end
end

# Error hierarchies — on fail catches subclasses
class AppError < Error end
class AuthError < AppError end
class PermissionDenied < AuthError end
class TokenExpired < AuthError end
```

```opal
def read_config(path::String) -> Dict
  if not File.exists?(path)
    fail FileNotFound.new(path: path)
  end
  JSON.parse(File.read(path))
end

try
  config = read_config("missing.json")
on fail FileNotFound as e
  print(f"Missing: {e.path}")
on fail AuthError as e
  # Catches both PermissionDenied and TokenExpired
  print(f"Auth failed: {e.message}")
on fail as e
  log(f"Unexpected: {e.message}")
  fail(e)  # re-raise
ensure
  cleanup()
end
```

#### Result Types & the `!` Operator

`Result(T, E)` is an enum (see [6.9](#69-enums--algebraic-data-types)) for expected errors. The `!` postfix operator unwraps `Ok` or propagates `Err` from the enclosing function.

```opal
# Without ! — verbose nested matching
def process(path::String) -> Result(Config, Error)
  match read_file(path)
    case Result.Ok(content)
      match parse_json(content)
        case Result.Ok(config)
          Result.Ok(config)
        case Result.Err(e)
          Result.Err(e)
      end
    case Result.Err(e)
      Result.Err(e)
  end
end

# With ! — linear and clean
def process(path::String) -> Result(Config, Error)
  content = read_file(path)!
  config = parse_json(content)!
  Result.Ok(config)
end
```

**Helper methods on Result:**

```opal
result.ok?                    # => true if Ok
result.err?                   # => true if Err
result.unwrap()               # => value if Ok, raises exception if Err
result.unwrap("msg")          # => value if Ok, raises with custom message if Err
result.unwrap_or(default)     # => value if Ok, default if Err
result.map(|v| v + 1)        # => Ok(v + 1) if Ok, passes Err through
result.map_err(|e| wrap(e))  # => passes Ok through, transforms Err
```

#### Bridging Exceptions and Results

```opal
# Exception -> Result: wrap a throwing block
result = Result.from do
  read_config("missing.json")
end
# => Result.Err(FileNotFound(...)) if it threw
# => Result.Ok(config) if it succeeded

# Catch only a specific error type
result = Result.from(FileNotFound) do
  read_config("missing.json")
end

# Result -> Exception: .unwrap() raises on Err
config = parse_config(data).unwrap()

# Mixing both worlds
def start_app() -> Result(App, Error)
  data = Result.from do
    File.read("config.json")
  end!  # propagate if Err
  config = parse_config(data)!
  Result.Ok(App.new(config: config))
end
```

**Error handling rules:**
- `fail expr` raises any `Error` subclass.
- `on fail Type as e` catches that type and subclasses. `on fail as e` catches all.
- `ensure` always runs.
- `expr!` on a `Result` unwraps `Ok` or returns `Err` from the enclosing function.
- The enclosing function must return `Result` — using `!` elsewhere is a compile-time error.
- `.unwrap()` raises an exception on `Err`. `!` returns `Err` as a value.
- `Result.from do ... end` catches exceptions into `Result.Err`.
- `Result.from(Type) do ... end` catches only that type.

### 7.2 Guards & Rules

Guards validate data before a function body executes. Guard functions are reusable — the same guard works as a function pre-condition (`@guard`) and as a model field constraint (`where guard`).

```opal
# Define reusable guards
guard positive(value) fails :must_be_positive
  return value > 0
end

guard old_enough(age) fails :too_young
  return age >= 18
end

# As function pre-condition (decorator) — guard receives the function's arguments
@positive
def sqrt(value::Float64) -> Float64
  value ** 0.5
end

@old_enough
def register_voter(name::String, age::Int32)
  print(f"{name} registered to vote")
end

# Same guards work in model field validation (where clause)
model Registration
  needs name::String where |v| v.length > 0
  needs age::Int32 where old_enough
  needs deposit::Float64 where positive
end
```

**Rules:**
- `guard name(params) fails :symbol ... end` defines a reusable guard function.
- `@guard_name` before a function definition = pre-condition. The guard receives the function's arguments. If it fails, raises a `GuardError` with the `:symbol`.
- `where guard_name` on a model field = field validation. The guard receives the field's value. See [6.10 Models](#610-models-validated-data).
- For type constraints on parameters, use type annotations: `param::Type` (see [6.2](#62-type-system)).
- `@` is used for both guard decorators and macro invocations. Guards are resolved first; if no guard matches, it's treated as a macro.

### 7.3 Null Objects

Null objects provide default behavior instead of null checks.

```opal
class Person
  needs name::String
  needs age::Int32

  def greet()
    print(f"Hi, I'm {.name}")
  end
end

# Full form — subclass with overridden behavior
class NullPerson < Person
  def :init()
    super(name: "anonymous", age: 0)
  end

  def greet()
    print("Hi, I don't want to say my name")
  end
end

# Shortcut — auto-generates a subclass with default values
class AnonymousPerson as Person defaults {name: "anonymous", age: 0}
# Equivalent to a subclass whose :init calls super with these defaults.
# All methods delegate to Person — only construction differs.
```

```opal
# Usage
def find_person(id)
  result = database.find(id)
  if result == null
    NullPerson.new()
  else
    result
  end
end

person = find_person(999)
person.greet()  # no null check needed — NullPerson handles it
```

---

## 8. Concurrency

> See [Concurrency Design](docs/features/concurrency.md) for the full design rationale.

Opal's concurrency model has four layers: **actors** for stateful concurrent entities, **parallel blocks** for structured concurrency, **async/futures** for individual non-blocking calls, and **supervisors** for fault tolerance.

**Core principles:**
- **Sync by default** — all calls block and return values. Async is opt-in.
- **No colored functions** — there is no `async def`. Any expression can be made async at the call site.
- **Structured concurrency** — concurrent work has a parent scope. No orphaned tasks.

### 8.1 Actors

Actors are long-lived concurrent entities with isolated state. All external interaction goes through message passing via `receive` blocks and `.send()`. Methods defined with `def` are internal only.

```opal
actor Counter
  def :init()
    .count = 0
  end

  receive :increment
    .count += 1
    reply .count
  end

  receive :get_count
    reply .count
  end

  receive :reset
    .count = 0
    reply :ok
  end

  # Internal helper — not accessible from outside
  private def validate_count()
    .count >= 0
  end
end

# All interaction through .send() — sync by default
c = Counter.new()
c.send(:increment)     # => 1 (blocks until reply)
c.send(:increment)     # => 2
c.send(:get_count)     # => 2
c.send(:reset)         # => :ok
```

```opal
# Messages with arguments
actor Cache
  def :init(ttl::Int32)
    .store = {:}
    .ttl = ttl
  end

  receive :get(key)
    reply .store[key]
  end

  receive :set(key, value)
    .store[key] = value
    reply :ok
  end
end

cache = Cache.new(ttl: 60)
cache.send(:set, "user:1", "claudio")
cache.send(:get, "user:1")  # => "claudio"
```

### 8.2 Structured Concurrency (`parallel`)

The `parallel` block runs expressions concurrently and waits for all to complete.

```opal
# Fan-out: run expressions concurrently, collect all results
users, orders, inventory = parallel
  fetch_users()
  fetch_orders()
  fetch_inventory()
end
# Blocks until ALL complete.
# Results returned as a tuple, matching the order of expressions.
# If any expression fails, the others are cancelled.
```

```opal
# Parallel iteration
pages = parallel for url in urls
  Net.fetch(url)
end
# Returns a list of responses, fetched concurrently

# With a concurrency limit
pages = parallel max: 5 for url in urls
  Net.fetch(url)
end
# At most 5 fetches run at a time
```

**Cancellation rule:** if any branch in a `parallel` block fails, all sibling branches are cancelled and the failure propagates to the caller.

```opal
try
  a, b = parallel
    fetch_a()   # succeeds
    fetch_b()   # fails!
  end
on fail as e
  # fetch_a() is cancelled, error from fetch_b() is raised here
  print(f"Failed: {e.message}")
end
```

### 8.3 Async / Futures

For when `parallel` is too rigid and you need fine-grained control.

```opal
# async turns any expression into a Future
user_future = async fetch_user(id)

# Do other work while it runs...
prepare_template()

# Auto-await: using the future's value blocks until ready
print(f"Hello, {user_future.name}")  # blocks here if not yet done

# Explicit await (when you want to be clear about the blocking point)
user = await user_future

# Check readiness without blocking
if user_future.ready?()
  print("done!")
end
```

```opal
# Async with actors
count_future = async counter.send(:get_count)
# ... do other work ...
count = await count_future

# Error handling — failures surface when you await
future = async risky_operation()
try
  result = await future
on fail as e
  print(f"Operation failed: {e.message}")
end
```

**Rules:**
- `async expr` returns a `Future(T)` — the expression runs concurrently.
- **Auto-await on use:** accessing a Future's value blocks until ready.
- `await` is available for explicit blocking points.
- `.ready?()` checks completion without blocking.
- Failures are captured in the Future and re-raised on await.

### 8.4 Supervisors

Supervisors watch child actors and restart them on failure.

```opal
supervisor AppSupervisor
  strategy :one_for_one       # only restart the failed child
  max_restarts 3 within 60    # give up after 3 crashes in 60 seconds

  supervise Logger.new()
  supervise Cache.new(ttl: 60)
  supervise Worker.new()
end

app = AppSupervisor.start!
```

**Strategies:**

| Strategy | Behavior |
|---|---|
| `:one_for_one` | Restart only the crashed child. |
| `:all_for_one` | Restart all children if one crashes. |
| `:rest_for_one` | Restart the crashed child and all started after it. |

```opal
# Supervisor trees — supervisors can supervise other supervisors
supervisor RootSupervisor
  strategy :one_for_one

  supervise AppSupervisor
  supervise MetricsSupervisor
end
```

**Actor lifecycle hooks:**

```opal
actor Worker
  def :init()
    .jobs = []
  end

  receive :do(job)
    .jobs.push(job)
    process(job)
    reply :ok
  end

  # Called before the actor stops (crash or shutdown)
  def on_crash(reason)
    log(f"Worker crashed: {reason}. Had {.jobs.length} pending jobs.")
  end

  # Called after a restart
  def on_restart()
    log("Worker restarted")
  end
end
```

### 8.5 Complete Example

```opal
import Net
import JSON

actor RateLimiter
  def :init(max_per_second)
    .max = max_per_second
    .count = 0
  end

  receive :check
    if .count < .max
      .count += 1
      reply :ok
    else
      reply :limited
    end
  end

  receive :reset
    .count = 0
    reply :ok
  end
end

def fetch_dashboard(user_id)
  limiter = RateLimiter.new(max_per_second: 10)

  # Actor message (sync by default)
  status = limiter.send(:check)
  if status == :limited
    fail RateLimitError.new("Too many requests")
  end

  # Structured concurrency
  profile, notifications, feed = parallel
    fetch_profile(user_id)
    fetch_notifications(user_id)
    fetch_feed(user_id)
  end

  # Async for background work (don't need result now)
  async log_access(user_id)

  {profile: profile, notifications: notifications, feed: feed}
end

# Supervision for production
supervisor DashboardSupervisor
  strategy :one_for_one
  max_restarts 5 within 30

  supervise RateLimiter.new(max_per_second: 100)
end
```

**Concurrency summary:**

| Need | Tool | Syntax |
|---|---|---|
| Stateful concurrent entity | Actor | `actor`, `receive`, `.send()` |
| Run N things concurrently, wait for all | Parallel block | `parallel ... end` |
| Run N items concurrently | Parallel for | `parallel for x in xs ... end` |
| Limit concurrency | Parallel max | `parallel max: N for ...` |
| Make one call non-blocking | Async/Future | `async expr`, auto-await on use |
| Fault tolerance | Supervisor | `supervisor`, `strategy`, `supervise` |
| Crash recovery hooks | Lifecycle | `on_crash(reason)`, `on_restart()` |

---

## 9. Software Engineering Patterns

### 9.1 Dependency Injection (`needs`)

> See [Dependency Injection & Events Design](docs/features/dependency-injection-and-events.md) for the full design rationale.

`needs` declares a dependency with a name and a protocol/type. Dependencies become instance variables (`.name`) and must be provided at construction time via `.new()`.

```opal
protocol Database
  def save(record) -> Bool
  def find(id::Int32) -> Record?
end

protocol Mailer
  def send_confirmation(order::Order)
end

class OrderService
  needs db::Database
  needs mailer::Mailer

  def place_order(order)
    .db.save(order)
    .mailer.send_confirmation(order)
  end
end

# Explicit wiring — you see exactly what connects to what
service = OrderService.new(
  db: PostgresDB.new(),
  mailer: SMTPMailer.new()
)

# Testing — swap implementations
test_service = OrderService.new(
  db: MockDB.new(),
  mailer: MockMailer.new()
)
```

`needs` works on classes, modules, and actors:

```opal
# On a module
module Billing
  needs payments::PaymentGateway

  def charge(order)
    .payments.charge(order.total)
  end
end

# On an actor
actor PaymentProcessor
  needs gateway::PaymentGateway

  receive :charge(order)
    .gateway.charge(order.total)
    reply :ok
  end
end
```

**Rules:**
- `needs name::Protocol` declares a required dependency.
- `needs name::Protocol = default_expr` declares an optional dependency with a default.
- Dependencies are checked at construction — missing a required `needs` is a runtime error.
- `needs` dependencies are accessible as `.name` (same as instance variables).
- If the class also has `:init`, `needs` deps are injected *before* `:init` runs.

#### Optional Container (for large apps)

For small apps, manual wiring with `.new()` is sufficient. For large apps, the `Container` class from the standard library resolves dependencies by protocol.

```opal
import Container

app = Container.new()
app.register(Database, PostgresDB.new())
app.register(Mailer, SMTPMailer.new())

# Resolve — container fills in all `needs` automatically
service = app.resolve(OrderService)
# Equivalent to: OrderService.new(db: postgres, mailer: smtp)

# Resolve modules — handlers are auto-registered with deps
app.resolve(NotificationHandler)
app.resolve(InventoryHandler)

app.start!
```

```opal
# Testing with container — swap just what you need
test_app = Container.new()
test_app.register(Database, MockDB.new())
test_app.register(Mailer, MockMailer.new())

test_service = test_app.resolve(OrderService)
```

`Container` is a standard library class, not a language keyword — the language stays small.

### 9.2 Domain Events (`event`, `emit`, `on`)

Events are declared as named, immutable data structures. They're emitted with `emit` and handled with `on`. Under the hood, events are dispatched through an actor-based event bus — handlers get supervision and fault tolerance for free.

```opal
# Declare events — they're just immutable data
event OrderPlaced(order::Order, placed_at::Time)
event OrderShipped(order::Order, tracking::String)
event PaymentFailed(order::Order, reason::String)

# Emit from anywhere
class OrderService
  needs db::Database

  def place_order(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end

# Handle in modules — deps available via needs
module NotificationHandler
  needs mailer::Mailer

  on OrderPlaced do |e|
    .mailer.send_confirmation(e.order)
  end

  on OrderShipped do |e|
    .mailer.send_tracking(e.order, e.tracking)
  end

  on PaymentFailed do |e|
    .mailer.send_payment_alert(e.order, e.reason)
  end
end

module InventoryHandler
  needs warehouse::WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end
```

Events compose with existing features:

```opal
# With pattern matching
module AnalyticsHandler
  needs tracker::Analytics

  on OrderPlaced do |e|
    match e.order.total
      case amount if amount > 1000
        .tracker.flag_high_value(e.order)
      case _
        .tracker.record(e.order)
    end
  end
end

# With guards
@only_business_hours
on OrderPlaced do |e|
  notify_sales_team(e.order)
end
```

**Rules:**
- `event Name(fields...)` declares an event type (immutable data).
- `emit event_instance` dispatches the event to all registered `on` handlers.
- `on EventType do |e| ... end` registers a handler.
- Handlers run **asynchronously** by default (fire-and-forget from the emitter).
- Multiple handlers for the same event run **concurrently** (via actors underneath).
- Handlers in modules have access to the module's `needs` dependencies.

#### Emit and Async Interaction

`emit` is async by default because events represent something that already happened. Use `emit ... await` when you need all handlers to finish first.

```opal
# Async (default) — returns immediately
emit OrderPlaced.new(order: order)

# Sync — blocks until all handlers complete
emit OrderPlaced.new(order: order) await

# Background sync — returns a Future
delivery = async emit OrderPlaced.new(order: order) await
do_other_work()
await delivery  # check if handlers succeeded
```

| Pattern | Behavior |
|---|---|
| `emit Event.new(...)` | Async — fire and forget, returns immediately |
| `emit Event.new(...) await` | Sync — blocks until all handlers complete |
| `async emit Event.new(...) await` | Background sync — all handlers run, returns Future |
| `emit` inside `parallel` | Each branch emits independently |
| `emit` inside actor `receive` | Works normally, handlers run outside the actor |

#### Complete DDD Example

```opal
import Container
import Time

# --- Domain Events ---
event OrderPlaced(order::Order, placed_at::Time)
event PaymentFailed(order::Order, reason::String)

# --- Domain Service (with DI) ---
class OrderService
  needs db::Database
  needs validator::OrderValidator

  def place_order(order)
    .validator.validate!(order)
    .db.save(order)
    emit OrderPlaced.new(order: order, placed_at: Time.now())
  end
end

# --- Event Handlers (with DI) ---
module NotificationHandler
  needs mailer::Mailer

  on OrderPlaced do |e|
    .mailer.send_confirmation(e.order)
  end

  on PaymentFailed do |e|
    .mailer.send_payment_alert(e.order, e.reason)
  end
end

module InventoryHandler
  needs warehouse::WarehouseService

  on OrderPlaced do |e|
    .warehouse.reserve(e.order.items)
  end
end

# --- Actor for stateful concurrent work ---
actor PaymentProcessor
  needs gateway::PaymentGateway

  receive :charge(order)
    try
      .gateway.charge(order.total)
      reply :ok
    on fail as e
      emit PaymentFailed.new(order: order, reason: e.message)
      reply :failed
    end
  end
end

# --- App Wiring ---
app = Container.new()
app.register(Database, PostgresDB.new())
app.register(Mailer, SMTPMailer.new())
app.register(OrderValidator, StrictValidator.new())
app.register(WarehouseService, LocalWarehouse.new())
app.register(PaymentGateway, StripeGateway.new())

order_service = app.resolve(OrderService)
app.resolve(NotificationHandler)
app.resolve(InventoryHandler)
payment = app.resolve(PaymentProcessor)

supervisor AppSupervisor
  strategy :one_for_one
  supervise payment
end

AppSupervisor.start!

# --- Use it ---
order_service.place_order(new_order)
# 1. Validates order        (via injected validator)
# 2. Saves to DB            (via injected db)
# 3. Emits OrderPlaced
# 4. Sends email            (async, via NotificationHandler)
# 5. Reserves stock         (async, via InventoryHandler)
```

### 9.3 Specifications

The specification pattern allows composable business rules.

```opal
import Spec.Specification

class Person
  needs name::String
  needs age::Int32
  needs place_of_birth::String
end

class OverAgeSpec as Specification
  def is_satisfied_by(person::Person) -> Bool
    person.age >= 21
  end
end

class BornAtSpec as Specification
  needs born_at::String

  def is_satisfied_by(person::Person) -> Bool
    person.place_of_birth == .born_at
  end
end

claudio = Person.new(name: "claudio", age: 15, place_of_birth: "CA")
andrea = Person.new(name: "andrea", age: 21, place_of_birth: "CT")
people = [claudio, andrea]

over_age = OverAgeSpec.new()
over_age_people = people.filter(|p| over_age.is_satisfied_by(p))  # => [andrea]

californian = BornAtSpec.new(born_at: "CA")

# Logically combining business rules
spec = over_age.not().and(californian)
some_people = people.filter(|p| spec.is_satisfied_by(p))  # => [claudio]
```

### 9.4 Settings

> See [Validation & Settings](docs/features/validation-and-settings.md) for the full design rationale.

`model X as Settings` adds configuration loading from environment variables, config files, and `.env` files. Only the root model is `Settings` — nested groups are regular `model`s.

```opal
# Nested groups are regular models
model DatabaseSettings
  needs host::String = "localhost"
  needs port::Int32 = 5432
  needs name::String = "opal_dev"
  needs pool_size::Int32 = 5 where |v| v > 0
end

model CacheSettings
  needs host::String = "localhost"
  needs port::Int32 = 6379
  needs ttl::Int32 = 3600 where |v| v > 0
end

# Only the root is Settings
model AppSettings as Settings
  needs debug::Bool = false
  needs secret_key::String
  needs log_level::String = "info" where |v| v in ["debug", "info", "warn", "error"]
  needs db::DatabaseSettings
  needs cache::CacheSettings
end

# Load — root handles everything
settings = AppSettings.load(
  env_prefix: "OPAL_",
  config: "config.toml"
)

settings.debug            # => from OPAL_DEBUG
settings.db.host          # => from OPAL_DB__HOST or config or "localhost"
settings.cache.ttl        # => from OPAL_CACHE__TTL or config or 3600
settings.secret_key       # => required — raises SettingsError if missing
```

**Source priority (lowest to highest):**
1. Field defaults in model definition
2. Config file (TOML, JSON)
3. `.env` file
4. Environment variables
5. Explicit keyword arguments to `.load()`

**Settings rules:**
- `model X as Settings` makes the root a settings model with `.load()`.
- Nested groups are regular `model` — only the root loads from sources.
- Env delimiter defaults to `__`, configurable via `env_delimiter:` in `.load()`.
- Type coercion from env strings: `"true"` -> Bool, `"5432"` -> Int32, `"a,b,c"` -> List(String).
- Required fields (no default) raise `SettingsError` if missing from all sources.
- All validation runs after merging — same as regular models.

---

## 10. Metaprogramming

> See [Metaprogramming Design](docs/features/metaprogramming.md) for the full design rationale.

Opal's metaprogramming system is Julia-inspired, adapted to Opal's `end`-block syntax and `:symbol` conventions. It provides quoting, interpolation, macros, and AST manipulation as first-class features.

**Core principles:**
- **Hygienic by default.** Macro-introduced variables don't leak into the caller's scope. Explicit `esc()` to opt out.
- **Valid AST only.** Macros produce Opal AST nodes, not arbitrary text. No C-preprocessor-style pitfalls.
- **No generated functions.** Opal's multiple dispatch + macros covers the same ground — YAGNI.
- **Subdomains as macro packages.** Users and Opal itself can define domain-specific extensions as packages of macros.

### 10.1 Quoting — Code as Data

Code is captured as `Expr` (AST node) using `quote ... end`. Inside a quote, `$` interpolates values.

#### Basic Quoting

```opal
# Capture code as data
ast = quote x + y * 2 end
typeof(ast)   # => Expr
ast.head      # => :call
ast.args      # => [:+, :x, Expr(:call, :*, :y, 2)]

# Multi-line quoting
ast = quote
  x = 1
  y = 2
  x + y
end
```

#### Interpolation

```opal
# Splice runtime values into the AST
name = :greet
message = "hello"
ast = quote
  def $name()
    print($message)
  end
end
# ast represents: def greet() print("hello") end

# Splat interpolation for lists
params = [:a, :b, :c]
ast = quote f($params...) end
# ast represents: f(a, b, c)
```

#### Programmatic AST Construction

```opal
# Build AST without quoting
ast = Expr.new(:call, :+, 1, 2)
eval(ast)  # => 3

# Equivalent to:
ast = quote 1 + 2 end
eval(ast)  # => 3
```

#### Rules

- `quote ... end` returns an `Expr` — code as a manipulable data structure.
- `$expr` inside a quote splices the value of `expr` into the AST at construction time.
- `$list...` splats a list of expressions into argument position.
- `Expr.new(head, args...)` constructs AST nodes programmatically.
- `eval(expr)` evaluates an `Expr` at runtime (metaprogramming use only).

### 10.2 Macros

Macros receive AST at parse time and return transformed AST. They're hygienic by default.

#### Basic Macros

```opal
macro say_hello()
  quote
    print("Hello, world!")
  end
end

@say_hello  # => "Hello, world!"
```

#### Macros with Arguments

```opal
macro say_hello(name)
  quote
    print(f"Hello, {$name}")
  end
end

@say_hello "claudio"  # => "Hello, claudio"
```

#### Hygiene

Variables introduced inside a macro's `quote` are scoped to the macro — they don't shadow or leak into the caller's scope.

```opal
macro measure(body)
  quote
    start = Time.now()
    result = $body
    elapsed = Time.since(start)
    print(f"Took {elapsed}")
    result
  end
end

# Safe — caller's 'start' is NOT shadowed
start = "hello"
@measure do
  expensive_operation()
end
print(start)  # still "hello"
```

#### Escaping Hygiene

Use `esc(expr)` to explicitly inject an expression into the caller's scope:

```opal
macro define_var(name, value)
  quote
    $(esc(name)) = $value
  end
end

@define_var x, 42
print(x)  # => 42 (x exists in caller's scope because of esc)
```

#### Debugging Macros

```opal
# See what a macro expands to without executing it
macroexpand(@measure do 1 + 1 end)
# => Expr representing the expanded code
```

#### Rules

- `macro name(params) ... end` defines a macro. The body must return an `Expr`.
- `@name args` invokes a macro at parse time.
- Macros receive arguments as `Expr` (AST), not evaluated values.
- **Hygienic by default:** variables in macro quotes don't leak.
- `esc(expr)` escapes into the caller's scope (opt-in).
- `macroexpand(@name args)` shows expansion without executing.

### 10.3 AST Reflection & Introspection

#### Inspecting Expressions

```opal
ast = quote x + y * 2 end
ast.dump()
# Expr(:call, :+,
#   :x,
#   Expr(:call, :*, :y, 2))

ast.head       # => :call
ast.args       # => [:+, :x, Expr(:call, :*, :y, 2)]
ast.args[0]    # => :+ (the operator)
ast.args[1]    # => :x
```

#### Transforming AST

```opal
def double_literals(expr::Expr)
  match expr
    case n::Int32
      n * 2
    case Expr(head, args)
      Expr.new(head, args.map(|a| double_literals(a))...)
    case other
      other
  end
end

ast = quote 1 + 2 * 3 end
doubled = double_literals(ast)
eval(doubled)  # => eval(2 + 4 * 6) => 26
```

#### Runtime Introspection

```opal
# Introspect functions
methods(greet)         # => list of dispatch variants
typeof(greet)          # => Function
code_ast(greet)        # => the Expr representing the function body

# Introspect classes
User.fields()          # => [(:name, String), (:email, String), (:age, Int32)]
User.methods()         # => [:to_json, :from_json, :new, ...]
User.needs()           # => [(:db, Database), (:mailer, Mailer)]
User.implements()      # => [Printable, Comparable]
```

### 10.4 Practical Macro Examples

#### Code Generation — JSON Serialization

```opal
macro json_serializable(class_def)
  fields = class_def.needs_fields()

  to_json = quote
    def to_json()
      JSON.object($(generate_field_pairs(fields)...))
    end
  end

  from_json = quote
    def self.from_json(data::String)
      parsed = JSON.parse(data)
      self.new($(generate_from_json(fields)...))
    end
  end

  class_def.add_methods(to_json, from_json)
end

@json_serializable
class User
  needs name::String
  needs email::String
  needs age::Int32
end

user = User.new(name: "claudio", email: "c@opal.dev", age: 15)
user.to_json()   # => '{"name":"claudio","email":"c@opal.dev","age":15}'
User.from_json('{"name":"claudio","email":"c@opal.dev","age":15}')
```

#### DSL Creation — Test Framework

```opal
macro test(name, body)
  quote
    try
      $body
      Test.pass($name)
    on fail as e
      Test.fail($name, e.message)
    end
  end
end

macro describe(name, body)
  quote
    Test.group($name)
    $body
    Test.end_group()
  end
end

@describe "Math" do
  @test "addition" do
    assert_eq(2 + 2, 4)
  end

  @test "negative numbers" do
    assert_eq(-1 + 1, 0)
  end
end
```

This is how Opal's built-in test framework (`OpalTest` subdomain) is implemented under the hood. See [Testing](#testing) for the full testing specification.

#### Debugging — @debug Macro

```opal
macro debug(expr)
  name = string(expr)
  quote
    value = $expr
    print(f"Debug: {$name} = {value}")
    value
  end
end

x = 42
@debug x * 2 + 1  # => "Debug: x * 2 + 1 = 85"
```

#### Memoization

```opal
macro memoize(fn_def)
  fn_name = fn_def.name
  quote
    _cache = {:}

    def $fn_name($(fn_def.params...))
      key = ($(fn_def.params...),)
      if _cache.has?(key)
        return _cache[key]
      end
      result = $(fn_def.body)
      _cache[key] = result
      result
    end
  end
end

@memoize
def fibonacci(n::Int32) -> Int32
  if n <= 1 then n else fibonacci(n - 1) + fibonacci(n - 2) end
end
```

### 10.5 Self-Hosting Potential

With quoting + macros, some of Opal's own features could be defined in Opal itself. This doesn't mean they *must* be — core keywords can stay in the parser for performance and clarity. But the macro system is powerful enough that users could build equivalent constructs.

#### What Stays in the Parser (Core Syntax)

These are fundamental to the language and must be parsed natively:

- `def`, `class`, `module`, `actor`, `if`, `for`, `while`, `match`, `try`
- `quote`, `macro`, `$` (metaprogramming primitives)
- `=`, `.`, `::`, operators

#### What Could Be Macros

These are essentially code transformations and could theoretically be implemented as macros:

- `needs` — generates constructor injection
- `event` — generates an immutable data class
- `emit` — generates actor-based event dispatch
- `on` — generates event handler registration
- `guard` — generates pre-condition checks
- `supervisor` — generates actor supervision setup

Whether they stay as keywords or become macros is an implementation decision. The key insight is that the macro system is *expressive enough* to define them.

### 10.6 Domain Extension Guidelines

Opal's macro system enables **subdomains** — packages of macros that extend the language for a specific problem domain. This is how Opal and its ecosystem grow without bloating the core language.

#### What is a Subdomain?

A subdomain is a module that exports macros, providing domain-specific syntax and abstractions. It's a mini-language within Opal, tailored to a particular problem.

#### Creating a Subdomain

A subdomain is a standard Opal module that exports macros:

```opal
# File: opal_web/macros.opl
module OpalWeb
  # Route definition DSL
  macro get(path, body)
    quote
      app.route("GET", $path, |req, res|
        $body
      end)
    end
  end

  macro post(path, body)
    quote
      app.route("POST", $path, |req, res|
        $body
      end)
    end
  end

  # Middleware DSL
  macro middleware(name, body)
    quote
      app.use($name, |req, res, next|
        $body
        next()
      end)
    end
  end
end
```

```opal
# Usage — the subdomain provides web-specific syntax
import OpalWeb

@middleware :logging do
  print(f"[{Time.now()}] {req.method} {req.path}")
end

@get "/" do
  res.send("Hello, world!")
end

@post "/users" do
  user = User.from_json(req.body)
  user.save()
  res.json(user.to_json())
end
```

#### Subdomain Guidelines

**1. Name macros as verbs or nouns that read naturally at the call site.**

```opal
# Good — reads like a sentence
@get "/users" do ... end
@test "addition" do ... end
@memoize def fib(n) ... end

# Bad — unclear at the call site
@r "/users" do ... end
@m def fib(n) ... end
```

**2. One macro per concept. Don't overload a macro to do multiple things.**

```opal
# Good — separate macros for separate concepts
@get "/users" do ... end
@post "/users" do ... end

# Bad — one macro with a mode parameter
@route "GET", "/users" do ... end
```

**3. Macros should produce valid, inspectable code.**

```opal
# Always test with macroexpand
macroexpand(@get "/" do res.send("hello") end)
# Should produce clean, readable Opal
```

**4. Document what the macro expands to.**

Every macro should include a comment or doc showing the equivalent non-macro code:

```opal
# @get "/" do ... end
# expands to:
# app.route("GET", "/", |req, res| ... end)
```

**5. Prefer macros that compose with existing features.**

Macros should work with guards, pattern matching, DI, and events — not bypass them:

```opal
# Good — composes with guards
@positive
@memoize
def sqrt(x::Float64) -> Float64
  x ** 0.5
end

# Good — composes with needs
@json_serializable
class User
  needs name::String  # needs still works inside macro-processed class
end
```

**6. Subdomains should be importable and scoped.**

```opal
# Import a subdomain
import OpalWeb          # all macros available
import OpalWeb.{get, post}  # selective import

# Macros from different subdomains don't conflict
import OpalWeb
import OpalTest
# @get is from OpalWeb, @test is from OpalTest
```

#### Opal's Own Subdomains

Opal's standard library can use this same model. Rather than hardcoding every feature, the stdlib provides subdomains:

| Subdomain | Provides | Macros |
|---|---|---|
| `Opal.Core` | Core language (parser-level) | None — native syntax |
| `Opal.Test` | Testing framework | `@test`, `@describe`, `@assert` |
| `Opal.Web` | Web framework | `@get`, `@post`, `@middleware` |
| `Opal.Data` | Database/ORM | `@schema`, `@migration`, `@query` |
| `Opal.Bench` | Benchmarking | `@benchmark`, `@profile` |
| `Opal.Debug` | Debugging tools | `@debug`, `@trace`, `@breakpoint` |
| `Opal.Serial` | Serialization | `@json_serializable`, `@msgpack` |

Each subdomain is an independent package — you only import what you use.

#### Summary

**What Opal Gets from Julia:**

| Julia Feature | Opal Adaptation |
|---|---|
| `:(expr)` quoting | `quote expr end` / `quote ... end` |
| `$var` interpolation | `$var` (identical) |
| `Expr` type | `Expr` type with `.head`, `.args`, `.dump()` |
| `macro ... end` | `macro ... end` (identical structure) |
| `@name` invocation | `@name` (identical) |
| `eval()` | `eval()` (identical) |
| `esc()` | `esc()` (identical) |
| `macroexpand()` | `macroexpand()` (identical) |
| `@generated function` | Skipped — multiple dispatch + macros covers it |
| Non-standard string literals | Already in Opal (`f"..."`, `r"..."`, `t"..."`) |

**New Keywords:**

| Keyword | Purpose |
|---|---|
| `quote ... end` | Capture code as AST |
| `$` (inside quote) | Interpolate into AST |
| `macro ... end` | Define a macro |
| `@name` | Invoke a macro |

---

## 11. Standard Library

Opal ships with a standard library organized into modules:

| Module | Purpose |
|---|---|
| `IO` | Standard input/output, printing, reading |
| `File` | File reading, writing, path manipulation |
| `Net` | HTTP client/server, TCP/UDP sockets |
| `Math` | Mathematical functions and constants |
| `Collections` | Advanced data structures (Set, Queue, Stack, etc.) |
| `String` | String manipulation, formatting, template processing |
| `Time` | Date, time, duration, formatting |
| `JSON` | JSON parsing and generation |
| `Test` | Built-in test framework — `@describe`, `@test`, assertions, lifecycle hooks |
| `Mock` | Mock creation for tests — `Mock.new(Protocol)`, stubs, call verification |
| `Spec` | Specification pattern base classes |
| `Container` | Optional dependency injection container for large apps |
| `Iter` | `Iterable` and `Iterator` protocols, lazy sequences |
| `Option` | `Option(T)` enum — `Some(value)` or `None` for explicit nullable handling |
| `Result` | `Result(T, E)` enum — `Ok(value)` or `Err(error)` for error handling |
| `Settings` | Base for settings models — env/config/file loading with source priority |

```opal
import IO
import File
import JSON

# Read a JSON config file
content = File.read("config.json")
config = JSON.parse(content)
IO.print(f"Loaded {config.keys().length} settings")
```

```opal
import Test

Test.describe("Math operations")
  Test.it("adds two numbers")
    Test.assert_eq(2 + 2, 4)
  end

  Test.it("handles negative numbers")
    Test.assert_eq(-1 + 1, 0)
  end
end
```

---

## 12. Tooling

### Testing

> See [Testing Framework](docs/features/testing.md) for the full design rationale.

Tests use `.topl` files and are discovered automatically by `opal test`. The `@describe`/`@test` macros from `OpalTest` are sugar for the `Test` module API.

```opal
# tests/math.topl
import OpalTest

@describe "Math" do
  @test "addition" do
    assert_eq(2 + 2, 4)
  end

  @test "negative numbers" do
    assert_eq(-1 + 1, 0)
  end
end
```

#### Assertions

```opal
assert_eq(actual, expected)         # equality
assert_ne(actual, expected)         # inequality
assert_true(expr)                   # boolean true
assert_false(expr)                  # boolean false
assert_raises(ErrorType) do ... end # expects exception
assert_match(pattern, value)        # pattern matching
```

#### Lifecycle Hooks

```opal
@describe "Database" do
  @before_all do
    .db = TestDB.create()
  end

  @after_all do
    .db.destroy()
  end

  @before_each do
    .db.begin_transaction()
  end

  @after_each do
    .db.rollback()
  end

  @test "insert" do
    .db.insert({name: "test"})
    assert_eq(.db.count(), 1)
  end
end
```

Hooks run in order: `@before_all` (once) -> `@before_each` -> test -> `@after_each` -> `@after_all` (once). Nested `@describe` blocks inherit parent hooks.

#### Mocking

Two approaches: DI-based (swap via `needs`) and the `Mock` helper for quick stubs.

```opal
import Mock

mock_db = Mock.new(Database)
mock_db.stub(:save, true)
mock_db.stub(:find, |id| null)

service = OrderService.new(db: mock_db, mailer: mock_mailer)
service.place_order(order)

assert_true(mock_db.called?(:save))
assert_eq(mock_db.call_count(:save), 1)
```

#### Test Runner

```
$ opal test                        # run all .topl files
$ opal test tests/math.topl        # run one file
$ opal test --filter "addition"    # filter by name
```

### Project Scaffolding

```
$ opal init --type lib MyCoolProject
       create  MyCoolProject/.gitignore
       create  MyCoolProject/LICENSE
       create  MyCoolProject/README.md
       create  MyCoolProject/src/MyCoolProject.opl
       create  MyCoolProject/tests/MyCoolProject.topl
Initialized empty Git repository in ~/MyCoolProject/.git/
```

Templates are supported: `opal init --type web MyWebApp`, `opal init --type cli MyCLI`.

### Documentation Generation

```
$ opal docs MyCoolProject
       created  docs/libs/MyCoolProject.md
Documentation created in docs/ [should I publish to GHPages?].
```

### Linter

```
$ opal lint src/
  src/main.opl:12:5  warning  unused variable 'temp'
  src/main.opl:28:1  error    unreachable code after return
  2 issues (1 error, 1 warning)
```

### Package Manager

Integrated package management, inspired by Poetry/Cargo.

```
$ opal pkg add http_server@1.2
  Added http_server 1.2.0 to dependencies
$ opal pkg install
  Installing 3 packages...
  Done.
```

```opal
# Using with for DSL-style configuration blocks
import Nginx

my_site = Nginx.create with {
  user:              "www www",
  worker_processes:  5,
  error_log:         "logs/error.log",
  pid:               "logs/nginx.pid"
}

my_site.http with {
  index:        "index.html index.htm index.php",
  default_type: "application/octet-stream"
}

my_site.http.server with {
  listen:      80,
  server_name: "domain.com",
  access_log:  "logs/domain.access.log main"
}

my_site.serve!
```

The `with` keyword passes a dictionary of configuration values to the preceding expression. It is reserved for DSL-style configuration blocks like the above. Object creation uses `.new()` with named arguments; string interpolation uses f-strings (or t-strings for safe templating).

---

## 13. Pretotyping

([No, it's not a typo.](http://www.pretotyping.org/))

Opal aims to make simple web applications as concise as possible:

**Python (Flask):**
```python
from flask import Flask
app = Flask(__name__)

@app.route("/")
def hello():
    return "Hello World!"

if __name__ == "__main__":
    app.run()
```

**Opal equivalent:**
```opal
import OpalWeb

app = OpalWeb.App.new("app name")

@get "/" do
  "Hello world!"
end

app.run!
```

---

## Appendix

### A. Links

- Keyboard and symbols
  - [Help pages Mac OS keyboard](https://forlang.wsu.edu/help-pages/help-pages-keyboards-os-x/)
  - [Special symbols](https://discussions.apple.com/thread/6535997?start=0&tstart=0)
- Generic language
  - [Algebraic data types: without them, we all suffer](https://genericlanguage.wordpress.com/2015/06/09/algebraic-data-types-without-them-we-all-suffer/)
  - [Advice on writing a programming language](https://genericlanguage.wordpress.com/2014/02/04/advice-on-writing-a-programming-language/)
  - [Programmers as glue factories](https://genericlanguage.wordpress.com/2014/03/29/programmers-as-glue-factories/)
  - [More on abstraction](https://genericlanguage.wordpress.com/2015/01/08/more-on-abstraction/)
- [ANTLR](http://www.antlr.org/)
- [ATS language](http://www.ats-lang.org/)
- Tokenizer -> [Lexical scanning in Go](https://blog.golang.org/two-go-talks-lexical-scanning-in-go-and)
- Parser -> [Top Down Operator Precedence](http://javascript.crockford.com/tdop/tdop.html)
- [Akka / actor model / concurrency](http://readwrite.com/2014/07/10/akka-jonas-boner-concurrency-distributed-computing-internet-of-things/)
  - [pyutil](https://github.com/zooko/pyutil)
  - [ANTLR4 Python target](https://github.com/antlr/antlr4/blob/master/doc/python-target.md)
- Ruby [BNF](https://www.cse.buffalo.edu//~regan/cse305/RubyBNF.pdf)

### B. Topics

- BNF [1](https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_Form), [2](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_Form), [3](https://en.wikipedia.org/wiki/Augmented_Backus%E2%80%93Naur_Form)
- Type inference in [Crystal](http://crystal-lang.org/2013/09/23/type-inference-part-1.html)
- [ANTLR Ruby grammar](https://github.com/antlr/grammars-v4/tree/master/ruby)
- [Memetalk bits.py](https://github.com/thiago-silva/memetalk/blob/master/sugarfoot/pyutils/bits.py)
- [Almost Y combinator in JavaScript](http://blog.klipse.tech/lambda/2016/08/10/almost-y-combinator-javascript.html)
- [Set theory](https://en.wikipedia.org/wiki/Set_theory)

### C. Tutorials

- [How to write an interpreter?](https://www.youtube.com/watch?v=1h1mM7VwNGo)
- [Let's Build a Simple Interpreter](https://ruslanspivak.com/lsbasi-part1/)

### D. References

- [Mini](https://github.com/keleshev/mini)
- [Memetalk](https://github.com/thiago-silva/memetalk)
- Crystal
  - [kernel.cr](https://github.com/crystal-lang/crystal/blob/master/src/kernel.cr)
- Pixie
  - [target.py](https://github.com/pixie-lang/pixie/blob/master/target.py)
- Kermit
  - [Kermit interpreter](https://bitbucket.org/pypy/example-interpreter/src/a00d0f9c36f151112d35708b82035a541fe6f16f/kermit/?at=default)
- [RPython/RPLY interpreter](http://joshsharp.com.au/blog/rpython-rply-interpreter-1.html)
  - [Braid](https://github.com/joshsharp/braid)
- Cycy
  - [Cycy parser](https://github.com/Magnetic/cycy/blob/master/cycy/parser/core.py)
- Bytecode
  - [lang-rio](https://libraries.io/github/edcrypt/lang-rio)

### E. Ideas

- Go's [GoVet / GoFmt](https://golang.org/cmd/gofmt/) — built-in linting and formatting
- Optional type system (e.g., method signatures)
- Use of Examples to define arguments, maybe classes or entities
- Guard clauses (also using Examples)
- Gather data of usage to suggest optimization
- Intelligent Assistant to help during development time (providing Examples, or based on historical data)
- Runtime modes
  - **Rock** — Simple, single-box application, local memory, local disk
  - **Rough** — Halfway between Rock and Polished
  - **Polished** — Production-ready, cluster, shared memory, virtual networked storage
- Internal Supervisor/Systemd support
- Pattern matching like [Rust patterns](https://doc.rust-lang.org/book/second-edition/ch18-03-pattern-syntax.html)
- Syntax references
  - If — [Crystal suffix syntax](http://crystal-lang.org/docs/syntax_and_semantics/as_a_suffix.html)
  - Range — [Crystal range literals](http://crystal-lang.org/docs/syntax_and_semantics/literals/range.html)
- Unicode references — [Unicode table](http://unicode-table.com/en/#telugu)
- Syntax highlighter (cobalt) — [markup.su](http://markup.su/highlighter/)
