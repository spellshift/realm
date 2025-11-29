mod assert;

// --- Enumerate Tests ---

#[test]
fn test_enumerate() {
    // Basic enumeration on list
    assert::pass(
        r#"
        l = ["a", "b", "c"]
        res = []
        for i, x in enumerate(l):
            res.append([i, x])
        assert_eq(res, [[0, "a"], [1, "b"], [2, "c"]])
    "#,
    );

    // Enumerate with custom start index
    assert::pass(
        r#"
        l = ["a", "b"]
        res = []
        for i, x in enumerate(l, 10):
            res.append([i, x])
        assert_eq(res, [[10, "a"], [11, "b"]])
    "#,
    );

    // Enumerate on tuple
    assert::pass(
        r#"
        t = ("x", "y")
        res_indices = []
        res_values = []
        for i, v in enumerate(t):
            res_indices.append(i)
            res_values.append(v)
        assert_eq(res_indices, [0, 1])
        assert_eq(res_values, ["x", "y"])
    "#,
    );

    // Enumerate on string
    assert::pass(
        r#"
        s = "hi"
        res = []
        for i, c in enumerate(s):
            res.append([i, c])
        assert_eq(res, [[0, "h"], [1, "i"]])
    "#,
    );

    // Validation: Enumerate returns a list of tuples, which can be indexed manually if not unpacking
    assert::pass(
        r#"
        e = enumerate(["a", "b"])
        assert_eq(e[0], (0, "a"))
        assert_eq(e[1], (1, "b"))
    "#,
    );
}

// --- Method Tests ---

#[test]
fn test_list_methods() {
    assert::pass(
        r#"
        # append
        l = [1]
        l.append(2)
        assert_eq(l, [1, 2])

        # extend
        l.extend([3, 4])
        assert_eq(l, [1, 2, 3, 4])
        l.extend((5,))
        assert_eq(l, [1, 2, 3, 4, 5])

        # insert
        l = [1, 3]
        l.insert(1, 2)
        assert_eq(l, [1, 2, 3])
        l.insert(0, 0)
        assert_eq(l, [0, 1, 2, 3])
        l.insert(100, 4) # Out of bounds index clamps to end
        assert_eq(l, [0, 1, 2, 3, 4])

        # remove
        l = [1, 2, 3, 2]
        l.remove(2)
        assert_eq(l, [1, 3, 2])

        # index
        l = [10, 20, 30]
        assert_eq(l.index(20), 1)

        # pop
        l = [1, 2]
        assert_eq(l.pop(), 2)
        assert_eq(l, [1])

        # sort
        l = [3, 1, 2]
        l.sort()
        assert_eq(l, [1, 2, 3])

        # Sort strings
        l = ["b", "a", "c"]
        l.sort()
        assert_eq(l, ["a", "b", "c"])
    "#,
    );

    assert::fail("l = [1]; l.remove(2)", "ValueError");
    assert::fail("l = [1]; l.index(2)", "ValueError");
}

#[test]
fn test_dict_methods() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}

        # keys, values, items
        ks = d.keys()
        assert_eq(len(ks), 2)

        vs = d.values()
        assert_eq(len(vs), 2)

        it = d.items()
        assert_eq(len(it), 2)

        # get
        assert_eq(d.get("a"), 1)
        assert_eq(d.get("c"), None)
        assert_eq(d.get("c", 10), 10)

        # update
        d.update({"c": 3})
        assert_eq(d["c"], 3)
        assert_eq(d["a"], 1)

        # popitem
        d = {"x": 1}
        item = d.popitem()
        assert_eq(item, ("x", 1))
        assert_eq(len(d), 0)
    "#,
    );

    assert::fail("d={}; d.popitem()", "empty");
}

#[test]
fn test_string_methods() {
    assert::pass(
        r#"
        s = "Hello World"

        # lower/upper
        assert_eq(s.lower(), "hello world")
        assert_eq(s.upper(), "HELLO WORLD")

        # split
        assert_eq("a,b,c".split(","), ["a", "b", "c"])
        assert_eq("a b c".split(" "), ["a", "b", "c"])

        # strip
        assert_eq("  abc  ".strip(), "abc")

        # starts/ends
        assert_eq(s.startswith("Hell"), True)
        assert_eq(s.endswith("ld"), True)
        assert_eq(s.startswith("x"), False)

        # find
        assert_eq(s.find("World"), 6)
        assert_eq(s.find("x"), -1)

        # replace
        assert_eq("banana".replace("a", "o"), "bonono")

        # join
        l = ["a", "b", "c"]
        assert_eq("-".join(l), "a-b-c")

        # format
        assert_eq("Hello {} {}".format("Mr", "Bond"), "Hello Mr Bond")
    "#,
    );
}

#[test]
fn test_slicing_robust() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]

        # Standard
        assert_eq(l[1:4], [1, 2, 3])

        # Step
        assert_eq(l[::2], [0, 2, 4])

        # Reversal
        assert_eq(l[::-1], [5, 4, 3, 2, 1, 0])

        # Reversal with bounds
        # Start at index 4 (val 4), stop before 1 (val 1), step -1
        # Indices: 4, 3, 2. Values: [4, 3, 2]
        assert_eq(l[4:1:-1], [4, 3, 2])

        # Negative indices with reversal
        # -1 is 5. -3 is 3.
        # l[-1:-3:-1] -> indices [5, 4]. Values [5, 4].
        assert_eq(l[-1:-3:-1], [5, 4])

        # Strings
        s = "abcdef"
        assert_eq(s[::-1], "fedcba")
        assert_eq(s[4:1:-1], "edc")

        # Tuples
        t = (1, 2, 3)
        assert_eq(t[::-1], (3, 2, 1))
    "#,
    );
}

// --- Function Arguments ---

#[test]
fn test_function_arguments() {
    // Default arguments
    assert::pass(
        r#"
        def f(a, b=10):
            return a + b
        assert_eq(f(5), 15)
        assert_eq(f(5, 20), 25)
    "#,
    );

    // Keyword arguments
    assert::pass(
        r#"
        def f(a, b, c=5):
            return a + b + c
        assert_eq(f(1, 2), 8)
        assert_eq(f(a=1, b=2), 8)
        assert_eq(f(b=2, a=1), 8)
        assert_eq(f(1, c=10, b=2), 13)
    "#,
    );

    // *args
    assert::pass(
        r#"
        def f(a, *args):
            return len(args)
        assert_eq(f(1), 0)
        assert_eq(f(1, 2, 3), 2)
    "#,
    );

    // **kwargs
    assert::pass(
        r#"
        def f(a, **kwargs):
            return kwargs["x"]
        assert_eq(f(1, x=10), 10)
    "#,
    );

    // All together
    assert::pass(
        r#"
        def f(a, b=2, *args, **kwargs):
            return a + b + len(args) + len(kwargs)

        # a=1, b=2, args=[], kwargs={} -> 1+2+0+0 = 3
        assert_eq(f(1), 3)

        # a=1, b=3, args=[], kwargs={} -> 1+3+0+0 = 4
        assert_eq(f(1, 3), 4)

        # a=1, b=3, args=[4, 5], kwargs={} -> 1+3+2+0 = 6
        assert_eq(f(1, 3, 4, 5), 6)

        # a=1, b=3, args=[4], kwargs={"x": 1} -> 1+3+1+1 = 6
        assert_eq(f(1, 3, 4, x=1), 6)
    "#,
    );
}

// --- Comprehensions ---

#[test]
fn test_comprehensions() {
    assert::pass(
        r#"
        l = [x * 2 for x in [1, 2, 3]]
        assert_eq(l, [2, 4, 6])

        l = [x for x in [1, 2, 3, 4] if x > 2]
        assert_eq(l, [3, 4])

        # Scoping check: x should not leak
        x = 100
        l = [x for x in [1, 2]]
        assert_eq(x, 100)
    "#,
    );

    assert::pass(
        r#"
        d = {str(x): x*x for x in [1, 2]}
        assert_eq(d["1"], 1)
        assert_eq(d["2"], 4)
    "#,
    );
}

// --- Tuples ---

#[test]
fn test_tuples() {
    assert::pass(
        r#"
        t = (1, 2, 3)
        assert_eq(t[0], 1)
        assert_eq(len(t), 3)
        assert_eq((1,), (1,))
        assert_eq((), ())
    "#,
    );
}

// --- Bitwise Operators ---

#[test]
fn test_bitwise() {
    assert::all_true(
        r#"
        (1 & 2) == 0
        (1 | 2) == 3
        (1 ^ 3) == 2
        ~0 == -1
        (1 << 2) == 4
        (8 >> 1) == 4
    "#,
    );
}

// --- Literals & Constants ---

#[test]
fn test_literals_and_constants() {
    assert::all_true(
        r#"
        True == True
        False == False
        True != False
        None == None
        None != False
        None != True
        None != 0
        None != ""
        None != []
        1 == 1
        "hello" == "hello"
        [1, 2] == [1, 2]
        {"a": 1} == {"a": 1}
    "#,
    );
}

#[test]
fn test_basic_arithmetic() {
    assert::all_true(
        r#"
        1 + 2 == 3
        10 - 2 == 8
        5 * 5 == 25
        10 / 2 == 5
        -5 + 3 == -2
        -(5 + 5) == -10
        1 + 2 * 3 == 7
        (1 + 2) * 3 == 9
        10 - 5 - 2 == 3
        10 / 2 * 3 == 15
    "#,
    );
}

#[test]
fn test_comparisons() {
    assert::all_true(
        r#"
        1 < 2
        2 > 1
        1 <= 1
        1 <= 2
        2 >= 2
        3 >= 2
        1 != 2
        1 == 1
        "a" < "b"
    "#,
    );
}

#[test]
fn test_logic_operators() {
    assert::all_true(
        r#"
        True and True
        not False
        (True or False) == True
        (False or False) == False
        not (1 == 2)
        (True or (1/0)) == True
    "#,
    );
    assert::pass("assert((False and (1/0)) == False)");
}

#[test]
fn test_if_else() {
    assert::pass(
        r#"
        x = 10
        res = 0
        if x > 5:
            res = 1
        else:
            res = 2
        assert_eq(res, 1)

        if x < 5:
            res = 3
        else:
            res = 4
        assert_eq(res, 4)
    "#,
    );

    assert::pass(
        r#"
        def check(x):
            if x == 0: return "zero"
            elif x == 1: return "one"
            elif x == 2: return "two"
            else: return "many"

        assert_eq(check(0), "zero")
        assert_eq(check(1), "one")
        assert_eq(check(2), "two")
        assert_eq(check(100), "many")
    "#,
    );
}

#[test]
fn test_loops() {
    assert::pass(
        r#"
        sum = 0
        for i in [1, 2, 3, 4]:
            sum = sum + i
        assert_eq(sum, 10)
    "#,
    );

    assert::pass(
        r#"
        sum = 0
        for i in range(5):
            sum = sum + i
        assert_eq(sum, 10)
    "#,
    );

    assert::pass(
        r#"
        res = 0
        for i in range(10):
            if i == 5:
                break
            res = i
        assert_eq(res, 4)
    "#,
    );

    assert::pass(
        r#"
        count = 0
        for i in range(5):
            if i == 2:
                continue
            count = count + 1
        assert_eq(count, 4)
    "#,
    );

    assert::pass(
        r#"
        total = 0
        for x in range(3):
            for y in range(3):
                if y == 1:
                    continue
                total = total + 1
        assert_eq(total, 6)
    "#,
    );
}

#[test]
fn test_functions_basic() {
    assert::pass(
        r#"
        def add(a, b):
            return a + b

        assert_eq(add(10, 5), 15)

        def do_nothing():
            return None

        assert_eq(do_nothing(), None)
    "#,
    );
}

#[test]
fn test_recursion() {
    assert::pass(
        r#"
        def fib(n):
            if n < 2:
                return n
            return fib(n-1) + fib(n-2)

        assert_eq(fib(6), 8)
    "#,
    );

    assert::pass(
        r#"
        def fact(n):
            if n <= 1: return 1
            return n * fact(n - 1)

        assert_eq(fact(5), 120)
    "#,
    );
}

#[test]
fn test_recursion_limit() {
    assert::fail(
        r#"
        def crash():
            crash()
        crash()
    "#,
        "Recursion limit exceeded",
    );
}

#[test]
fn test_closures() {
    assert::pass(
        r#"
        def make_adder(n):
            def adder(x):
                return x + n
            return adder

        add5 = make_adder(5)
        add10 = make_adder(10)

        assert_eq(add5(3), 8)
        assert_eq(add10(3), 13)
    "#,
    );

    assert::pass(
        r#"
        x = 0
        def inc():
            x = x + 1
            return x

        assert_eq(inc(), 1)
        assert_eq(inc(), 2)
    "#,
    );
}

#[test]
fn test_lists() {
    assert::pass(
        r#"
        l = [10, 20, 30]
        assert_eq(l[0], 10)
        assert_eq(l[2], 30)
        assert_eq(len(l), 3)
        assert_eq(l[-1], 30)
    "#,
    );

    assert::pass(
        r#"
        l = []
        l.append(1)
        l.append(2)
        assert_eq(l, [1, 2])

        val = l.pop()
        assert_eq(val, 2)
        assert_eq(l, [1])
    "#,
    );
}

#[test]
fn test_dictionaries() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        assert_eq(len(d), 2)
        assert_eq(d["a"], 1)

        keys = d.keys()
        assert_eq(len(keys), 2)
    "#,
    );

    assert::pass(
        r#"
        k = "foo"
        d = {k: "bar"}
        assert_eq(d["foo"], "bar")
    "#,
    );

    assert::pass(
        r#"
        d = {
            "inner": [1, 2],
            "nested": {"x": 10}
        }
        assert_eq(d["inner"][1], 2)
        assert_eq(d["nested"]["x"], 10)
    "#,
    );
}

#[test]
fn test_strings() {
    assert::pass(
        r#"
        x = "hello"
        y = "world"
        assert_eq(x + " " + y, "hello world")
    "#,
    );

    // F-strings
    assert::pass(
        r#"
        name = "Bob"
        age = 20
        assert_eq(f"{name} is {age}", "Bob is 20")
        assert_eq(f"Next is {age + 1}", "Next is 21")
    "#,
    );
}

#[test]
fn test_byte_strings() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(len(b), 5)
        assert_eq(type(b), "bytes")
    "#,
    );
}

#[test]
fn test_doc_strings() {
    assert::pass(
        r#"
        def func():
            """This is a docstring"""
            return 1
        assert_eq(func(), 1)
    "#,
    );

    assert::pass(
        r#"
        s = """line1
        line2"""
        assert_eq(type(s), "string")
    "#,
    );
}

#[test]
fn test_type_conversions() {
    assert::all_true(
        r#"
        int("123") == 123
        str(123) == "123"
        bool(1) == True
        bool(0) == False
        bool("a") == True
        bool("") == False
        bool([]) == False
        bool([1]) == True
    "#,
    );
}

#[test]
fn test_introspection() {
    assert::all_true(
        r#"
        type(1) == "int"
        type("s") == "string"
        type(True) == "bool"
        type(None) == "NoneType"
        type([]) == "list"
        type({}) == "dict"
        type(()) == "tuple"
    "#,
    );
}

#[test]
fn test_runtime_errors() {
    assert::fail("1 / 0", "divide by zero");
    assert::fail("undefined_var", "Undefined variable");
    assert::fail("1 + 'string'", "Unsupported binary op");
    assert::fail("l = []; l.pop()", "pop from empty list");
    assert::fail("l = [1]; l[5]", "List index out of range");
    assert::fail("d = {}; d['missing']", "KeyError");
    assert::fail("assert(False)", "Assertion failed");
    assert::fail("fail('boom')", "boom");
    assert::fail("len(1)", "not defined for type");
    assert::fail("1()", "Cannot call value");
    assert::fail("l = [1]; l.unknown()", "has no method");
}
