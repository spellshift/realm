mod assert;

#[test]
fn test_try_except_basic() {
    // Exception raised and caught with `except Exception:`
    assert::pass(
        r#"
        res = "initial"
        try:
            x = 1 / 0
            res = "unreached"
        except Exception:
            res = "caught"
        assert_eq(res, "caught")
    "#,
    );

    // No exception raised
    assert::pass(
        r#"
        res = "initial"
        try:
            res = "ok"
        except Exception:
            res = "caught"
        assert_eq(res, "ok")
    "#,
    );

    // Bare except
    assert::pass(
        r#"
        res = "initial"
        try:
            unknown_var
        except:
            res = "caught_bare"
        assert_eq(res, "caught_bare")
    "#,
    );
}

#[test]
fn test_try_except_as_binding() {
    // Catch division by zero and bind to `e`
    assert::pass(
        r#"
        msg = ""
        try:
            1 / 0
        except Exception as e:
            msg = str(e)
        assert("by zero" in msg)
    "#,
    );

    // Catch index error, key error, value error, fail()
    assert::pass(
        r#"
        errors = []

        try:
            l = [1, 2]
            _ = l[10]
        except Exception as e:
            errors.append(e)

        try:
            d = {"a": 1}
            _ = d["missing"]
        except Exception as e:
            errors.append(e)

        try:
            int("not_a_number")
        except Exception as e:
            errors.append(e)

        try:
            fail("custom failure")
        except Exception as e:
            errors.append(e)

        assert_eq(len(errors), 4)
        assert("out of range" in errors[0])
        assert("missing" in errors[1])
        assert("invalid" in errors[2].lower())
        assert("custom failure" in errors[3])
    "#,
    );

    // Bare `except as e:`
    assert::pass(
        r#"
        msg = ""
        try:
            1 / 0
        except as e:
            msg = e
        assert("by zero" in msg)
    "#,
    );
}

#[test]
fn test_try_except_tuple_and_multiple_handlers() {
    // Tuple of exception types `except (Exception,) as e:`
    assert::pass(
        r#"
        caught = False
        try:
            1 / 0
        except (Exception,) as e:
            caught = True
            assert("by zero" in e)
        assert(caught)
    "#,
    );

    // Multiple except clauses: only first matching runs
    assert::pass(
        r#"
        which = 0
        try:
            1 / 0
        except Exception:
            which = 1
        except:
            which = 2
        assert_eq(which, 1)
    "#,
    );
}

#[test]
fn test_try_except_nested_and_functions() {
    // Error deep in call stack caught by outer try/except
    assert::pass(
        r#"
        def level3():
            return 1 / 0

        def level2():
            return level3()

        def level1():
            return level2()

        caught = ""
        try:
            level1()
        except Exception as e:
            caught = e

        assert("by zero" in caught)

        # Ensure subsequent function calls still work properly (stack/depth unwound)
        def ok_fn(x):
            return x + 1
        assert_eq(ok_fn(41), 42)
    "#,
    );

    // Nested try/except where inner catches and outer does not trigger
    assert::pass(
        r#"
        trace = []
        try:
            trace.append("outer_try")
            try:
                trace.append("inner_try")
                1 / 0
            except Exception:
                trace.append("inner_except")
            trace.append("after_inner")
        except Exception:
            trace.append("outer_except")
        assert_eq(trace, ["outer_try", "inner_try", "inner_except", "after_inner"])
    "#,
    );

    // Nested try/except where inner except raises a new error caught by outer except
    assert::pass(
        r#"
        trace = []
        try:
            try:
                1 / 0
            except Exception:
                trace.append("inner_except")
                int("bad")
        except Exception as e:
            trace.append("outer_except:" + e)
        assert_eq(len(trace), 2)
        assert_eq(trace[0], "inner_except")
        assert("outer_except:" in trace[1])
    "#,
    );
}

#[test]
fn test_try_except_else_finally() {
    // else runs only when no exception is raised
    assert::pass(
        r#"
        trace = []
        try:
            trace.append("try")
        except Exception:
            trace.append("except")
        else:
            trace.append("else")
        finally:
            trace.append("finally")
        assert_eq(trace, ["try", "else", "finally"])
    "#,
    );

    // else is skipped when exception is raised, finally still runs
    assert::pass(
        r#"
        trace = []
        try:
            trace.append("try")
            1 / 0
        except Exception:
            trace.append("except")
        else:
            trace.append("else")
        finally:
            trace.append("finally")
        assert_eq(trace, ["try", "except", "finally"])
    "#,
    );

    // try / finally without except: finally runs before exception propagates
    assert::pass(
        r#"
        cleaned_up = False
        caught_outer = False
        try:
            try:
                1 / 0
            finally:
                cleaned_up = True
        except Exception:
            caught_outer = True
        assert(cleaned_up)
        assert(caught_outer)
    "#,
    );
}

#[test]
fn test_try_except_control_flow() {
    // break and continue inside try/except/finally in a for loop
    assert::pass(
        r#"
        results = []
        finally_count = 0
        for i in range(5):
            try:
                if i == 1:
                    continue
                if i == 2:
                    1 / 0
                if i == 4:
                    break
                results.append(i)
            except Exception:
                results.append(-i)
            finally:
                finally_count += 1

        assert_eq(results, [0, -2, 3])
        assert_eq(finally_count, 5)
    "#,
    );

    // return inside try still runs finally (and skips else)
    assert::pass(
        r#"
        def f(fail_it, log):
            try:
                if fail_it:
                    1 / 0
                log.append("try_return")
                return "from_try"
            except Exception:
                log.append("except_return")
                return "from_except"
            else:
                log.append("else")
            finally:
                log.append("finally")

        log1 = []
        assert_eq(f(False, log1), "from_try")
        assert_eq(log1, ["try_return", "finally"])

        log2 = []
        assert_eq(f(True, log2), "from_except")
        assert_eq(log2, ["except_return", "finally"])
    "#,
    );

    // return inside finally overrides return in try/except and overrides unhandled exception
    assert::pass(
        r#"
        def override_return():
            try:
                1 / 0
            finally:
                return "from_finally"

        assert_eq(override_return(), "from_finally")
    "#,
    );
}

#[test]
fn test_raise_statement() {
    // raise Exception("message")
    assert::pass(
        r#"
        caught = ""
        try:
            raise Exception("something broke")
        except Exception as e:
            caught = e
        assert_eq(caught, "something broke")
    "#,
    );

    // raise Exception (without parens) and raise Exception()
    assert::pass(
        r#"
        count = 0
        try:
            raise Exception
        except Exception as e:
            assert_eq(e, "")
            count += 1

        try:
            raise Exception()
        except Exception as e:
            assert_eq(e, "")
            count += 1

        assert_eq(count, 2)
    "#,
    );

    // bare re-raise inside except
    assert::pass(
        r#"
        inner_saw = ""
        outer_saw = ""
        try:
            try:
                raise Exception("root cause")
            except Exception as e:
                inner_saw = e
                raise
        except Exception as e:
            outer_saw = e

        assert_eq(inner_saw, "root cause")
        assert_eq(outer_saw, "root cause")
    "#,
    );

    // bare raise outside except fails
    assert::fail("raise", "No active exception to reraise");

    // raising non-exception fails with TypeError
    assert::fail("raise 123", "exceptions must derive from Exception");
}

#[test]
fn test_invalid_except_type_and_syntax() {
    // Catching non-exception value raises TypeError
    assert::fail(
        r#"
        try:
            1 / 0
        except 123:
            pass
    "#,
        "catching classes that do not inherit from Exception is not allowed",
    );

    assert::fail(
        r#"
        try:
            1 / 0
        except len:
            pass
    "#,
        "catching classes that do not inherit from Exception is not allowed",
    );

    // Syntax errors
    assert::fail(
        r#"
        try:
            x = 1
    "#,
        "Expected 'except' or 'finally' after 'try' block.",
    );

    assert::fail(
        r#"
        try:
            x = 1
        except Exception as:
            pass
    "#,
        "Expected identifier after 'as'.",
    );
}

#[test]
fn test_comments_and_inline_try_except() {
    assert::pass(
        r#"
        res = 0
        try: 1 / 0
        # comment between try and except
        except Exception as e: res = 42
        assert_eq(res, 42)
    "#,
    );
}

#[test]
fn test_exception_builtin_and_analysis() {
    assert::pass(
        r#"
        assert_eq(Exception(), "")
        assert_eq(Exception("err"), "err")
        assert_eq(Exception("err", 404), '("err", 404)')
    "#,
    );

    let src = "try:\n    raise Exception('boom')\nexcept Exception as e:\n    pass\n";
    let mut lexer = eldritch_core::Lexer::new(src.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = eldritch_core::Parser::new(tokens);
    let (stmts, errors) = parser.parse();
    assert!(errors.is_empty());

    let node = eldritch_core::analysis::find_node_at_offset(&stmts, 18);
    assert!(node.is_some());
}
