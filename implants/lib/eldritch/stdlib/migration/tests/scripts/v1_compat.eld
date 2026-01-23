v1funcs = {
    "int": [],
    "float": [],
    "string": ["capitalize", "codepoints", "count", "elems", "endswith", "find", "format", "index", "isalnum", "isalpha", "isdigit", "islower", "isspace", "istitle", "isupper", "join", "lower", "lstrip", "partition", "removeprefix", "removesuffix", "replace", "rfind", "rindex", "rpartition", "rsplit", "rstrip", "split", "splitlines", "startswith", "strip", "title", "upper"],
    "bool": [],
    "list": ["append", "clear", "extend", "index", "insert", "pop", "remove"],
    "dict": ["clear", "get", "items", "keys", "pop", "popitem", "setdefault", "update", "values"],
    "tuple": [],
}

items = [
    10,                 # int
    3.14,               # float
    "eldritch",         # string
    True,               # bool
    [1,2,"a"],          # list
    {"a": 4},           # dict
    (1,2,3),            # tuple

    #b"\00\01",         # bytes - v2 exclusive
    #{"a", 1, 2, "b"}   # set - v2 exclusive
]

exit = False # Used to force an error

if "test>>>".rstrip(">") != "test":
    print(f"[!] Fail: failed to rstrip")

if "test> > >".rstrip("> ") != "test":
    print(f"[!] Fail: failed to rstrip")

if "< < <test".lstrip("< ") != "test":
    print(f"[!] Fail: failed to lstrip")

if "<<<test>>>".strip("<>") != "test":
    print(f"[!] Fail: failed to strip")

if len("test    test".split()) != 2:
    print("[!] Fail: split should handle repeats", "test    test".split())
    exit()

if len("test\ttest".split()) != 2:
    print("[!] Fail: split should split on tabs")
    exit()

# These functions should exist
c = chr(ord('A'))
if c != "A":
    print(f"[!] Fail: {c} != 'A'")
    exit()

for i in items:
    if type(i) in ("string", "list", "tuple"):
        i[0] # Ensure substrings for indexable types
        pass
    # look through all the types and see if they have the expected functions
    wants = dir(i)
    for func in v1funcs.get(type(i), []):
        if func not in wants:
            t = type(i)
            print(f"[!] Fail: missing function '{t}.{func}()'")
            #exit() # force an error to quit

res = regex.match("banana phone", r"(na+\sp)") # Raw strings dont work
