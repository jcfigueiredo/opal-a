# FFI (Foreign Function Interface)

---

## Overview

Opal provides a placeholder `extern` syntax for calling functions from external shared libraries. The exact calling convention and type mapping depend on the runtime implementation.

---

## 1. Extern Syntax

```opal
# Declare external functions from a C library
extern "libmath.so"
  def sin(x::Float64) -> Float64
  def cos(x::Float64) -> Float64
  def sqrt(x::Float64) -> Float64
end

# Use them like normal Opal functions
result = sin(3.14)
hypotenuse = sqrt(x ** 2 + y ** 2)
```

---

## 2. Rules

- `extern "library"` declares functions from an external shared library.
- Function signatures must be fully typed -- no inference across the FFI boundary.
- External functions are called like regular Opal functions once declared.
- The runtime determines: calling convention (C ABI, etc.), type mapping (Opal types to native types), library resolution (paths, linking).

---

## Summary

| Feature | Decision |
|---|---|
| Keyword | `extern "library"` -- declares external functions |
| Type safety | Full type annotations required on all FFI signatures |
| Usage | External functions are called like regular Opal functions |
| Status | Placeholder -- exact calling convention depends on runtime implementation |
