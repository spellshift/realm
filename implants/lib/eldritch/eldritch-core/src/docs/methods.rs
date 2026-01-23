/// Documentation for methods on builtin types.
/// This file is parsed by the documentation generator.

// List methods
// ============

//: list.append
//: Appends an item to the end of the list.
//:
//: **Parameters**
//: - `x` (Any): The item to append.
//:
//: **Returns**
//: - `None`

//: list.extend
//: Extends the list by appending all the items from the iterable.
//:
//: **Parameters**
//: - `iterable` (Iterable): The elements to add.
//:
//: **Returns**
//: - `None`

//: list.insert
//: Inserts an item at a given position.
//:
//: **Parameters**
//: - `i` (Int): The index of the element before which to insert.
//: - `x` (Any): The element to insert.
//:
//: **Returns**
//: - `None`

//: list.remove
//: Removes the first item from the list whose value is equal to x.
//: Raises ValueError if there is no such item.
//:
//: **Parameters**
//: - `x` (Any): The item to remove.
//:
//: **Returns**
//: - `None`

//: list.pop
//: Removes the item at the given position in the list, and returns it.
//: If no index is specified, removes and returns the last item in the list.
//:
//: **Parameters**
//: - `i` (Option<Int>): The index of the item to remove. Defaults to -1.
//:
//: **Returns**
//: - `Any`: The removed item.

//: list.clear
//: Removes all items from the list.
//:
//: **Returns**
//: - `None`

//: list.index
//: Returns the zero-based index in the list of the first item whose value is equal to x.
//: Raises ValueError if there is no such item.
//:
//: **Parameters**
//: - `x` (Any): The item to search for.
//: - `start` (Option<Int>): Optional start index.
//: - `end` (Option<Int>): Optional end index.
//:
//: **Returns**
//: - `Int`: The index of the item.

//: list.count
//: Returns the number of times x appears in the list.
//:
//: **Parameters**
//: - `x` (Any): The item to count.
//:
//: **Returns**
//: - `Int`: The count.

//: list.sort
//: Sorts the items of the list in place.
//:
//: **Parameters**
//: - `key` (Option<Function>): A function of one argument that is used to extract a comparison key from each list element.
//: - `reverse` (Option<Bool>): If set to True, then the list elements are sorted as if each comparison were reversed.
//:
//: **Returns**
//: - `None`

//: list.reverse
//: Reverses the elements of the list in place.
//:
//: **Returns**
//: - `None`

//: list.copy
//: Returns a shallow copy of the list.
//:
//: **Returns**
//: - `List`: A shallow copy of the list.


// Dictionary methods
// ==================

//: dict.clear
//: Removes all items from the dictionary.
//:
//: **Returns**
//: - `None`

//: dict.copy
//: Returns a shallow copy of the dictionary.
//:
//: **Returns**
//: - `Dict`: A shallow copy.

//: dict.fromkeys
//: Create a new dictionary with keys from iterable and values set to value.
//:
//: **Parameters**
//: - `iterable` (Iterable): The keys.
//: - `value` (Any): The value to set. Defaults to None.
//:
//: **Returns**
//: - `Dict`: The new dictionary.

//: dict.get
//: Return the value for key if key is in the dictionary, else default.
//:
//: **Parameters**
//: - `key` (Any): The key to search for.
//: - `default` (Any): The value to return if key is not found. Defaults to None.
//:
//: **Returns**
//: - `Any`: The value or default.

//: dict.items
//: Return a new view of the dictionary's items ((key, value) pairs).
//:
//: **Returns**
//: - `List<Tuple>`: A list of (key, value) tuples.

//: dict.keys
//: Return a new view of the dictionary's keys.
//:
//: **Returns**
//: - `List`: A list of keys.

//: dict.pop
//: Remove the key from the dictionary and return its value.
//: If key is not found, default is returned if given, otherwise KeyError is raised.
//:
//: **Parameters**
//: - `key` (Any): The key to remove.
//: - `default` (Option<Any>): The value to return if key is not found.
//:
//: **Returns**
//: - `Any`: The removed value.

//: dict.popitem
//: Remove and return a (key, value) pair from the dictionary.
//: Pairs are returned in LIFO order.
//:
//: **Returns**
//: - `Tuple`: A (key, value) pair.

//: dict.setdefault
//: If key is in the dictionary, return its value.
//: If not, insert key with a value of default and return default.
//:
//: **Parameters**
//: - `key` (Any): The key.
//: - `default` (Any): The default value. Defaults to None.
//:
//: **Returns**
//: - `Any`: The value.

//: dict.update
//: Update the dictionary with the key/value pairs from other, overwriting existing keys.
//:
//: **Parameters**
//: - `other` (Dict | Iterable): The dictionary or iterable of pairs to update from.
//:
//: **Returns**
//: - `None`

//: dict.values
//: Return a new view of the dictionary's values.
//:
//: **Returns**
//: - `List`: A list of values.


// Set methods
// ===========

//: set.add
//: Adds an element to the set.
//:
//: **Parameters**
//: - `elem` (Any): The element to add.
//:
//: **Returns**
//: - `None`

//: set.clear
//: Removes all elements from the set.
//:
//: **Returns**
//: - `None`

//: set.copy
//: Returns a shallow copy of the set.
//:
//: **Returns**
//: - `Set`: A shallow copy.

//: set.difference
//: Return the difference of two or more sets as a new set.
//: (i.e. all elements that are in this set but not the others.)
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `Set`: The difference set.

//: set.difference_update
//: Remove all elements of another set from this set.
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `None`

//: set.discard
//: Remove an element from a set if it is a member.
//: If the element is not a member, do nothing.
//:
//: **Parameters**
//: - `elem` (Any): The element to remove.
//:
//: **Returns**
//: - `None`

//: set.intersection
//: Return the intersection of two or more sets as a new set.
//: (i.e. elements that are common to all of the sets.)
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `Set`: The intersection set.

//: set.intersection_update
//: Update the set with the intersection of itself and another.
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `None`

//: set.isdisjoint
//: Return True if two sets have a null intersection.
//:
//: **Parameters**
//: - `other` (Iterable): Another set/iterable.
//:
//: **Returns**
//: - `Bool`

//: set.issubset
//: Report whether another set contains this set.
//:
//: **Parameters**
//: - `other` (Iterable): Another set/iterable.
//:
//: **Returns**
//: - `Bool`

//: set.issuperset
//: Report whether this set contains another set.
//:
//: **Parameters**
//: - `other` (Iterable): Another set/iterable.
//:
//: **Returns**
//: - `Bool`

//: set.pop
//: Remove and return an arbitrary set element.
//: Raises KeyError if the set is empty.
//:
//: **Returns**
//: - `Any`: The removed element.

//: set.remove
//: Remove an element from a set; it must be a member.
//: If the element is not a member, raise a KeyError.
//:
//: **Parameters**
//: - `elem` (Any): The element to remove.
//:
//: **Returns**
//: - `None`

//: set.symmetric_difference
//: Return the symmetric difference of two sets as a new set.
//: (i.e. elements that are in either of the sets, but not both.)
//:
//: **Parameters**
//: - `other` (Iterable): Another set/iterable.
//:
//: **Returns**
//: - `Set`: The symmetric difference.

//: set.symmetric_difference_update
//: Update a set with the symmetric difference of itself and another.
//:
//: **Parameters**
//: - `other` (Iterable): Another set/iterable.
//:
//: **Returns**
//: - `None`

//: set.union
//: Return the union of sets as a new set.
//: (i.e. all elements that are in either set.)
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `Set`: The union.

//: set.update
//: Update a set with the union of itself and others.
//:
//: **Parameters**
//: - `*others` (Iterable): Other sets/iterables.
//:
//: **Returns**
//: - `None`


// String methods
// ==============

//: str.capitalize
//: Return a copy of the string with its first character capitalized and the rest lowercased.
//:
//: **Returns**
//: - `String`

//: str.casefold
//: Return a casefolded copy of the string. Casefolded strings may be used for caseless matching.
//:
//: **Returns**
//: - `String`

//: str.center
//: Return centered in a string of length width.
//: Padding is done using the specified fillchar (default is an ASCII space).
//:
//: **Parameters**
//: - `width` (Int): The total width.
//: - `fillchar` (String): The padding character.
//:
//: **Returns**
//: - `String`

//: str.count
//: Return the number of non-overlapping occurrences of substring sub in the range [start, end].
//:
//: **Parameters**
//: - `sub` (String): The substring to count.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Int`

//: str.endswith
//: Return True if the string ends with the specified suffix, otherwise return False.
//:
//: **Parameters**
//: - `suffix` (String | Tuple<String>): The suffix to check.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Bool`

//: str.find
//: Return the lowest index in the string where substring sub is found within the slice s[start:end].
//: Return -1 if sub is not found.
//:
//: **Parameters**
//: - `sub` (String): The substring to find.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Int`

//: str.format
//: Perform a string formatting operation.
//: The string on which this method is called can contain literal text or replacement fields delimited by braces {}.
//:
//: **Parameters**
//: - `*args` (Any): Positional arguments.
//: - `**kwargs` (Any): Keyword arguments.
//:
//: **Returns**
//: - `String`

//: str.index
//: Like find(), but raise ValueError when the substring is not found.
//:
//: **Parameters**
//: - `sub` (String): The substring to find.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Int`

//: str.isalnum
//: Return True if all characters in the string are alphanumeric and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isalpha
//: Return True if all characters in the string are alphabetic and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isascii
//: Return True if all characters in the string are ASCII.
//:
//: **Returns**
//: - `Bool`

//: str.isdecimal
//: Return True if all characters in the string are decimal characters and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isdigit
//: Return True if all characters in the string are digits and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isidentifier
//: Return True if the string is a valid identifier.
//:
//: **Returns**
//: - `Bool`

//: str.islower
//: Return True if all cased characters in the string are lowercase and there is at least one cased character.
//:
//: **Returns**
//: - `Bool`

//: str.isnumeric
//: Return True if all characters in the string are numeric characters and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isprintable
//: Return True if all characters in the string are printable or the string is empty.
//:
//: **Returns**
//: - `Bool`

//: str.isspace
//: Return True if all characters in the string are whitespace and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.istitle
//: Return True if the string is a titlecased string and there is at least one character.
//:
//: **Returns**
//: - `Bool`

//: str.isupper
//: Return True if all cased characters in the string are uppercase and there is at least one cased character.
//:
//: **Returns**
//: - `Bool`

//: str.join
//: Return a string which is the concatenation of the strings in iterable.
//: The separator between elements is the string providing this method.
//:
//: **Parameters**
//: - `iterable` (Iterable): The strings to join.
//:
//: **Returns**
//: - `String`

//: str.ljust
//: Return the string left justified in a string of length width.
//: Padding is done using the specified fillchar (default is an ASCII space).
//:
//: **Parameters**
//: - `width` (Int): The total width.
//: - `fillchar` (String): The padding character.
//:
//: **Returns**
//: - `String`

//: str.lower
//: Return a copy of the string with all cased characters converted to lowercase.
//:
//: **Returns**
//: - `String`

//: str.lstrip
//: Return a copy of the string with leading whitespace removed.
//: If chars is given and not None, remove characters in chars instead.
//:
//: **Parameters**
//: - `chars` (Option<String>): The characters to remove.
//:
//: **Returns**
//: - `String`

//: str.partition
//: Split the string at the first occurrence of sep, and return a 3-tuple containing the part before the separator, the separator itself, and the part after the separator.
//: If the separator is not found, return a 3-tuple containing the string itself, followed by two empty strings.
//:
//: **Parameters**
//: - `sep` (String): The separator.
//:
//: **Returns**
//: - `Tuple`

//: str.removeprefix
//: Return a str with the given prefix string removed if present.
//: If the string starts with the prefix string, return string[len(prefix):]. Otherwise, return a copy of the original string.
//:
//: **Parameters**
//: - `prefix` (String): The prefix to remove.
//:
//: **Returns**
//: - `String`

//: str.removesuffix
//: Return a str with the given suffix string removed if present.
//: If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)]. Otherwise, return a copy of the original string.
//:
//: **Parameters**
//: - `suffix` (String): The suffix to remove.
//:
//: **Returns**
//: - `String`

//: str.replace
//: Return a copy of the string with all occurrences of substring old replaced by new.
//: If the optional argument count is given, only the first count occurrences are replaced.
//:
//: **Parameters**
//: - `old` (String): The substring to replace.
//: - `new` (String): The replacement string.
//: - `count` (Option<Int>): The max number of replacements.
//:
//: **Returns**
//: - `String`

//: str.rfind
//: Return the highest index in the string where substring sub is found.
//: Return -1 if sub is not found.
//:
//: **Parameters**
//: - `sub` (String): The substring to find.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Int`

//: str.rindex
//: Like rfind() but raises ValueError when the substring is not found.
//:
//: **Parameters**
//: - `sub` (String): The substring to find.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Int`

//: str.rjust
//: Return the string right justified in a string of length width.
//: Padding is done using the specified fillchar (default is an ASCII space).
//:
//: **Parameters**
//: - `width` (Int): The total width.
//: - `fillchar` (String): The padding character.
//:
//: **Returns**
//: - `String`

//: str.rpartition
//: Split the string at the last occurrence of sep, and return a 3-tuple containing the part before the separator, the separator itself, and the part after the separator.
//: If the separator is not found, return a 3-tuple containing two empty strings, followed by the string itself.
//:
//: **Parameters**
//: - `sep` (String): The separator.
//:
//: **Returns**
//: - `Tuple`

//: str.rsplit
//: Return a list of the words in the string, using sep as the delimiter string.
//:
//: **Parameters**
//: - `sep` (Option<String>): The delimiter.
//: - `maxsplit` (Option<Int>): The max number of splits.
//:
//: **Returns**
//: - `List<String>`

//: str.rstrip
//: Return a copy of the string with trailing whitespace removed.
//: If chars is given and not None, remove characters in chars instead.
//:
//: **Parameters**
//: - `chars` (Option<String>): The characters to remove.
//:
//: **Returns**
//: - `String`

//: str.split
//: Return a list of the words in the string, using sep as the delimiter string.
//:
//: **Parameters**
//: - `sep` (Option<String>): The delimiter.
//: - `maxsplit` (Option<Int>): The max number of splits.
//:
//: **Returns**
//: - `List<String>`

//: str.splitlines
//: Return a list of the lines in the string, breaking at line boundaries.
//: Line breaks are not included in the resulting list unless keepends is given and true.
//:
//: **Parameters**
//: - `keepends` (Bool): Whether to keep line breaks.
//:
//: **Returns**
//: - `List<String>`

//: str.startswith
//: Return True if string starts with the specified prefix, otherwise return False.
//:
//: **Parameters**
//: - `prefix` (String | Tuple<String>): The prefix to check.
//: - `start` (Option<Int>): The start index.
//: - `end` (Option<Int>): The end index.
//:
//: **Returns**
//: - `Bool`

//: str.strip
//: Return a copy of the string with the leading and trailing whitespace removed.
//: If chars is given and not None, remove characters in chars instead.
//:
//: **Parameters**
//: - `chars` (Option<String>): The characters to remove.
//:
//: **Returns**
//: - `String`

//: str.swapcase
//: Return a copy of the string with uppercase characters converted to lowercase and vice versa.
//:
//: **Returns**
//: - `String`

//: str.title
//: Return a titlecased version of the string where words start with an uppercase character and the remaining characters are lowercase.
//:
//: **Returns**
//: - `String`

//: str.upper
//: Return a copy of the string with all cased characters converted to uppercase.
//:
//: **Returns**
//: - `String`

//: str.zfill
//: Return a copy of the string left filled with ASCII '0' digits to make a string of length width.
//:
//: **Parameters**
//: - `width` (Int): The total width.
//:
//: **Returns**
//: - `String`

//: str.codepoints
//: Returns a list of the integer codepoints of the string.
//:
//: **Returns**
//: - `List<Int>`

//: str.elems
//: Returns a list of single-character strings containing the characters of the string.
//:
//: **Returns**
//: - `List<String>`
