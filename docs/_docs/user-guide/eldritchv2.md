---
title: Eldritch V2 Language Guide
permalink: /user-guide/eldritchv2
---

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
import file
# or just use the global object directly if available
file.list(".")
```

## Built-in Functions

Eldritch V2 provides a rich set of built-in functions available in the global scope.

### Core

*   **`print(*args)`: Prints objects to the standard output.`**
    
    Converts each argument to a string and prints it to the standard output,
    separated by spaces.

*   **`pprint(object, indent=2)`: Pretty-prints an object.`**
    
    Prints the object in a formatted, readable way with indentation.
    Useful for debugging complex data structures like dictionaries and lists.

*   **`len(s)`: Returns the length of an object.`**
    
    The argument may be a sequence (such as a string, bytes, tuple, list, or range)
    or a collection (such as a dictionary or set).

*   **`type(object)`: Returns the type of the object.`**
    
    Returns a string representation of the type of the object.

*   **`dir([object])`: Returns a list of valid attributes for the object.`**
    
    Without arguments, return the list of names in the current local scope.
    With an argument, attempt to return a list of valid attributes for that object.

*   **`libs()`: Lists all registered libraries.`**
    
    Returns a list of strings representing the names of all libraries loaded
    in the current environment scope chain.

*   **`fail(message)`: Aborts execution with an error message.`**
    
    **Parameters**
    - `message` (Any): The message to include in the error.

*   **`assert(condition)`: Aborts if the condition is false.`**
    
    **Parameters**
    - `condition` (Any): The condition to check.

*   **`assert_eq(a, b)`: Aborts if `a` is not equal to `b`.`**
    
    **Parameters**
    - `a` (Any): Left operand.
    - `b` (Any): Right operand.

### Type Constructors & Conversion

*   **`bool(x)`: Converts a value to a Boolean.`**
    
    Returns True when the argument x is true, False otherwise.

*   **`int(x)`: Converts a number or string to an integer.`**
    
    If x is a number, return x.__int__(). For floating point numbers, this truncates towards zero.
    If x is not a number or if base is given, then x must be a string, bytes, or bytearray instance representing an integer literal in the given base.

*   **`float(x)`: Converts a number or string to a floating point number.`**
    
    **Parameters**
    - `x` (Int | Float | String): The value to convert.

*   **`str(object)`: Returns a string containing a nicely printable representation of an object.`**
    
    **Parameters**
    - `object` (Any): The object to convert.

*   **`bytes(source)`: Creates a bytes object.`**
    
    If source is an integer, the array will have that size and will be initialized with null bytes.
    If source is a string, it will be converted using UTF-8 encoding.
    If source is an iterable, it must be an iterable of integers in the range 0 <= x < 256, which are used as the initial contents of the array.

*   **`list([iterable])`: Creates a list.`**
    
    If no argument is given, the constructor creates a new empty list.
    The argument must be an iterable if specified.

*   **`dict(**kwargs)` or `dict(iterable, **kwargs)`: Creates a dictionary.`**
    
    **Parameters**
    - `iterable` (Iterable): An iterable of key-value pairs (tuples/lists of length 2).
    - `**kwargs` (Any): Keyword arguments to add to the dictionary.

*   **`set([iterable])`: Creates a set.`**
    
    If no argument is given, the constructor creates a new empty set.
    The argument must be an iterable if specified.

*   **`tuple([iterable])`: Creates a tuple.`**
    
    If no argument is given, the constructor creates a new empty tuple.
    The argument must be an iterable if specified.

### Math & Logic

*   **`abs(x)`: Returns the absolute value of a number.`**
    
    **Parameters**
    - `x` (Int | Float): The number.

*   **`max(iterable)` or `max(arg1, arg2, *args)`: Returns the largest item.`**
    
    **Parameters**
    - `iterable` (Iterable): An iterable to search.
    - `arg1, arg2, *args` (Any): Two or more arguments to compare.

*   **`min(iterable)` or `min(arg1, arg2, *args)`: Returns the smallest item.`**
    
    **Parameters**
    - `iterable` (Iterable): An iterable to search.
    - `arg1, arg2, *args` (Any): Two or more arguments to compare.

*   **`range(stop)` or `range(start, stop[, step])`: Returns a sequence of numbers.`**
    
    **Parameters**
    - `start` (Int): The start value (inclusive). Defaults to 0.
    - `stop` (Int): The stop value (exclusive).
    - `step` (Int): The step size. Defaults to 1.

### Iteration

*   **`all(iterable)`: Returns True if all elements of the iterable are true.`**
    
    **Parameters**
    - `iterable` (Iterable): The iterable to check.

*   **`any(iterable)`: Returns True if any element of the iterable is true.`**
    
    **Parameters**
    - `iterable` (Iterable): The iterable to check.

*   **`enumerate(iterable, start=0)`: Returns an enumerate object.`**
    
    Returns a list of tuples containing (index, value) pairs.
    
    **Parameters**
    - `iterable` (Iterable): The sequence to enumerate.
    - `start` (Int): The starting index. Defaults to 0.

*   **`reversed(seq)`: Returns a reverse iterator.`**
    
    Returns a list of the elements of the sequence in reverse order.
    
    **Parameters**
    - `seq` (Sequence): The sequence to reverse (List, Tuple, String).

*   **`sorted`**

*   **`zip(*iterables)`: Returns an iterator of tuples.`**
    
    Returns a list of tuples, where the i-th tuple contains the i-th element from each of the argument sequences or iterables.
    The returned list is truncated to the length of the shortest argument sequence.
    
    **Parameters**
    - `*iterables` (Iterable): Iterables to zip together.


---

## Type Methods

Methods available on built-in types.

### List methods

*   **`list.append`**
    Appends an item to the end of the list.
    **Parameters**
*   **`- `x` (Any): The item to append.`**
    **Returns**
    - `None`
*   **`list.extend`**
    Extends the list by appending all the items from the iterable.
    **Parameters**
*   **`- `iterable` (Iterable): The elements to add.`**
    **Returns**
    - `None`
*   **`list.insert`**
    Inserts an item at a given position.
    **Parameters**
*   **`- `i` (Int): The index of the element before which to insert.`**
    - `x` (Any): The element to insert.
    **Returns**
    - `None`
*   **`list.remove`**
    Removes the first item from the list whose value is equal to x.
    Raises ValueError if there is no such item.
    **Parameters**
*   **`- `x` (Any): The item to remove.`**
    **Returns**
    - `None`
*   **`list.pop`**
    Removes the item at the given position in the list, and returns it.
    If no index is specified, removes and returns the last item in the list.
    **Parameters**
*   **`- `i` (Option<Int>): The index of the item to remove. Defaults to -1.`**
    **Returns**
*   **`- `Any`: The removed item.`**
*   **`list.clear`**
    Removes all items from the list.
    **Returns**
    - `None`
*   **`list.index`**
    Returns the zero-based index in the list of the first item whose value is equal to x.
    Raises ValueError if there is no such item.
    **Parameters**
*   **`- `x` (Any): The item to search for.`**
    - `start` (Option<Int>): Optional start index.
    - `end` (Option<Int>): Optional end index.
    **Returns**
*   **`- `Int`: The index of the item.`**
*   **`list.count`**
    Returns the number of times x appears in the list.
    **Parameters**
*   **`- `x` (Any): The item to count.`**
    **Returns**
*   **`- `Int`: The count.`**
*   **`list.sort`**
    Sorts the items of the list in place.
    **Parameters**
*   **`- `key` (Option<Function>): A function of one argument that is used to extract a comparison key from each list element.`**
    - `reverse` (Option<Bool>): If set to True, then the list elements are sorted as if each comparison were reversed.
    **Returns**
    - `None`
*   **`list.reverse`**
    Reverses the elements of the list in place.
    **Returns**
    - `None`
*   **`list.copy`**
    Returns a shallow copy of the list.
    **Returns**
*   **`- `List`: A shallow copy of the list.`**
### Dictionary methods

*   **`dict.clear`**
    Removes all items from the dictionary.
    **Returns**
    - `None`
*   **`dict.copy`**
    Returns a shallow copy of the dictionary.
    **Returns**
*   **`- `Dict`: A shallow copy.`**
*   **`dict.fromkeys`**
    Create a new dictionary with keys from iterable and values set to value.
    **Parameters**
*   **`- `iterable` (Iterable): The keys.`**
    - `value` (Any): The value to set. Defaults to None.
    **Returns**
*   **`- `Dict`: The new dictionary.`**
*   **`dict.get`**
    Return the value for key if key is in the dictionary, else default.
    **Parameters**
*   **`- `key` (Any): The key to search for.`**
    - `default` (Any): The value to return if key is not found. Defaults to None.
    **Returns**
*   **`- `Any`: The value or default.`**
*   **`dict.items`**
    Return a new view of the dictionary's items ((key, value) pairs).
    **Returns**
*   **`- `List<Tuple>`: A list of (key, value) tuples.`**
*   **`dict.keys`**
    Return a new view of the dictionary's keys.
    **Returns**
*   **`- `List`: A list of keys.`**
*   **`dict.pop`**
    Remove the key from the dictionary and return its value.
    If key is not found, default is returned if given, otherwise KeyError is raised.
    **Parameters**
*   **`- `key` (Any): The key to remove.`**
    - `default` (Option<Any>): The value to return if key is not found.
    **Returns**
*   **`- `Any`: The removed value.`**
*   **`dict.popitem`**
    Remove and return a (key, value) pair from the dictionary.
    Pairs are returned in LIFO order.
    **Returns**
*   **`- `Tuple`: A (key, value) pair.`**
*   **`dict.setdefault`**
    If key is in the dictionary, return its value.
    If not, insert key with a value of default and return default.
    **Parameters**
*   **`- `key` (Any): The key.`**
    - `default` (Any): The default value. Defaults to None.
    **Returns**
*   **`- `Any`: The value.`**
*   **`dict.update`**
    Update the dictionary with the key/value pairs from other, overwriting existing keys.
    **Parameters**
*   **`- `other` (Dict | Iterable): The dictionary or iterable of pairs to update from.`**
    **Returns**
    - `None`
*   **`dict.values`**
    Return a new view of the dictionary's values.
    **Returns**
*   **`- `List`: A list of values.`**
### Set methods

*   **`set.add`**
    Adds an element to the set.
    **Parameters**
*   **`- `elem` (Any): The element to add.`**
    **Returns**
    - `None`
*   **`set.clear`**
    Removes all elements from the set.
    **Returns**
    - `None`
*   **`set.copy`**
    Returns a shallow copy of the set.
    **Returns**
*   **`- `Set`: A shallow copy.`**
*   **`set.difference`**
    Return the difference of two or more sets as a new set.
    (i.e. all elements that are in this set but not the others.)
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
*   **`- `Set`: The difference set.`**
*   **`set.difference_update`**
    Remove all elements of another set from this set.
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
    - `None`
*   **`set.discard`**
    Remove an element from a set if it is a member.
    If the element is not a member, do nothing.
    **Parameters**
*   **`- `elem` (Any): The element to remove.`**
    **Returns**
    - `None`
*   **`set.intersection`**
    Return the intersection of two or more sets as a new set.
    (i.e. elements that are common to all of the sets.)
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
*   **`- `Set`: The intersection set.`**
*   **`set.intersection_update`**
    Update the set with the intersection of itself and another.
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
    - `None`
*   **`set.isdisjoint`**
    Return True if two sets have a null intersection.
    **Parameters**
*   **`- `other` (Iterable): Another set/iterable.`**
    **Returns**
    - `Bool`
*   **`set.issubset`**
    Report whether another set contains this set.
    **Parameters**
*   **`- `other` (Iterable): Another set/iterable.`**
    **Returns**
    - `Bool`
*   **`set.issuperset`**
    Report whether this set contains another set.
    **Parameters**
*   **`- `other` (Iterable): Another set/iterable.`**
    **Returns**
    - `Bool`
*   **`set.pop`**
    Remove and return an arbitrary set element.
    Raises KeyError if the set is empty.
    **Returns**
*   **`- `Any`: The removed element.`**
*   **`set.remove`**
    Remove an element from a set; it must be a member.
    If the element is not a member, raise a KeyError.
    **Parameters**
*   **`- `elem` (Any): The element to remove.`**
    **Returns**
    - `None`
*   **`set.symmetric_difference`**
    Return the symmetric difference of two sets as a new set.
    (i.e. elements that are in either of the sets, but not both.)
    **Parameters**
*   **`- `other` (Iterable): Another set/iterable.`**
    **Returns**
*   **`- `Set`: The symmetric difference.`**
*   **`set.symmetric_difference_update`**
    Update a set with the symmetric difference of itself and another.
    **Parameters**
*   **`- `other` (Iterable): Another set/iterable.`**
    **Returns**
    - `None`
*   **`set.union`**
    Return the union of sets as a new set.
    (i.e. all elements that are in either set.)
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
*   **`- `Set`: The union.`**
*   **`set.update`**
    Update a set with the union of itself and others.
    **Parameters**
*   **`- `*others` (Iterable): Other sets/iterables.`**
    **Returns**
    - `None`
### String methods

*   **`str.capitalize`**
    Return a copy of the string with its first character capitalized and the rest lowercased.
    **Returns**
    - `String`
*   **`str.casefold`**
    Return a casefolded copy of the string. Casefolded strings may be used for caseless matching.
    **Returns**
    - `String`
*   **`str.center`**
    Return centered in a string of length width.
    Padding is done using the specified fillchar (default is an ASCII space).
    **Parameters**
*   **`- `width` (Int): The total width.`**
    - `fillchar` (String): The padding character.
    **Returns**
    - `String`
*   **`str.count`**
    Return the number of non-overlapping occurrences of substring sub in the range [start, end].
    **Parameters**
*   **`- `sub` (String): The substring to count.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Int`
*   **`str.endswith`**
    Return True if the string ends with the specified suffix, otherwise return False.
    **Parameters**
*   **`- `suffix` (String | Tuple<String>): The suffix to check.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Bool`
*   **`str.find`**
    Return the lowest index in the string where substring sub is found within the slice s[start:end].
    Return -1 if sub is not found.
    **Parameters**
*   **`- `sub` (String): The substring to find.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Int`
*   **`str.format`**
    Perform a string formatting operation.
    The string on which this method is called can contain literal text or replacement fields delimited by braces {}.
    **Parameters**
*   **`- `*args` (Any): Positional arguments.`**
    - `**kwargs` (Any): Keyword arguments.
    **Returns**
    - `String`
*   **`str.index`**
    Like find(), but raise ValueError when the substring is not found.
    **Parameters**
*   **`- `sub` (String): The substring to find.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Int`
*   **`str.isalnum`**
    Return True if all characters in the string are alphanumeric and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isalpha`**
    Return True if all characters in the string are alphabetic and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isascii`**
    Return True if all characters in the string are ASCII.
    **Returns**
    - `Bool`
*   **`str.isdecimal`**
    Return True if all characters in the string are decimal characters and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isdigit`**
    Return True if all characters in the string are digits and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isidentifier`**
    Return True if the string is a valid identifier.
    **Returns**
    - `Bool`
*   **`str.islower`**
    Return True if all cased characters in the string are lowercase and there is at least one cased character.
    **Returns**
    - `Bool`
*   **`str.isnumeric`**
    Return True if all characters in the string are numeric characters and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isprintable`**
    Return True if all characters in the string are printable or the string is empty.
    **Returns**
    - `Bool`
*   **`str.isspace`**
    Return True if all characters in the string are whitespace and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.istitle`**
    Return True if the string is a titlecased string and there is at least one character.
    **Returns**
    - `Bool`
*   **`str.isupper`**
    Return True if all cased characters in the string are uppercase and there is at least one cased character.
    **Returns**
    - `Bool`
*   **`str.join`**
    Return a string which is the concatenation of the strings in iterable.
    The separator between elements is the string providing this method.
    **Parameters**
*   **`- `iterable` (Iterable): The strings to join.`**
    **Returns**
    - `String`
*   **`str.ljust`**
    Return the string left justified in a string of length width.
    Padding is done using the specified fillchar (default is an ASCII space).
    **Parameters**
*   **`- `width` (Int): The total width.`**
    - `fillchar` (String): The padding character.
    **Returns**
    - `String`
*   **`str.lower`**
    Return a copy of the string with all cased characters converted to lowercase.
    **Returns**
    - `String`
*   **`str.lstrip`**
    Return a copy of the string with leading whitespace removed.
    If chars is given and not None, remove characters in chars instead.
    **Parameters**
*   **`- `chars` (Option<String>): The characters to remove.`**
    **Returns**
    - `String`
*   **`str.partition`**
    Split the string at the first occurrence of sep, and return a 3-tuple containing the part before the separator, the separator itself, and the part after the separator.
    If the separator is not found, return a 3-tuple containing the string itself, followed by two empty strings.
    **Parameters**
*   **`- `sep` (String): The separator.`**
    **Returns**
    - `Tuple`
*   **`str.removeprefix`**
    Return a str with the given prefix string removed if present.
    If the string starts with the prefix string, return string[len(prefix):]. Otherwise, return a copy of the original string.
    **Parameters**
*   **`- `prefix` (String): The prefix to remove.`**
    **Returns**
    - `String`
*   **`str.removesuffix`**
    Return a str with the given suffix string removed if present.
    If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)]. Otherwise, return a copy of the original string.
    **Parameters**
*   **`- `suffix` (String): The suffix to remove.`**
    **Returns**
    - `String`
*   **`str.replace`**
    Return a copy of the string with all occurrences of substring old replaced by new.
    If the optional argument count is given, only the first count occurrences are replaced.
    **Parameters**
*   **`- `old` (String): The substring to replace.`**
    - `new` (String): The replacement string.
    - `count` (Option<Int>): The max number of replacements.
    **Returns**
    - `String`
*   **`str.rfind`**
    Return the highest index in the string where substring sub is found.
    Return -1 if sub is not found.
    **Parameters**
*   **`- `sub` (String): The substring to find.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Int`
*   **`str.rindex`**
    Like rfind() but raises ValueError when the substring is not found.
    **Parameters**
*   **`- `sub` (String): The substring to find.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Int`
*   **`str.rjust`**
    Return the string right justified in a string of length width.
    Padding is done using the specified fillchar (default is an ASCII space).
    **Parameters**
*   **`- `width` (Int): The total width.`**
    - `fillchar` (String): The padding character.
    **Returns**
    - `String`
*   **`str.rpartition`**
    Split the string at the last occurrence of sep, and return a 3-tuple containing the part before the separator, the separator itself, and the part after the separator.
    If the separator is not found, return a 3-tuple containing two empty strings, followed by the string itself.
    **Parameters**
*   **`- `sep` (String): The separator.`**
    **Returns**
    - `Tuple`
*   **`str.rsplit`**
    Return a list of the words in the string, using sep as the delimiter string.
    **Parameters**
*   **`- `sep` (Option<String>): The delimiter.`**
    - `maxsplit` (Option<Int>): The max number of splits.
    **Returns**
    - `List<String>`
*   **`str.rstrip`**
    Return a copy of the string with trailing whitespace removed.
    If chars is given and not None, remove characters in chars instead.
    **Parameters**
*   **`- `chars` (Option<String>): The characters to remove.`**
    **Returns**
    - `String`
*   **`str.split`**
    Return a list of the words in the string, using sep as the delimiter string.
    **Parameters**
*   **`- `sep` (Option<String>): The delimiter.`**
    - `maxsplit` (Option<Int>): The max number of splits.
    **Returns**
    - `List<String>`
*   **`str.splitlines`**
    Return a list of the lines in the string, breaking at line boundaries.
    Line breaks are not included in the resulting list unless keepends is given and true.
    **Parameters**
*   **`- `keepends` (Bool): Whether to keep line breaks.`**
    **Returns**
    - `List<String>`
*   **`str.startswith`**
    Return True if string starts with the specified prefix, otherwise return False.
    **Parameters**
*   **`- `prefix` (String | Tuple<String>): The prefix to check.`**
    - `start` (Option<Int>): The start index.
    - `end` (Option<Int>): The end index.
    **Returns**
    - `Bool`
*   **`str.strip`**
    Return a copy of the string with the leading and trailing whitespace removed.
    If chars is given and not None, remove characters in chars instead.
    **Parameters**
*   **`- `chars` (Option<String>): The characters to remove.`**
    **Returns**
    - `String`
*   **`str.swapcase`**
    Return a copy of the string with uppercase characters converted to lowercase and vice versa.
    **Returns**
    - `String`
*   **`str.title`**
    Return a titlecased version of the string where words start with an uppercase character and the remaining characters are lowercase.
    **Returns**
    - `String`
*   **`str.upper`**
    Return a copy of the string with all cased characters converted to uppercase.
    **Returns**
    - `String`
*   **`str.zfill`**
    Return a copy of the string left filled with ASCII '0' digits to make a string of length width.
    **Parameters**
*   **`- `width` (Int): The total width.`**
    **Returns**
    - `String`
*   **`str.codepoints`**
    Returns a list of the integer codepoints of the string.
    **Returns**
    - `List<Int>`
*   **`str.elems`**
    Returns a list of single-character strings containing the characters of the string.
    **Returns**
    - `List<String>`

---

## Standard Library

The standard library provides powerful capabilities for interacting with the host system.

### Agent

The `agent` library provides capabilities for interacting with the agent's internal state, configuration, and task management.

It allows you to:
- Modify agent configuration (callback intervals, transports).
- Manage background tasks.
- Report data back to the C2 server (though the `report` library is often preferred for high-level reporting).
- Control agent execution (termination).


*   **`agent._terminate_this_process_clowntown`**
    **DANGER**: Terminates the agent process immediately.
    
    This method calls `std::process::exit(0)`, effectively killing the agent.
    Use with extreme caution.
    
    **Returns**
    - `None` (Does not return as the process exits).
    
    **Errors**
    - This function is unlikely to return an error, as it terminates the process.

*   **`agent.get_config`**
    Returns the current configuration of the agent as a dictionary.
    
    **Returns**
    - `Dict<String, Value>`: A dictionary containing configuration keys and values.
    
    **Errors**
    - Returns an error string if the configuration cannot be retrieved or is not implemented.

*   **`agent.fetch_asset`**
    Fetches an asset (file) from the C2 server by name.
    
    This method requests the asset content from the server.
    
    **Parameters**
    - `name` (`str`): The name of the asset to fetch.
    
    **Returns**
    - `Bytes`: The content of the asset as a byte array.
    
    **Errors**
    - Returns an error string if the asset cannot be fetched or communication fails.

*   **`agent.report_credential`**
    Reports a captured credential to the C2 server.
    
    **Parameters**
    - `credential` (`Credential`): The credential object to report.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the reporting fails.

*   **`agent.report_file`**
    Reports a file (chunk) to the C2 server.
    
    This is typically used internally by `report.file`.
    
    **Parameters**
    - `file` (`File`): The file chunk wrapper to report.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the reporting fails.

*   **`agent.report_process_list`**
    Reports a list of processes to the C2 server.
    
    This is typically used internally by `report.process_list`.
    
    **Parameters**
    - `list` (`ProcessList`): The process list wrapper to report.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the reporting fails.

*   **`agent.report_task_output`**
    Reports the output of a task to the C2 server.
    
    This is used to send stdout/stderr or errors back to the controller.
    
    **Parameters**
    - `output` (`str`): The standard output content.
    - `error` (`Option<str>`): Optional error message.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the reporting fails.

*   **`agent.reverse_shell`**
    Initiates a reverse shell session.
    
    This starts a reverse shell based on the agent's capabilities (e.g., PTY or raw).
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the reverse shell cannot be started.

*   **`agent.claim_tasks`**
    Manually triggers a check-in to claim pending tasks from the C2 server.
    
    **Returns**
    - `List<Task>`: A list of tasks retrieved from the server.
    
    **Errors**
    - Returns an error string if the check-in fails.

*   **`agent.get_transport`**
    Returns the name of the currently active transport.
    
    **Returns**
    - `str`: The name of the transport (e.g., "http", "grpc").
    
    **Errors**
    - Returns an error string if the transport cannot be identified.

*   **`agent.list_transports`**
    Returns a list of available transport names.
    
    **Returns**
    - `List<str>`: A list of transport names.
    
    **Errors**
    - Returns an error string if the list cannot be retrieved.

*   **`agent.get_callback_interval`**
    Returns the current callback interval in seconds.
    
    **Returns**
    - `int`: The interval in seconds.
    
    **Errors**
    - Returns an error string if the interval cannot be retrieved.

*   **`agent.set_callback_interval`**
    Sets the callback interval for the agent.
    
    This configuration change is typically transient and may not persist across reboots.
    
    **Parameters**
    - `interval` (`int`): The new interval in seconds.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the interval cannot be set.

*   **`agent.set_active_callback_uri`**
    Sets the active callback URI for the agent.
    
    **Parameters**
    - `uri` (`str`): The new URI to callback to
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the active callback uri cannot be set.

*   **`agent.list_tasks`**
    Lists the currently running or queued background tasks on the agent.
    
    **Returns**
    - `List<Task>`: A list of task objects.
    
    **Errors**
    - Returns an error string if the task list cannot be retrieved.

*   **`agent.stop_task`**
    Stops a specific background task by its ID.
    
    **Parameters**
    - `task_id` (`int`): The ID of the task to stop.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the task cannot be stopped or does not exist.

### Assets

The `assets` library provides access to files embedded directly within the agent binary.

This allows you to:
- Deploy tools or payloads without downloading them from the network.
- Read embedded configuration or scripts.
- List available embedded assets.

**Note**: Asset paths are typically relative to the embedding root (e.g., `sliver/agent-x64`).


*   **`assets.read_binary`**
    Reads the content of an embedded asset as a list of bytes.
    
    **Parameters**
    - `name` (`str`): The name/path of the asset to read.
    
    **Returns**
    - `List<int>`: The asset content as a list of bytes (u8).
    
    **Errors**
    - Returns an error string if the asset does not exist.

*   **`assets.read`**
    Reads the content of an embedded asset as a UTF-8 string.
    
    **Parameters**
    - `name` (`str`): The name/path of the asset to read.
    
    **Returns**
    - `str`: The asset content as a string.
    
    **Errors**
    - Returns an error string if the asset does not exist or contains invalid UTF-8 data.

*   **`assets.copy`**
    Copies an embedded asset to a destination path on the disk.
    
    **Parameters**
    - `src` (`str`): The name/path of the source asset.
    - `dest` (`str`): The destination file path on the local system.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the asset does not exist or the file cannot be written (e.g., permission denied).

*   **`assets.list`**
    Returns a list of all available asset names.
    
    **Returns**
    - `List<str>`: A list of asset names available in the agent.
    
    **Errors**
    - Returns an error string if the asset list cannot be retrieved.

### Crypto

The `crypto` library provides cryptographic primitives, hashing, encoding, and JSON handling utilities.

It supports:
- AES encryption and decryption.
- Hashing (MD5, SHA1, SHA256) for data and files.
- Base64 encoding and decoding.
- JSON serialization and deserialization.


*   **`crypto.aes_decrypt`**
    Decrypts data using AES (CBC mode).
    
    **Parameters**
    - `key` (`Bytes`): The decryption key (must be 16, 24, or 32 bytes).
    - `iv` (`Bytes`): The initialization vector (must be 16 bytes).
    - `data` (`Bytes`): The encrypted data to decrypt.
    
    **Returns**
    - `Bytes`: The decrypted data.
    
    **Errors**
    - Returns an error string if decryption fails (e.g., invalid padding, incorrect key length).

*   **`crypto.aes_encrypt`**
    Encrypts data using AES (CBC mode).
    
    **Parameters**
    - `key` (`Bytes`): The encryption key (must be 16, 24, or 32 bytes).
    - `iv` (`Bytes`): The initialization vector (must be 16 bytes).
    - `data` (`Bytes`): The data to encrypt.
    
    **Returns**
    - `Bytes`: The encrypted data.
    
    **Errors**
    - Returns an error string if encryption fails (e.g., incorrect key length).

*   **`crypto.aes_decrypt_file`**
    Decrypts a file using AES.
    
    **Parameters**
    - `src` (`str`): The source file path.
    - `dst` (`str`): The destination file path.
    - `key` (`str`): The decryption key.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if decryption fails or file operations fail.

*   **`crypto.aes_encrypt_file`**
    Encrypts a file using AES.
    
    **Parameters**
    - `src` (`str`): The source file path.
    - `dst` (`str`): The destination file path.
    - `key` (`str`): The encryption key.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if encryption fails or file operations fail.

*   **`crypto.md5`**
    Calculates the MD5 hash of the provided data.
    
    **Parameters**
    - `data` (`Bytes`): The input data.
    
    **Returns**
    - `str`: The hexadecimal representation of the hash.
    
    **Errors**
    - Returns an error string if hashing fails.

*   **`crypto.sha1`**
    Calculates the SHA1 hash of the provided data.
    
    **Parameters**
    - `data` (`Bytes`): The input data.
    
    **Returns**
    - `str`: The hexadecimal representation of the hash.
    
    **Errors**
    - Returns an error string if hashing fails.

*   **`crypto.sha256`**
    Calculates the SHA256 hash of the provided data.
    
    **Parameters**
    - `data` (`Bytes`): The input data.
    
    **Returns**
    - `str`: The hexadecimal representation of the hash.
    
    **Errors**
    - Returns an error string if hashing fails.

*   **`crypto.hash_file`**
    Calculates the hash of a file on disk.
    
    **Parameters**
    - `file` (`str`): The path to the file.
    - `algo` (`str`): The hashing algorithm to use ("MD5", "SHA1", "SHA256", "SHA512").
    
    **Returns**
    - `str`: The hexadecimal representation of the hash.
    
    **Errors**
    - Returns an error string if the file cannot be read or the algorithm is not supported.

*   **`crypto.encode_b64`**
    Encodes a string to Base64.
    
    **Parameters**
    - `content` (`str`): The string content to encode.
    - `encode_type` (`Option<str>`): The encoding variant. Valid options:
      - "STANDARD" (default)
      - "STANDARD_NO_PAD"
      - "URL_SAFE"
      - "URL_SAFE_NO_PAD"
    
    **Returns**
    - `str`: The Base64 encoded string.
    
    **Errors**
    - Returns an error string if the encoding type is invalid.

*   **`crypto.decode_b64`**
    Decodes a Base64 encoded string.
    
    **Parameters**
    - `content` (`str`): The Base64 string to decode.
    - `encode_type` (`Option<str>`): The decoding variant (matches encoding options).
      - "STANDARD" (default)
      - "STANDARD_NO_PAD"
      - "URL_SAFE"
      - "URL_SAFE_NO_PAD"
    
    **Returns**
    - `str`: The decoded string.
    
    **Errors**
    - Returns an error string if decoding fails or the variant is invalid.

*   **`crypto.is_json`**
    Checks if a string is valid JSON.
    
    **Parameters**
    - `content` (`str`): The string to check.
    
    **Returns**
    - `bool`: `True` if valid JSON, `False` otherwise.

*   **`crypto.from_json`**
    Parses a JSON string into an Eldritch value (Dict, List, etc.).
    
    **Parameters**
    - `content` (`str`): The JSON string.
    
    **Returns**
    - `Value`: The parsed value.
    
    **Errors**
    - Returns an error string if the JSON is invalid.

*   **`crypto.to_json`**
    Serializes an Eldritch value into a JSON string.
    
    **Parameters**
    - `content` (`Value`): The value to serialize.
    
    **Returns**
    - `str`: The JSON string representation.
    
    **Errors**
    - Returns an error string if serialization fails (e.g., circular references, unsupported types).

### File

The `file` library provides comprehensive filesystem operations.

It supports:
- reading and writing files (text and binary).
- file manipulation (copy, move, remove).
- directory operations (mkdir, list).
- compression and decompression (gzip).
- content searching and replacement.


*   **`file.append`**
    Appends content to a file.
    
    If the file does not exist, it will be created.
    
    **Parameters**
    - `path` (`str`): The path to the file.
    - `content` (`str`): The string content to append.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the file cannot be opened or written to.

*   **`file.compress`**
    Compresses a file or directory using GZIP.
    
    If `src` is a directory, it will be archived (tar) before compression.
    
    **Parameters**
    - `src` (`str`): The source file or directory path.
    - `dst` (`str`): The destination path for the compressed file (e.g., `archive.tar.gz`).
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the source doesn't exist or compression fails.

*   **`file.copy`**
    Copies a file from source to destination.
    
    If the destination exists, it will be overwritten.
    
    **Parameters**
    - `src` (`str`): The source file path.
    - `dst` (`str`): The destination file path.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the source doesn't exist or copy fails.

*   **`file.decompress`**
    Decompresses a GZIP file.
    
    If the file is a tar archive, it will be extracted to the destination directory.
    
    **Parameters**
    - `src` (`str`): The source compressed file path.
    - `dst` (`str`): The destination path (file or directory).
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if decompression fails.

*   **`file.exists`**
    Checks if a file or directory exists at the given path.
    
    **Parameters**
    - `path` (`str`): The path to check.
    
    **Returns**
    - `bool`: `True` if it exists, `False` otherwise.

*   **`file.follow`**
    Follows a file (tail -f) and executes a callback function for each new line.
    
    This is useful for monitoring logs.
    
    **Parameters**
    - `path` (`str`): The file path to follow.
    - `fn` (`function(str)`): A callback function that takes a string (the new line) as an argument.
    
    **Returns**
    - `None` (This function may block indefinitely or until interrupted).
    
    **Errors**
    - Returns an error string if the file cannot be opened.

*   **`file.is_dir`**
    Checks if the path exists and is a directory.
    
    **Parameters**
    - `path` (`str`): The path to check.
    
    **Returns**
    - `bool`: `True` if it is a directory, `False` otherwise.

*   **`file.is_file`**
    Checks if the path exists and is a file.
    
    **Parameters**
    - `path` (`str`): The path to check.
    
    **Returns**
    - `bool`: `True` if it is a file, `False` otherwise.

*   **`file.list`**
    Lists files and directories in the specified path.
    
    Supports globbing patterns (e.g., `/home/*/*.txt`).
    
    **Parameters**
    - `path` (`Option<str>`): The directory path or glob pattern. Defaults to current working directory.
    
    **Returns**
    - `List<Dict>`: A list of dictionaries containing file details:
      - `file_name` (`str`)
      - `absolute_path` (`str`)
      - `size` (`int`)
      - `owner` (`str`)
      - `group` (`str`)
      - `permissions` (`str`)
      - `modified` (`str`)
      - `type` (`str`: "File" or "Directory")
    
    **Errors**
    - Returns an error string if listing fails.

*   **`file.mkdir`**
    Creates a new directory.
    
    **Parameters**
    - `path` (`str`): The directory path to create.
    - `parent` (`Option<bool>`): If `True`, creates parent directories as needed (like `mkdir -p`). Defaults to `False`.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if creation fails.

*   **`file.move`**
    Moves or renames a file or directory.
    
    **Parameters**
    - `src` (`str`): The source path.
    - `dst` (`str`): The destination path.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the move fails.

*   **`file.parent_dir`**
    Returns the parent directory of the given path.
    
    **Parameters**
    - `path` (`str`): The file or directory path.
    
    **Returns**
    - `str`: The parent directory path.
    
    **Errors**
    - Returns an error string if the path is invalid or has no parent.

*   **`file.read`**
    Reads the entire content of a file as a string.
    
    Supports globbing; if multiple files match, reads the first one (or behavior may vary, usually reads specific file).
    *Note*: v1 docs say it errors if a directory matches.
    
    **Parameters**
    - `path` (`str`): The file path.
    
    **Returns**
    - `str`: The file content.
    
    **Errors**
    - Returns an error string if the file cannot be read or contains invalid UTF-8.

*   **`file.read_binary`**
    Reads the entire content of a file as binary data.
    
    **Parameters**
    - `path` (`str`): The file path.
    
    **Returns**
    - `List<int>`: The file content as a list of bytes (u8).
    
    **Errors**
    - Returns an error string if the file cannot be read.

*   **`file.remove`**
    Deletes a file or directory recursively.
    
    **Parameters**
    - `path` (`str`): The path to remove.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if removal fails.

*   **`file.replace`**
    Replaces the first occurrence of a regex pattern in a file with a replacement string.
    
    **Parameters**
    - `path` (`str`): The file path.
    - `pattern` (`str`): The regex pattern to match.
    - `value` (`str`): The replacement string.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the file cannot be modified or the regex is invalid.

*   **`file.replace_all`**
    Replaces all occurrences of a regex pattern in a file with a replacement string.
    
    **Parameters**
    - `path` (`str`): The file path.
    - `pattern` (`str`): The regex pattern to match.
    - `value` (`str`): The replacement string.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the file cannot be modified or the regex is invalid.

*   **`file.temp_file`**
    Creates a temporary file and returns its path.
    
    **Parameters**
    - `name` (`Option<str>`): Optional preferred filename. If None, a random name is generated.
    
    **Returns**
    - `str`: The absolute path to the temporary file.
    
    **Errors**
    - Returns an error string if creation fails.

*   **`file.template`**
    Renders a Jinja2 template file to a destination path.
    
    **Parameters**
    - `template_path` (`str`): Path to the source template file.
    - `dst` (`str`): Destination path for the rendered file.
    - `args` (`Dict<str, Value>`): Variables to substitute in the template.
    - `autoescape` (`bool`): Whether to enable HTML auto-escaping (OWASP recommendations).
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the template cannot be read, parsed, or written.

*   **`file.timestomp`**
    Timestomps a file.
    
    Modifies the timestamps (modified, access, creation) of a file.
    Can use a reference file or specific values.
    
    **Parameters**
    - `path` (`str`): The target file to modify.
    - `mtime` (`Option<Value>`): New modification time (Int epoch or String).
    - `atime` (`Option<Value>`): New access time (Int epoch or String).
    - `ctime` (`Option<Value>`): New creation time (Int epoch or String). Windows only.
    - `ref_file` (`Option<str>`): Path to a reference file to copy timestamps from.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the operation fails or input is invalid.

*   **`file.write`**
    Writes content to a file, overwriting it if it exists.
    
    **Parameters**
    - `path` (`str`): The file path.
    - `content` (`str`): The string content to write.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if writing fails.

*   **`file.find`**
    Finds files matching specific criteria.
    
    **Parameters**
    - `path` (`str`): The base directory to start searching from.
    - `name` (`Option<str>`): Filter by filename (substring match).
    - `file_type` (`Option<str>`): Filter by type ("file" or "dir").
    - `permissions` (`Option<int>`): Filter by permissions (Unix octal e.g., 777, Windows readonly check).
    - `modified_time` (`Option<int>`): Filter by modification time (epoch seconds).
    - `create_time` (`Option<int>`): Filter by creation time (epoch seconds).
    
    **Returns**
    - `List<str>`: A list of matching file paths.
    
    **Errors**
    - Returns an error string if the search encounters issues.

### HTTP

The `http` library enables the agent to make HTTP requests.

It supports:
- GET and POST requests.
- File downloading.
- Custom headers.

**Note**: TLS validation behavior depends on the underlying agent configuration and may not be exposed per-request in this version of the library (unlike v1 which had `allow_insecure` arg).


*   **`http.download`**
    Downloads a file from a URL to a local path.
    
    **Parameters**
    - `url` (`str`): The URL to download from.
    - `path` (`str`): The local destination path.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the download fails.

*   **`http.get`**
    Performs an HTTP GET request.
    
    **Parameters**
    - `url` (`str`): The target URL.
    - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    
    **Returns**
    - `Dict`: A dictionary containing the response:
      - `status_code` (`int`): HTTP status code.
      - `body` (`Bytes`): The response body.
      - `headers` (`Dict<str, str>`): Response headers.
    
    **Errors**
    - Returns an error string if the request fails.

*   **`http.post`**
    Performs an HTTP POST request.
    
    **Parameters**
    - `url` (`str`): The target URL.
    - `body` (`Option<Bytes>`): The request body.
    - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    
    **Returns**
    - `Dict`: A dictionary containing the response:
      - `status_code` (`int`): HTTP status code.
      - `body` (`Bytes`): The response body.
      - `headers` (`Dict<str, str>`): Response headers.
    
    **Errors**
    - Returns an error string if the request fails.

### Pivot

The `pivot` library provides tools for lateral movement, scanning, and tunneling.

It supports:
- Reverse shells (PTY and REPL).
- SSH execution and file copy.
- Network scanning (ARP, Port).
- Traffic tunneling (Port forwarding, Bind proxy).
- Simple network interaction (Ncat).
- SMB execution (Stubbed/Proposed).


*   **`pivot.reverse_shell_pty`**
    Spawns a reverse shell with a PTY (Pseudo-Terminal) attached.
    
    This provides a full interactive shell experience over the agent's C2 channel.
    
    **Parameters**
    - `cmd` (`Option<str>`): The shell command to run (e.g., `/bin/bash`, `cmd.exe`). If `None`, defaults to system shell.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the shell cannot be spawned.

*   **`pivot.reverse_shell_repl`**
    Spawns a basic REPL-style reverse shell.
    
    Useful if PTY is not available.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if failure occurs.

*   **`pivot.ssh_exec`**
    Executes a command on a remote host via SSH.
    
    **Parameters**
    - `target` (`str`): The remote host IP or hostname.
    - `port` (`int`): The SSH port (usually 22).
    - `command` (`str`): The command to execute.
    - `username` (`str`): SSH username.
    - `password` (`Option<str>`): SSH password (optional).
    - `key` (`Option<str>`): SSH private key (optional).
    - `key_password` (`Option<str>`): Password for the private key (optional).
    - `timeout` (`Option<int>`): Connection timeout in seconds (optional).
    
    **Returns**
    - `Dict`: A dictionary containing command output:
      - `stdout` (`str`)
      - `stderr` (`str`)
      - `status` (`int`): Exit code.
    
    **Errors**
    - Returns an error string if connection fails.

*   **`pivot.ssh_copy`**
    Copies a file to a remote host via SSH (SCP/SFTP).
    
    **Parameters**
    - `target` (`str`): The remote host IP or hostname.
    - `port` (`int`): The SSH port.
    - `src` (`str`): Local source file path.
    - `dst` (`str`): Remote destination file path.
    - `username` (`str`): SSH username.
    - `password` (`Option<str>`): SSH password.
    - `key` (`Option<str>`): SSH private key.
    - `key_password` (`Option<str>`): Key password.
    - `timeout` (`Option<int>`): Connection timeout.
    
    **Returns**
    - `str`: "Success" message or error detail.
    
    **Errors**
    - Returns an error string if copy fails.

*   **`pivot.port_scan`**
    Scans TCP/UDP ports on target hosts.
    
    **Parameters**
    - `target_cidrs` (`List<str>`): List of CIDRs to scan (e.g., `["192.168.1.0/24"]`).
    - `ports` (`List<int>`): List of ports to scan.
    - `protocol` (`str`): "tcp" or "udp".
    - `timeout` (`int`): Timeout per port in seconds.
    - `fd_limit` (`Option<int>`): Maximum concurrent file descriptors/sockets (defaults to 64).
    
    **Returns**
    - `List<Dict>`: List of open ports/results.

*   **`pivot.arp_scan`**
    Performs an ARP scan to discover live hosts on the local network.
    
    **Parameters**
    - `target_cidrs` (`List<str>`): List of CIDRs to scan.
    
    **Returns**
    - `List<Dict>`: List of discovered hosts with IP, MAC, and Interface.

*   **`pivot.ncat`**
    Sends arbitrary data to a host via TCP or UDP and waits for a response.
    
    **Parameters**
    - `address` (`str`): Target address.
    - `port` (`int`): Target port.
    - `data` (`str`): Data to send.
    - `protocol` (`str`): "tcp" or "udp".
    
    **Returns**
    - `str`: The response data.

### Process

The `process` library allows interaction with system processes.

It supports:
- Listing running processes.
- Retrieving process details (info, name).
- Killing processes.
- Inspecting network connections (netstat).


*   **`process.info`**
    Returns detailed information about a specific process.
    
    **Parameters**
    - `pid` (`Option<int>`): The process ID to query. If `None`, returns info for the current agent process.
    
    **Returns**
    - `Dict`: Dictionary with process details (pid, name, cmd, exe, environ, cwd, memory_usage, user, etc.).
    
    **Errors**
    - Returns an error string if the process is not found or cannot be accessed.

*   **`process.kill`**
    Terminates a process by its ID.
    
    **Parameters**
    - `pid` (`int`): The process ID to kill.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the process cannot be killed (e.g., permission denied).

*   **`process.list`**
    Lists all currently running processes.
    
    **Returns**
    - `List<Dict>`: A list of process dictionaries containing `pid`, `ppid`, `name`, `path`, `username`, `command`, `cwd`, etc.
    
    **Errors**
    - Returns an error string if the process list cannot be retrieved.

*   **`process.name`**
    Returns the name of a process given its ID.
    
    **Parameters**
    - `pid` (`int`): The process ID.
    
    **Returns**
    - `str`: The process name.
    
    **Errors**
    - Returns an error string if the process is not found.

*   **`process.netstat`**
    Returns a list of active network connections (TCP/UDP/Unix).
    
    **Returns**
    - `List<Dict>`: A list of connection details including socket type, local/remote address/port, and associated PID.
    
    **Errors**
    - Returns an error string if network information cannot be retrieved.

### Random

The `random` library provides cryptographically secure random value generation.


*   **`random.bool`**
    Generates a random boolean value.
    
    **Returns**
    - `bool`: True or False.

*   **`random.bytes`**
    Generates a list of random bytes.
    
    **Parameters**
    - `len` (`int`): Number of bytes to generate.
    
    **Returns**
    - `List<int>`: The random bytes.

*   **`random.int`**
    Generates a random integer within a range.
    
    **Parameters**
    - `min` (`int`): Minimum value (inclusive).
    - `max` (`int`): Maximum value (exclusive).
    
    **Returns**
    - `int`: The random integer.

*   **`random.string`**
    Generates a random string.
    
    **Parameters**
    - `len` (`int`): Length of the string.
    - `charset` (`Option<str>`): Optional string of characters to use. If `None`, defaults to alphanumeric.
    
    **Returns**
    - `str`: The random string.

*   **`random.uuid`**
    Generates a random UUID (v4).
    
    **Returns**
    - `str`: The UUID string.

### Regex

The `regex` library provides regular expression capabilities using Rust's `regex` crate syntax.

**Note**: Currently, it primarily supports a single capture group. Multi-group support might be limited.


*   **`regex.match_all`**
    Returns all substrings matching the pattern in the haystack.
    
    If the pattern contains capture groups, returns the captured string for each match.
    
    **Parameters**
    - `haystack` (`str`): The string to search.
    - `pattern` (`str`): The regex pattern.
    
    **Returns**
    - `List<str>`: A list of matching strings.
    
    **Errors**
    - Returns an error string if the regex is invalid.

*   **`regex.match`**
    Returns the first substring matching the pattern.
    
    **Parameters**
    - `haystack` (`str`): The string to search.
    - `pattern` (`str`): The regex pattern.
    
    **Returns**
    - `str`: The matching string.
    
    **Errors**
    - Returns an error string if no match is found or the regex is invalid.

*   **`regex.replace_all`**
    Replaces all occurrences of the pattern with the value.
    
    **Parameters**
    - `haystack` (`str`): The string to modify.
    - `pattern` (`str`): The regex pattern to match.
    - `value` (`str`): The replacement string.
    
    **Returns**
    - `str`: The modified string.
    
    **Errors**
    - Returns an error string if the regex is invalid.

*   **`regex.replace`**
    Replaces the first occurrence of the pattern with the value.
    
    **Parameters**
    - `haystack` (`str`): The string to modify.
    - `pattern` (`str`): The regex pattern to match.
    - `value` (`str`): The replacement string.
    
    **Returns**
    - `str`: The modified string.
    
    **Errors**
    - Returns an error string if the regex is invalid.

### Report

The `report` library handles structured data reporting to the C2 server.

It allows you to:
- Exfiltrate files (in chunks).
- Report process snapshots.
- Report captured credentials (passwords, SSH keys).


*   **`report.file`**
    Reports (exfiltrates) a file from the host to the C2 server.
    
    The file is sent asynchronously in chunks.
    
    **Parameters**
    - `path` (`str`): The path of the file to exfiltrate.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if the file cannot be read or queued for reporting.

*   **`report.process_list`**
    Reports a snapshot of running processes.
    
    This updates the process list view in the C2 UI.
    
    **Parameters**
    - `list` (`List<Dict>`): The list of process dictionaries (typically from `process.list()`).
    
    **Returns**
    - `None`

*   **`report.ssh_key`**
    Reports a captured SSH private key.
    
    **Parameters**
    - `username` (`str`): The associated username.
    - `key` (`str`): The SSH key content.
    
    **Returns**
    - `None`

*   **`report.user_password`**
    Reports a captured user password.
    
    **Parameters**
    - `username` (`str`): The username.
    - `password` (`str`): The password.
    
    **Returns**
    - `None`

### Sys

The `sys` library provides general system interaction capabilities.

It supports:
- Process execution (`exec`, `shell`).
- System information (`get_os`, `get_ip`, `get_user`, `hostname`).
- Registry operations (Windows).
- DLL injection and reflection.
- Environment variable access.


*   **`sys.dll_inject`**
    Injects a DLL from disk into a remote process.
    
    **Parameters**
    - `dll_path` (`str`): Path to the DLL on disk.
    - `pid` (`int`): Target process ID.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if injection fails.

*   **`sys.dll_reflect`**
    Reflectively injects a DLL from memory into a remote process.
    
    **Parameters**
    - `dll_bytes` (`List<int>`): Content of the DLL.
    - `pid` (`int`): Target process ID.
    - `function_name` (`str`): Exported function to call.
    
    **Returns**
    - `None`
    
    **Errors**
    - Returns an error string if injection fails.

*   **`sys.exec`**
    Executes a program directly (without a shell).
    
    **Parameters**
    - `path` (`str`): Path to the executable.
    - `args` (`List<str>`): List of arguments.
    - `disown` (`Option<bool>`): If `True`, runs in background/detached.
    - `env_vars` (`Option<Dict<str, str>>`): Environment variables to set.
    
    **Returns**
    - `Dict`: Output containing `stdout`, `stderr`, and `status` (exit code).

*   **`sys.get_env`**
    Returns the current process's environment variables.
    
    **Returns**
    - `Dict<str, str>`: Map of environment variables.

*   **`sys.get_ip`**
    Returns network interface information.
    
    **Returns**
    - `List<Dict>`: List of interfaces with `name` and `ip`.

*   **`sys.get_os`**
    Returns information about the operating system.
    
    **Returns**
    - `Dict`: Details like `arch`, `distro`, `platform`.

*   **`sys.get_pid`**
    Returns the current process ID.
    
    **Returns**
    - `int`: The PID.

*   **`sys.get_reg`**
    Reads values from the Windows Registry.
    
    **Parameters**
    - `reghive` (`str`): The registry hive (e.g., "HKEY_LOCAL_MACHINE").
    - `regpath` (`str`): The registry path.
    
    **Returns**
    - `Dict<str, str>`: A dictionary of registry keys and values.

*   **`sys.get_user`**
    Returns information about the current user.
    
    **Returns**
    - `Dict`: User details (uid, gid, name, groups).

*   **`sys.hostname`**
    Returns the system hostname.
    
    **Returns**
    - `str`: The hostname.

*   **`sys.is_bsd`**
    Checks if the OS is BSD.
    
    **Returns**
    - `bool`: True if BSD.

*   **`sys.is_linux`**
    Checks if the OS is Linux.
    
    **Returns**
    - `bool`: True if Linux.

*   **`sys.is_macos`**
    Checks if the OS is macOS.
    
    **Returns**
    - `bool`: True if macOS.

*   **`sys.is_windows`**
    Checks if the OS is Windows.
    
    **Returns**
    - `bool`: True if Windows.

*   **`sys.shell`**
    Executes a command via the system shell (`/bin/sh` or `cmd.exe`).
    
    **Parameters**
    - `cmd` (`str`): The command string to execute.
    
    **Returns**
    - `Dict`: Output containing `stdout`, `stderr`, and `status`.

*   **`sys.write_reg_hex`**
    Writes a hex value to the Windows Registry.
    
    **Parameters**
    - `reghive` (`str`)
    - `regpath` (`str`)
    - `regname` (`str`)
    - `regtype` (`str`): e.g., "REG_BINARY".
    - `regvalue` (`str`): Hex string.
    
    **Returns**
    - `bool`: True on success.

*   **`sys.write_reg_int`**
    Writes an integer value to the Windows Registry.
    
    **Parameters**
    - `reghive` (`str`)
    - `regpath` (`str`)
    - `regname` (`str`)
    - `regtype` (`str`): e.g., "REG_DWORD".
    - `regvalue` (`int`)
    
    **Returns**
    - `bool`: True on success.

*   **`sys.write_reg_str`**
    Writes a string value to the Windows Registry.
    
    **Parameters**
    - `reghive` (`str`)
    - `regpath` (`str`)
    - `regname` (`str`)
    - `regtype` (`str`): e.g., "REG_SZ".
    - `regvalue` (`str`)
    
    **Returns**
    - `bool`: True on success.

### Time

The `time` library provides time measurement, formatting, and sleep capabilities.


*   **`time.format_to_epoch`**
    Converts a formatted time string to a Unix timestamp (epoch seconds).
    
    **Parameters**
    - `input` (`str`): The time string (e.g., "2023-01-01 12:00:00").
    - `format` (`str`): The format string (e.g., "%Y-%m-%d %H:%M:%S").
    
    **Returns**
    - `int`: The timestamp.
    
    **Errors**
    - Returns an error string if parsing fails.

*   **`time.format_to_readable`**
    Converts a Unix timestamp to a readable string.
    
    **Parameters**
    - `input` (`int`): The timestamp (epoch seconds).
    - `format` (`str`): The desired output format.
    
    **Returns**
    - `str`: The formatted time string.

*   **`time.now`**
    Returns the current time as a Unix timestamp.
    
    **Returns**
    - `int`: Current epoch seconds.

*   **`time.sleep`**
    Pauses execution for the specified number of seconds.
    
    **Parameters**
    - `secs` (`int`): Seconds to sleep.
    
    **Returns**
    - `None`

