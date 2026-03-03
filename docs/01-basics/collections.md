# Collections

---

## Overview

Opal provides four built-in collection types: lists (ordered, mutable), tuples (ordered, immutable), dictionaries (mutable key-value mappings), and ranges (iterable sequences). All collections that implement `Iterable` share a rich set of transformation, querying, and grouping methods. Comprehensions offer concise syntax for building lists and dicts from iteration.

---

## 1. Lists

Lists are ordered, mutable sequences.

```opal
[]                        # empty list
numbers = [1, 2, 3, 4, 5]
names = ["alice", "bob"]
mixed = [1, "hello", :ok] # List[Int32 | String | Symbol]

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

## 2. Tuples

Tuples are ordered, immutable sequences. They use parentheses.

```opal
()                              # empty tuple
point = (10, 20)                # Tuple[Int32, Int32]
record = (:banana, "apple", "🙈")  # Tuple[Symbol, String, String]

record[0]                       # => :banana
record[1]                       # => "apple"
record[2]                       # => "🙈"
```

## 3. Dictionaries

Dictionaries are mutable mappings of key-value pairs. Keys can be any immutable object and must be unique.

```opal
{:}                             # empty dict
{1: 2, 3: 4}                   # Dict[Int32, Int32]
{1: 2, "a": 3}                 # Dict[Int32 | String, Int32]
{"α": "alpha", "β": "beta"}    # Dict[String, String]
{:plane: "✈", :train: "🚂"}    # Dict[Symbol, String]

# Access
ages = {"alice": 30, "bob": 25}
ages["alice"]                   # => 30
ages["carol"] = 28              # insert new entry
```

## 4. Ranges

A range is constructed with a range literal. Types on both extremes must be the same.

```opal
1..10       # inclusive range: 1, 2, 3, ..., 10
1...10      # exclusive range: 1, 2, 3, ..., 9
"a".."z"    # character range

# Ranges are iterable
for i in 1..5
  print(i)
end
```

## 5. Collection Methods

All `Iterable` types (List, Range, etc.) support these methods:

```opal
numbers = [1, 2, 3, 4, 5]

# Transforming
numbers.map(|x| x * 2)                 # => [2, 4, 6, 8, 10]
numbers.filter(|x| x > 3)              # => [4, 5]
numbers.reduce(0, |acc, x| acc + x)    # => 15

# Querying
numbers.find(|x| x > 3)               # => 4 (first match, or null)
numbers.any?(|x| x > 3)               # => true
numbers.all?(|x| x > 0)               # => true
numbers.count(|x| x > 3)              # => 2

# Ordering
numbers.sort()                          # => [1, 2, 3, 4, 5]
numbers.sort(|a, b| b - a)             # => [5, 4, 3, 2, 1]
numbers.reverse()                       # => [5, 4, 3, 2, 1]

# Slicing
numbers.take(3)                         # => [1, 2, 3]
numbers.drop(3)                         # => [4, 5]

# Grouping
numbers.group_by(|x| if x > 3 then "big" else "small" end)
# => {"small": [1, 2, 3], "big": [4, 5]}

# Combining
[1, 2].zip([3, 4])                     # => [(1, 3), (2, 4)]
[[1, 2], [3, 4]].flatten()             # => [1, 2, 3, 4]

# Iteration
numbers.each(|x| print(x))             # prints each, returns null
```

## 6. Comprehensions

Comprehensions provide a concise syntax for building lists and dicts from iteration and filtering.

```opal
# List comprehension
squares = [x ** 2 for x in 1..10]

# With filter
even_squares = [x ** 2 for x in 1..10 if x % 2 == 0]

# Dict comprehension
name_lengths = {name: name.length for name in ["alice", "bob", "carol"]}

# Nested iteration
pairs = [(x, y) for x in 1..3 for y in 1..3 if x != y]

# With destructuring
adults = [name for (name, age) in people if age >= 18]
```

Comprehensions are sugar for `filter` + `map`. Both styles are available -- use whichever reads better for the situation.

## 7. Regex

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

## Summary

| Type | Literal | Mutable | Key Trait |
|---|---|---|---|
| List | `[1, 2, 3]` | Yes | Ordered sequence |
| Tuple | `(1, 2, 3)` | No | Immutable sequence |
| Dict | `{"a": 1}` | Yes | Key-value mapping |
| Range | `1..10` | No | Iterable sequence |
| Regex | `/pattern/` | No | Pattern matching |

| Method Category | Methods |
|---|---|
| Transforming | `map`, `filter`, `reduce` |
| Querying | `find`, `any?`, `all?`, `count` |
| Ordering | `sort`, `reverse` |
| Slicing | `take`, `drop` |
| Grouping | `group_by` |
| Combining | `zip`, `flatten` |
| Iteration | `each` |
