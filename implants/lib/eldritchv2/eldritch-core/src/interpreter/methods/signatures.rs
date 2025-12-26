use crate::ast::Value;
use crate::interpreter::signature::{MethodSignature, ParameterSignature};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

fn param(name: &str, type_name: Option<&str>, is_optional: bool) -> ParameterSignature {
    ParameterSignature {
        name: String::from(name),
        type_name: type_name.map(String::from),
        is_optional,
        is_variadic: false,
        is_kwargs: false,
    }
}

fn sig(
    name: &str,
    params: Vec<ParameterSignature>,
    ret: Option<&str>,
    doc: Option<&str>,
) -> MethodSignature {
    MethodSignature {
        name: String::from(name),
        params,
        return_type: ret.map(String::from),
        doc: doc.map(String::from),
        deprecated: None, // No native methods are deprecated
    }
}

pub fn get_native_method_signature(value: &Value, method: &str) -> Option<MethodSignature> {
    match value {
        Value::List(_) => get_list_signature(method),
        Value::Dictionary(_) => get_dict_signature(method),
        Value::Set(_) => get_set_signature(method),
        Value::String(_) => get_string_signature(method),
        _ => None,
    }
}

fn get_list_signature(method: &str) -> Option<MethodSignature> {
    match method {
        "append" => Some(sig(
            "append",
            vec![param("item", Some("any"), false)],
            Some("None"),
            Some("Appends an item to the end of the list."),
        )),
        "extend" => Some(sig(
            "extend",
            vec![param("iterable", Some("iterable"), false)],
            Some("None"),
            Some("Extends the list by appending elements from the iterable."),
        )),
        "insert" => Some(sig(
            "insert",
            vec![
                param("index", Some("int"), false),
                param("item", Some("any"), false),
            ],
            Some("None"),
            Some("Inserts an item at a given position."),
        )),
        "remove" => Some(sig(
            "remove",
            vec![param("item", Some("any"), false)],
            Some("None"),
            Some("Removes the first item from the list whose value is equal to x."),
        )),
        "index" => Some(sig(
            "index",
            vec![param("item", Some("any"), false)],
            Some("int"),
            Some("Returns the index of the first item whose value is equal to x."),
        )),
        "pop" => Some(sig(
            "pop",
            vec![], // Native pop implementation in list.rs ignores arguments for now (based on my read) but python supports pop([i])
            // Wait, list.rs says: args.require(0, "pop")? -> It requires 0 arguments!
            // So our signature should be empty.
            Some("any"),
            Some("Removes and returns the last item in the list."),
        )),
        "sort" => Some(sig(
            "sort",
            vec![], // list.rs requires 0 arguments. No key= or reverse= support yet in built-in list.sort?
            // "args.require(0, "sort")?" -> Yes, no args.
            Some("None"),
            Some("Sorts the items of the list in place."),
        )),
        _ => None,
    }
}

fn get_dict_signature(method: &str) -> Option<MethodSignature> {
    match method {
        "clear" => Some(sig(
            "clear",
            vec![],
            Some("None"),
            Some("Removes all items from the dictionary."),
        )),
        "keys" => Some(sig(
            "keys",
            vec![],
            Some("list"),
            Some("Returns a list of the dictionary's keys."),
        )),
        "values" => Some(sig(
            "values",
            vec![],
            Some("list"),
            Some("Returns a list of the dictionary's values."),
        )),
        "items" => Some(sig(
            "items",
            vec![],
            Some("list"),
            Some("Returns a list of the dictionary's (key, value) pairs."),
        )),
        "get" => Some(sig(
            "get",
            vec![
                param("key", Some("any"), false),
                param("default", Some("any"), true),
            ],
            Some("any"),
            Some("Returns the value for key if key is in the dictionary, else default."),
        )),
        "update" => Some(sig(
            "update",
            vec![param("other", Some("dict"), false)],
            Some("None"),
            Some("Updates the dictionary with the key/value pairs from other."),
        )),
        "pop" => Some(sig(
            "pop",
            vec![
                param("key", Some("any"), false),
                param("default", Some("any"), true),
            ],
            Some("any"),
            Some("Removes the specified key and returns the corresponding value."),
        )),
        "popitem" => Some(sig(
            "popitem",
            vec![],
            Some("tuple"),
            Some("Removes and returns a (key, value) pair from the dictionary."),
        )),
        "setdefault" => Some(sig(
            "setdefault",
            vec![
                param("key", Some("any"), false),
                param("default", Some("any"), true),
            ],
            Some("any"),
            Some(
                "If key is in the dictionary, return its value. If not, insert key with a value of default and return default.",
            ),
        )),
        _ => None,
    }
}

fn get_set_signature(method: &str) -> Option<MethodSignature> {
    match method {
        "add" => Some(sig(
            "add",
            vec![param("item", Some("any"), false)],
            Some("None"),
            Some("Adds an element to the set."),
        )),
        "clear" => Some(sig(
            "clear",
            vec![],
            Some("None"),
            Some("Removes all elements from the set."),
        )),
        "contains" => Some(sig(
            "contains",
            vec![param("item", Some("any"), false)],
            Some("bool"),
            Some("Returns True if the set contains the item."),
        )),
        "difference" => Some(sig(
            "difference",
            vec![param("other", Some("iterable"), false)],
            Some("set"),
            Some("Returns a new set with elements in the set that are not in the others."),
        )),
        "discard" => Some(sig(
            "discard",
            vec![param("item", Some("any"), false)],
            Some("None"),
            Some("Removes an element from a set if it is a member."),
        )),
        "intersection" => Some(sig(
            "intersection",
            vec![param("other", Some("iterable"), false)],
            Some("set"),
            Some("Returns a new set with elements common to the set and all others."),
        )),
        "isdisjoint" => Some(sig(
            "isdisjoint",
            vec![param("other", Some("iterable"), false)],
            Some("bool"),
            Some("Returns True if two sets have a null intersection."),
        )),
        "issubset" => Some(sig(
            "issubset",
            vec![param("other", Some("iterable"), false)],
            Some("bool"),
            Some("Returns True if another set contains this set."),
        )),
        "issuperset" => Some(sig(
            "issuperset",
            vec![param("other", Some("iterable"), false)],
            Some("bool"),
            Some("Returns True if this set contains another set."),
        )),
        "pop" => Some(sig(
            "pop",
            vec![],
            Some("any"),
            Some("Removes and returns an arbitrary set element."),
        )),
        "remove" => Some(sig(
            "remove",
            vec![param("item", Some("any"), false)],
            Some("None"),
            Some("Removes an element from a set; it must be a member."),
        )),
        "symmetric_difference" => Some(sig(
            "symmetric_difference",
            vec![param("other", Some("iterable"), false)],
            Some("set"),
            Some("Returns a new set with elements in either the set or other but not both."),
        )),
        "union" => Some(sig(
            "union",
            vec![param("other", Some("iterable"), false)],
            Some("set"),
            Some("Returns a new set with elements from the set and all others."),
        )),
        "update" => Some(sig(
            "update",
            vec![param("other", Some("iterable"), false)],
            Some("None"),
            Some("Update the set, adding elements from all others."),
        )),
        _ => None,
    }
}

fn get_string_signature(method: &str) -> Option<MethodSignature> {
    match method {
        "split" => Some(sig(
            "split",
            vec![param("sep", Some("str"), true)], // Actually args.require_range(0, 1) in code? Let's check str.rs
            Some("list"),
            Some("Returns a list of the words in the string, using sep as the delimiter string."),
        )),
        // Checking str.rs for arguments...
        // args.require_range(0, 1, "strip")
        "strip" => Some(sig(
            "strip",
            vec![param("chars", Some("str"), true)],
            Some("str"),
            Some("Returns a copy of the string with leading and trailing characters removed."),
        )),
        "lstrip" => Some(sig(
            "lstrip",
            vec![param("chars", Some("str"), true)],
            Some("str"),
            Some("Returns a copy of the string with leading characters removed."),
        )),
        "rstrip" => Some(sig(
            "rstrip",
            vec![param("chars", Some("str"), true)],
            Some("str"),
            Some("Returns a copy of the string with trailing characters removed."),
        )),
        "lower" => Some(sig(
            "lower",
            vec![],
            Some("str"),
            Some("Returns a copy of the string converted to lowercase."),
        )),
        "upper" => Some(sig(
            "upper",
            vec![],
            Some("str"),
            Some("Returns a copy of the string converted to uppercase."),
        )),
        "capitalize" => Some(sig(
            "capitalize",
            vec![],
            Some("str"),
            Some("Returns a copy of the string with its first character capitalized."),
        )),
        "title" => Some(sig(
            "title",
            vec![],
            Some("str"),
            Some("Returns a version of the string where each word is titlecased."),
        )),
        "startswith" => Some(sig(
            "startswith",
            vec![param("prefix", Some("str"), false)],
            Some("bool"),
            Some("Returns True if the string starts with the specified prefix."),
        )),
        "endswith" => Some(sig(
            "endswith",
            vec![param("suffix", Some("str"), false)],
            Some("bool"),
            Some("Returns True if the string ends with the specified suffix."),
        )),
        "removeprefix" => Some(sig(
            "removeprefix",
            vec![param("prefix", Some("str"), false)],
            Some("str"),
            Some("Returns the string with the given prefix removed."),
        )),
        "removesuffix" => Some(sig(
            "removesuffix",
            vec![param("suffix", Some("str"), false)],
            Some("str"),
            Some("Returns the string with the given suffix removed."),
        )),
        "find" => Some(sig(
            "find",
            vec![param("sub", Some("str"), false)],
            Some("int"),
            Some("Returns the lowest index in the string where substring sub is found."),
        )),
        "index" => Some(sig(
            "index",
            vec![param("sub", Some("str"), false)],
            Some("int"),
            Some("Like find(), but raises ValueError when the substring is not found."),
        )),
        "rfind" => Some(sig(
            "rfind",
            vec![param("sub", Some("str"), false)],
            Some("int"),
            Some("Returns the highest index in the string where substring sub is found."),
        )),
        "rindex" => Some(sig(
            "rindex",
            vec![param("sub", Some("str"), false)],
            Some("int"),
            Some("Like rfind(), but raises ValueError when the substring is not found."),
        )),
        "count" => Some(sig(
            "count",
            vec![param("sub", Some("str"), false)],
            Some("int"),
            Some("Returns the number of non-overlapping occurrences of substring sub."),
        )),
        "replace" => Some(sig(
            "replace",
            vec![
                param("old", Some("str"), false),
                param("new", Some("str"), false),
            ],
            Some("str"),
            Some(
                "Returns a copy of the string with all occurrences of substring old replaced by new.",
            ),
        )),
        "join" => Some(sig(
            "join",
            vec![param("iterable", Some("iterable"), false)],
            Some("str"),
            Some("Returns a string which is the concatenation of the strings in iterable."),
        )),
        "partition" => Some(sig(
            "partition",
            vec![param("sep", Some("str"), false)],
            Some("tuple"),
            Some("Splits the string at the first occurrence of sep."),
        )),
        "rpartition" => Some(sig(
            "rpartition",
            vec![param("sep", Some("str"), false)],
            Some("tuple"),
            Some("Splits the string at the last occurrence of sep."),
        )),
        // "split" is handled above
        // "splitlines"?
        "codepoints" => Some(sig(
            "codepoints",
            vec![],
            Some("list"),
            Some("Returns a list of the integer codepoints in the string."),
        )),
        "elems" => Some(sig(
            "elems",
            vec![],
            Some("list"),
            Some("Returns a list of the characters in the string."),
        )),
        "isalnum" => Some(sig(
            "isalnum",
            vec![],
            Some("bool"),
            Some("Returns True if all characters in the string are alphanumeric."),
        )),
        "isalpha" => Some(sig(
            "isalpha",
            vec![],
            Some("bool"),
            Some("Returns True if all characters in the string are alphabetic."),
        )),
        "isdigit" => Some(sig(
            "isdigit",
            vec![],
            Some("bool"),
            Some("Returns True if all characters in the string are digits."),
        )),
        "islower" => Some(sig(
            "islower",
            vec![],
            Some("bool"),
            Some("Returns True if all cased characters in the string are lowercase."),
        )),
        "isupper" => Some(sig(
            "isupper",
            vec![],
            Some("bool"),
            Some("Returns True if all cased characters in the string are uppercase."),
        )),
        "isspace" => Some(sig(
            "isspace",
            vec![],
            Some("bool"),
            Some("Returns True if there are only whitespace characters in the string."),
        )),
        "istitle" => Some(sig(
            "istitle",
            vec![],
            Some("bool"),
            Some("Returns True if the string is a titlecased string."),
        )),
        _ => None,
    }
}
