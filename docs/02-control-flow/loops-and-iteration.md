# Loops & Iteration

---

## Overview

Opal supports `while` loops for condition-based repetition and `for-in` loops for iterating over collections and ranges. Loops can be controlled with `break` to exit early and `next` to skip to the next iteration. Index tracking is available via the `with_index()` method.

---

## 1. While

```opal
# while
while count < 10
  count += 1
end
```

## 2. For-In

```opal
# for-in
for item in [1, 2, 3]
  print(item)
end

for char in "a".."z"
  print(char)
end
```

## 3. Loop with Index

```opal
# Loop with index
for item, index in ["a", "b", "c"].with_index()
  print(f"{index}: {item}")
end
```

## 4. Break and Next

```opal
# break and next (skip)
for n in 1..100
  next if n % 2 == 0
  break if n > 50
  print(n)
end
```
