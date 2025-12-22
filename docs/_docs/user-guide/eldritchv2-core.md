---
title: Eldritch V2 core
tags:
 - User Guide
description: EldritchV2 Core User Guide
permalink: user-guide/eldritchv2-core
---
{% comment %} Generated from implants/lib/eldritchv2/eldritch-core/build.rs {% endcomment %}

# Overview

Eldritch V2 is a Starlark-like domain specific language used for scripting implant behaviors. It is designed to be familiar to Python users while remaining simple, safe, and easily embeddable.

## Quick Start

You can try the language in the [interactive REPL demo](/assets/eldritch-repl/index.html).

### Hello World

```python
print("Hello, World!")
```

### Variables and Types

Eldritch V2 is dynamically typed.

```python
x = 10              # int
y = 3.14            # float
name = "Eldritch"   # string
is_active = True    # bool
data = b"\x00\x01"  # bytes
items = [1, 2, 3]   # list
config = {"a": 1}   # dict
point = (10, 20)    # tuple
unique = {1, 2, 3}  # set
```

## Language Reference

### Control Flow

**If Statements**

```python
if x > 10:
    print("Big")
elif x == 10:
    print("Ten")
else:
    print("Small")
```

**Loops**

```python
# For loop
for i in range(5):
    print(i)

# While loop
while x > 0:
    x -= 1
    if x == 5:
        break
```

**Ternary Operator**

```python
status = "Active" if is_running else "Inactive"
```

### Functions

Functions are defined using `def`. They support positional arguments, keyword arguments, default values, `*args`, and `**kwargs`.

```python
def greet(name, greeting="Hello"):
    return "%s, %s!" % (greeting, name)

print(greet("World"))
```

### Modules

Eldritch V2 supports loading modules. Standard library modules (like `file`, `sys`) are available globally or can be imported if configured. In the standard environment, they are pre-loaded global objects.

```python
file.list(".")
```


## Built-in Functions

Eldritch V2 provides a rich set of built-in functions available in the global scope.


### Core

#### `print(*args)`: Prints objects to the standard output.

Converts each argument to a string and prints it to the standard output,
separated by spaces.

#### `pprint(object, indent=2)`: Pretty-prints an object.

Prints the object in a formatted, readable way with indentation.
Useful for debugging complex data structures like dictionaries and lists.

#### `len(s)`: Returns the length of an object.

The argument may be a sequence (such as a string, bytes, tuple, list, or range)
or a collection (such as a dictionary or set).

#### `type(object)`: Returns the type of the object.

Returns a string representation of the type of the object.

#### `dir([object])`: Returns a list of valid attributes for the object.

Without arguments, return the list of names in the current local scope.
With an argument, attempt to return a list of valid attributes for that object.

#### `libs()`: Lists all registered libraries.

Returns a list of strings representing the names of all libraries loaded
in the current environment scope chain.

#### `fail(message)`: Aborts execution with an error message.

**Parameters**
- `message` (Any): The message to include in the error.

#### `assert(condition)`: Aborts if the condition is false.

**Parameters**
- `condition` (Any): The condition to check.

#### `assert_eq(a, b)`: Aborts if `a` is not equal to `b`.

**Parameters**
- `a` (Any): Left operand.
- `b` (Any): Right operand.

### Type Constructors & Conversion

#### `bool(x)`: Converts a value to a Boolean.

Returns True when the argument x is true, False otherwise.

#### `int(x)`: Converts a number or string to an integer.

If x is a number, return x.__int__(). For floating point numbers, this truncates towards zero.
If x is not a number or if base is given, then x must be a string, bytes, or bytearray instance representing an integer literal in the given base.

#### `float(x)`: Converts a number or string to a floating point number.

**Parameters**
- `x` (Int | Float | String): The value to convert.

#### `str(object)`: Returns a string containing a nicely printable representation of an object.

**Parameters**
- `object` (Any): The object to convert.

#### `bytes(source)`: Creates a bytes object.

If source is an integer, the array will have that size and will be initialized with null bytes.
If source is a string, it will be converted using UTF-8 encoding.
If source is an iterable, it must be an iterable of integers in the range 0 <= x < 256, which are used as the initial contents of the array.

#### `list([iterable])`: Creates a list.

If no argument is given, the constructor creates a new empty list.
The argument must be an iterable if specified.

#### `dict(**kwargs)` or `dict(iterable, **kwargs)`: Creates a dictionary.

**Parameters**
- `iterable` (Iterable): An iterable of key-value pairs (tuples/lists of length 2).
- `**kwargs` (Any): Keyword arguments to add to the dictionary.

#### `set([iterable])`: Creates a set.

If no argument is given, the constructor creates a new empty set.
The argument must be an iterable if specified.

#### `tuple([iterable])`: Creates a tuple.

If no argument is given, the constructor creates a new empty tuple.
The argument must be an iterable if specified.

### Math & Logic

#### `abs(x)`: Returns the absolute value of a number.

**Parameters**
- `x` (Int | Float): The number.

#### `max(iterable)` or `max(arg1, arg2, *args)`: Returns the largest item.

**Parameters**
- `iterable` (Iterable): An iterable to search.
- `arg1, arg2, *args` (Any): Two or more arguments to compare.

#### `min(iterable)` or `min(arg1, arg2, *args)`: Returns the smallest item.

**Parameters**
- `iterable` (Iterable): An iterable to search.
- `arg1, arg2, *args` (Any): Two or more arguments to compare.

#### `range(stop)` or `range(start, stop[, step])`: Returns a sequence of numbers.

**Parameters**
- `start` (Int): The start value (inclusive). Defaults to 0.
- `stop` (Int): The stop value (exclusive).
- `step` (Int): The step size. Defaults to 1.

### Iteration

#### `all(iterable)`: Returns True if all elements of the iterable are true.

**Parameters**
- `iterable` (Iterable): The iterable to check.

#### `any(iterable)`: Returns True if any element of the iterable is true.

**Parameters**
- `iterable` (Iterable): The iterable to check.

#### `enumerate(iterable, start=0)`: Returns an enumerate object.

Returns a list of tuples containing (index, value) pairs.

**Parameters**
- `iterable` (Iterable): The sequence to enumerate.
- `start` (Int): The starting index. Defaults to 0.

#### `reversed(seq)`: Returns a reverse iterator.

Returns a list of the elements of the sequence in reverse order.

**Parameters**
- `seq` (Sequence): The sequence to reverse (List, Tuple, String).

####   **`sorted`**

#### `zip(*iterables)`: Returns an iterator of tuples.

Returns a list of tuples, where the i-th tuple contains the i-th element from each of the argument sequences or iterables.
The returned list is truncated to the length of the shortest argument sequence.

**Parameters**
- `*iterables` (Iterable): Iterables to zip together.

