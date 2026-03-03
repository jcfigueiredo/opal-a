# Literals

---

## Overview

Literals are the notation for representing fixed values in Opal source code. This covers null, booleans, numbers (with numeric semantics), strings (all quote styles, prefixes, escape sequences, and methods), and symbols (including symbol sets). Together they define what values look like in Opal.

---

## 1. Null

```opal
value = null
```

## 2. Booleans

```opal
are_you_here = true
are_you_there = false
```

## 3. Numbers

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

### Numeric Semantics

```opal
# Integer division returns integer
5 / 2        # => 2 (truncated)
5.0 / 2.0    # => 2.5 (float division)
5 / 2.0      # => 2.5 (promoted to float)
5.to_f / 2   # => 2.5 (explicit conversion)

# Modulo
5 % 2        # => 1

# Overflow is a runtime error (safe by default)
x: Int32 = 2_147_483_647
x + 1        # raises OverflowError

# Explicit wrapping arithmetic when needed
x.wrapping_add(1)  # => -2_147_483_648
```

**Numeric rules:**
- Integer division truncates: `5 / 2 = 2`.
- Mixed int/float operations promote to float: `5 / 2.0 = 2.5`.
- Integer overflow raises `OverflowError`. Use `.wrapping_add()`, `.wrapping_mul()` etc. for unchecked arithmetic.
- Default types: integer literals are `Int32`, float literals are `Float32`.

## 4. Strings — Single and Double Quotes

Both single and double quotes produce strings. Use whichever avoids escaping:

```opal
name = "claudio"
name = 'claudio'           # identical
json = '{"name": "claudio"}'  # single quotes avoid escaping
html = "<p class='bold'>hi</p>"  # double quotes avoid escaping
```

Escape sequences work in both:

```opal
"hello\n"    # newline
'hello\n'    # also newline
"it's"       # apostrophe in double quotes
'it\'s'      # escaped in single quotes
"say \"hi\"" # escaped in double quotes
'say "hi"'   # no escaping needed
```

All escape sequences:

| Escape | Meaning |
|---|---|
| `\\` | Backslash |
| `\'` | Single quote |
| `\"` | Double quote |
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |
| `\e` | Escape |
| `\f` | Form feed |
| `\v` | Vertical tab |
| `\101` | Octal code point |
| `\u0041` | Unicode (4 hex digits) |
| `\u{1F52E}` | Unicode (1-6 hex digits) |

There is no separate `Char` type — a "character" is a length-1 string.

## 5. String Prefixes & Methods

Opal provides several string prefixes for different use cases.

**Regular strings** — single or double quotes, supports escape sequences:

```opal
name = "claudio"
move_message = "my move is ♘ to ♚"

# Escape sequences: \n, \t, \\, \", \', etc.
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

String prefixes (`f`, `r`, `t`) work with both single and double quotes: `f'Hello {name}'` is identical to `f"Hello {name}"`.

### String Methods

Strings are immutable UTF-8 sequences. All methods return new strings (never mutate).

```opal
# Querying
"hello".length              # => 5
"hello".empty?()            # => false
"hello".contains?("ell")    # => true
"hello".starts_with?("he")  # => true
"hello".ends_with?("lo")    # => true

# Transforming
"hello".upper()             # => "HELLO"
"hello".lower()             # => "hello"
"  hello  ".trim()          # => "hello"
"hello".replace("l", "r")   # => "herro"
"hello".reverse()           # => "olleh"
"ha" * 3                    # => "hahaha"
"-" * 40                    # => "----------------------------------------"

# Splitting & Joining
"a,b,c".split(",")          # => ["a", "b", "c"]
["a", "b", "c"].join(", ")  # => "a, b, c"

# Slicing
"hello"[0]                  # => "h"
"hello"[1..3]               # => "ell" (String)

# Conversion
"42".to_int()               # => 42 (or raises on invalid)
"3.14".to_float()           # => 3.14
42.to_string()              # => "42"
```

**String rules:**
- Strings are immutable UTF-8 sequences.
- Indexing a string returns a length-1 `String`. Slicing also returns a `String`.
- All transformation methods return new strings.

## 6. Symbols

Symbols are self-identifying constants. They do not need to be assigned a value.

```opal
:hi
:bye
:"I have spaces."
:really?
:yes!
```

### Symbol Sets (Typed Symbols)

Symbols can form **symbol sets** — lightweight type aliases that constrain which symbols are valid in a given context. This bridges dynamic atoms with static safety.

```opal
# Named symbol set — a union of symbol literals
type Status = :ok | :error | :pending
type HttpMethod = :get | :post | :put | :delete | :patch
type LogLevel = :debug | :info | :warn | :error

# Use as a type annotation
def handle(status: Status)
  match status
    case :ok      then print("success")
    case :error   then print("failure")
    case :pending then print("waiting")
  end
end

handle(:ok)       # works
handle(:unknown)  # TYPE ERROR: :unknown is not in Status

# Inline symbol constraint (no named type needed)
def log(level: :debug | :info | :warn | :error, message: String)
  print(f"[{level}] {message}")
end
```

**Symbol set rules:**
- `type Name = :a | :b | :c` defines a symbol set (a type alias of a union of symbol literals).
- Symbol sets participate in exhaustiveness checking — the compiler warns on incomplete match.
- `Symbol` remains the unconstrained type (accepts any symbol) — gradual typing.
- Symbol sets compose with unions, generics, and constraints.

**Symbol sets vs enums:** Symbol sets are for simple tags with no data. `enum` is for data-carrying variants:

```opal
# Symbol set — lightweight tags
type Direction = :north | :south | :east | :west

# Enum — data-carrying variants
enum Shape
  Circle(radius: Float64)
  Rect(width: Float64, height: Float64)
end
```
