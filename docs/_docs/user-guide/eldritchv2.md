---
title: Eldritch V2 Language Guide
permalink: /docs/user-guide/eldritchv2
---

# Eldritch V2

Eldritch V2 is a Starlark-like domain specific language used for scripting implant behaviors. It is designed to be familiar to Python users while remaining simple and safe.

## New Features

### Operators

*   **Floor Division**: `//` operator performs floor division (rounds towards negative infinity).
    ```python
    10 // 3  # 3
    -10 // 3 # -4
    ```
*   **Augmented Assignment**: Supports `+=`, `-=`, `*=`, `/=`, `//=`, `%=`.
    ```python
    x = 1
    x += 1
    ```

### Expressions

*   **Ternary If**: One-line conditional expression.
    ```python
    x = "yes" if condition else "no"
    ```
*   **String Formatting**: Use `%` operator for simple string formatting.
    ```python
    "Hello %s" % "World"
    ```

### Assignments

*   **Unpacking**: Assign multiple variables from a sequence.
    ```python
    a, b = 1, 2
    x, y = [3, 4]
    ```

### Modules

Eldritch V2 supports loading modules (libraries) which can expose functions. Modules can be accessed using dot notation.

```python
# Assuming 'file' module is registered
file.move("source.txt", "dest.txt")
```

## Language Reference

Eldritch V2 supports standard Python/Starlark features:

*   **Types**: `None`, `bool`, `int`, `string`, `bytes`, `list`, `tuple`, `dict`.
*   **Control Flow**: `if`, `elif`, `else`, `for`, `break`, `continue`, `pass`, `return`.
*   **Functions**: `def` with positional, keyword, `*args`, `**kwargs`, and default values. `lambda` expressions.
*   **Comprehensions**: List and Dictionary comprehensions.

## API Integration

Host applications can register custom modules using the Rust API:

```rust
use eldritchv2::{Interpreter, eldritch_module, Value};

let my_lib = eldritch_module! {
    name: "mylib",
    functions: {
        "greet" => my_greet_fn,
    }
};

let mut interpreter = Interpreter::new();
interpreter.register_module("mylib", my_lib);
```
