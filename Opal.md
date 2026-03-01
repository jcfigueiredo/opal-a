# Opal — Opinionated Programming Algorithmic Language

[...towards a better programming...](http://www.chris-granger.com/2014/03/27/toward-a-better-programming/)

Opal is a dynamic, interpreted, object-oriented language with first-class functions, multiple dispatch, an actor-based concurrency model, and a gradual type system. It prioritizes readability, explicitness, and demonstrating sound software engineering concepts.

---

## 1. Design Philosophy

- **Readability is paramount.** Code is read far more than it is written.
- **One explicit way.** There should be one obvious way to do something — no alternative syntax for the same operation.
- **Software engineering concepts are first-class.** Specifications, guards, null objects, and the actor model are built into the language, not bolted on.
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

<class_def>     ::= "class" IDENTIFIER ("<" IDENTIFIER)? NEWLINE <class_body> "end"
<class_body>    ::= (<function_def> | <assignment>)*

<module_def>    ::= "module" IDENTIFIER NEWLINE <module_body> "end"
<module_body>   ::= (<function_def> | <class_def> | <assignment>)*

<match_expr>    ::= "match" <expression> NEWLINE <case_clause>+ "end"
<case_clause>   ::= "case" <pattern> NEWLINE <block>

<try_expr>      ::= "try" NEWLINE <block>
                     ("on" "fail" TYPE ("as" IDENTIFIER)? NEWLINE <block>)*
                     ("ensure" NEWLINE <block>)?
                     "end"

<actor_def>     ::= "actor" IDENTIFIER NEWLINE <actor_body> "end"
<actor_body>    ::= (<function_def> | <receive_clause>)*
<receive_clause>::= "receive" SYMBOL NEWLINE <block> "end"

<block>         ::= <statement>+

<binary_op>     ::= "+" | "-" | "*" | "/" | "%" | "**"
                   | "==" | "!=" | "<" | ">" | "<=" | ">="
                   | "and" | "or"
                   | ".." | "..."
<unary_op>      ::= "-" | "not"
```

---

## 4. Syntax & Semantics

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

### 4.4 Collections

#### 4.4.1 Lists

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

#### 4.4.2 Tuples

Tuples are ordered, immutable sequences. They use parentheses.

```opal
()                              # empty tuple
point = (10, 20)                # Tuple(Int32, Int32)
record = (:banana, "apple", '🙈')  # Tuple(Symbol, String, Char)

record[0]                       # => :banana
record[1]                       # => "apple"
record[2]                       # => '🙈'
```

#### 4.4.3 Dictionaries

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

#### 4.4.4 Ranges

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

### 4.5 Regex

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

### 4.6 Operators

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

### 4.7 Conditionals

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

### 4.8 Loops & Iteration

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

### 4.9 Functions & Closures

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

### 4.10 Classes & Methods

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

### 4.11 Modules & Namespaces

Modules group related functions, classes, and constants.

```opal
module Math
  PI = 3.14159265358979

  def abs(x::Number)
    if x < 0 then -x else x end
  end

  def max(a, b)
    if a > b then a else b end
  end
end

Math.abs(-5)   # => 5
Math.PI        # => 3.14159265358979
```

```opal
module Geometry
  class Circle
    def :init(radius::Float32)
      .radius = radius
    end

    def area()
      Math.PI * .radius ** 2
    end
  end
end

c = Geometry.Circle.new(radius: 5.0)
c.area()  # => 78.539...
```

### 4.12 Visibility / Access Control

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

### 4.13 Interfaces / Protocols

Protocols define a contract that classes must fulfill.

```opal
protocol Printable
  def to_string() -> String
end

protocol Comparable
  def compare_to(other) -> Int32
end

class Person implements Printable
  def :init(name, age)
    .name = name
    .age = age
  end

  def to_string()
    f"{.name}, age {.age}"
  end
end

# Multiple protocols
class Temperature implements Printable, Comparable
  def :init(degrees::Float32)
    .degrees = degrees
  end

  def to_string()
    f"{.degrees}°"
  end

  def compare_to(other::Temperature)
    if .degrees < other.degrees then -1
    else if .degrees > other.degrees then 1
    else 0
    end
  end
end
```

### 4.14 Guards & Rules

Guards validate data before a function body executes.

```opal
# Standalone guard function
guard old_enough(age) fails :too_young
  return age >= 18
end

class Registration
  # Type guards on parameters
  @name in (String, Symbol)
  @email in (String)
  def register(name, email)
    print(f"Registered {name} with {email}")
  end

  # Business rule guard with external function
  @old_enough
  def register_voter(name, age)
    print(f"{name} registered to vote")
  end
end
```

```opal
# Guard with custom error
guard positive(value) fails :must_be_positive
  return value > 0
end

@positive
def sqrt(value::Float64) -> Float64
  # only executes if value > 0
  value ** 0.5
end
```

### 4.15 Null Objects

Null objects provide default behavior instead of null checks.

```opal
class Person
  def :init(name, age)
    .name = name
    .age = age
  end

  def greet()
    print(f"Hi, I'm {.name}")
  end
end

# Define a Null Object by extending Nullable
class NullPerson as Nullable:Person
  def greet(*)
    print("Hi, I don't want to say my name")
  end
end

# Or shortcut: create a null variant with default values
class NullPerson as Person defaults {name: "anonymous", age: 0}
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

### 4.16 Pattern Matching

```opal
match value
  case 0
    "zero"
  case 1..10
    "small"
  case x if x > 100
    "large"
  case _
    "other"
end
```

```opal
# Destructuring tuples
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

# Matching on type
match response
  case s::String
    print(s)
  case n::Int32
    print(f"code: {n}")
  case (status, body)
    print(f"{status}: {body}")
end
```

### 4.17 Multiple Dispatch

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

### 4.18 Error Handling

Opal uses `try` / `on fail` / `ensure` for structured error handling.

```opal
# Basic
def pine(x::Int32)
  if x < 0
    fail("invalid value")
  end
  x + 1
end

def cone(y::Int32) -> Int32
  try
    q = pine(y)
  on fail as e
    print(f"caught: {e.message}")
    return 0
  end
  q
end
```

```opal
# Typed error handling
try
  result = risky_operation()
on fail FileNotFound as e
  log(f"File missing: {e.message}")
on fail NetworkError as e
  retry after 1.second
on fail as e
  log(f"Unexpected: {e.message}")
  fail(e)  # re-raise
ensure
  cleanup()
end
```

`ensure` always executes, whether the block succeeded or failed.

### 4.19 Concurrency (Actor Model)

Actors are concurrent entities that communicate through message passing. Each actor has its own isolated state.

```opal
actor Counter
  def :init()
    .count = 0
  end

  def increment!()
    .count = .count + 1
  end

  receive :get_count
    reply .count
  end

  receive :reset
    .count = 0
    reply :ok
  end
end

c = Counter.new()
c.increment!()
c.increment!()
c.send(:get_count)  # => 2
c.send(:reset)      # => :ok
c.send(:get_count)  # => 0
```

```opal
# Actors communicating
actor Logger
  receive :log(message)
    print(f"[LOG] {message}")
    reply :ok
  end
end

actor Worker
  def :init(logger)
    .logger = logger
  end

  def do_work(task)
    result = process(task)
    .logger.send(:log, f"Completed: {task}")
    result
  end
end

logger = Logger.new()
worker = Worker.new(logger)
worker.do_work("build report")
```

### 4.20 Specifications

The specification pattern allows composable business rules.

```opal
import "patterns.Specification"

class Person
  def :init(name, age, place_of_birth)
    .name = name
    .age = age
    .place_of_birth = place_of_birth
  end
end

class OverAgeSpec as Specification
  @person in (Person)
  def is_satisfied_by(person)
    person.age >= 21
  end
end

class BornAtSpec as Specification
  @born_at in (String)
  def :init(born_at)
    .born_at = born_at
  end

  @person in (Person)
  def is_satisfied_by(person)
    person.place_of_birth == .born_at
  end
end

claudio = Person.new(name: "claudio", age: 15, place_of_birth: "CA")
andrea = Person.new(name: "andrea", age: 21, place_of_birth: "CT")
people = [claudio, andrea]

over_age = OverAgeSpec.new()
over_age_people = people.where(over_age.is_satisfied_by)  # => [andrea]

californian = BornAtSpec.new(born_at: "CA")

# Logically combining business rules
californian_and_under_21 = not over_age and californian
some_people = people.where(californian_and_under_21.is_satisfied_by)  # => [claudio]
```

### 4.21 Type System

Opal uses **gradual typing**: unannotated code is dynamic, annotated code is checked.

```opal
# No annotations — fully dynamic
def add(a, b)
  a + b
end

# Annotated — type-checked at boundaries
def add(a::Int32, b::Int32) -> Int32
  a + b
end

# Type annotation syntax: :: for types
name::String = "claudio"
age::Int32 = 15

# Explicit casting with `as`
x = 3.14 as Int32   # => 3

# Optional types
def find(id::Int32) -> Person?
  # may return null
end
```

**Core types:** `Int8`, `Int16`, `Int32`, `Int64`, `Float32`, `Float64`, `Bool`, `Char`, `String`, `Template`, `Symbol`, `Null`, `List(T)`, `Tuple(...)`, `Dict(K, V)`, `Range(T)`, `Regex`.

**Type rules:**
- Unannotated parameters and variables are dynamic — no checking.
- Annotated parameters are checked at call sites.
- Return type annotations are checked at function exit.
- `as` performs explicit type conversion.
- `?` suffix denotes a nullable type (e.g., `String?` means `String | Null`).

### 4.22 Standard Library

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
| `Test` | Built-in test framework, assertions |
| `Mock` | Mocking and stubbing for tests |
| `Spec` | Specification pattern base classes |

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

## 5. Tooling

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
# Importing external packages
import Roman@"https://github.com/keleshev/rome/tree/0.0.2"

# Using with for DSL-style configuration blocks
import nginx

my_site = nginx.create with {
  user:              "www www",
  worker_processes:  5,
  error_log:         "logs/error.log",
  pid:               "logs/nginx.pid"
}!

my_site.http with {
  index:        "index.html index.htm index.php",
  default_type: "application/octet-stream"
}!

my_site.http.server with {
  listen:      80,
  server_name: "domain.com",
  access_log:  "logs/domain.access.log main"
}.serve!
```

The `with` keyword is reserved for DSL-style configuration blocks like the above. Object creation uses `.new()` with named arguments; string interpolation uses f-strings (or t-strings for safe templating).

---

## 6. Pretotyping

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
import Flask

app = Flask.new("app name")

app.get "/" and
  return "Hello world!"
end

# One-liner routes
app.get "/" and return "Hello world!"!
app.get "/foo" and return ("Hello world!", 200)!
app.get "/bar" and return ("http://foo.bar/", 301)!

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
