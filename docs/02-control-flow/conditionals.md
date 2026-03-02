# Conditionals

---

## Overview

Opal provides a straightforward conditional system with `if`/`elsif`/`else` blocks, a suffix form for single expressions, and a ternary-style inline form. All conditional blocks are terminated with `end`.

---

## 1. If / Else

```opal
# if / else
if a == b
  c = 1
else
  c = 2
end
```

## 2. If / Elsif / Else

```opal
# if / elsif / else
if score >= 90
  grade = "A"
elsif score >= 80
  grade = "B"
elsif score >= 70
  grade = "C"
else
  grade = "F"
end
```

## 3. Suffix Form (Single Expression)

```opal
# Suffix form (single expression)
print("even") if n % 2 == 0
```

## 4. Ternary-Style Inline

```opal
# Ternary-style inline
status = if active then "on" else "off" end
```
