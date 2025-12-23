#[cfg(test)]
mod tests {
    use eldritch_core::Interpreter;

    fn check_script(source: &str) {
        let mut interpreter = Interpreter::new();
        match interpreter.interpret(source) {
            Ok(_) => {}
            Err(e) => panic!("Script execution failed: {e}"),
        }
    }

    #[test]
    fn test_list_in() {
        let script = r#"
lst = ['a', 'b']
assert('a' in lst)
assert('b' in lst)
assert('c' not in lst)
assert('x' not in lst)
"#;
        check_script(script);
    }

    #[test]
    fn test_range_in() {
        let script = r#"
x = 0
if 1 in range(10):
    x += 1
assert(x == 1)
assert(5 in range(10))
assert(10 not in range(10))
assert(-1 not in range(10))
"#;
        check_script(script);
    }

    #[test]
    fn test_dict_in() {
        let script = r#"
d = {'key': 'value'}
assert('key' in d)
assert('value' not in d)
assert('other' not in d)
"#;
        check_script(script);
    }

    #[test]
    fn test_set_in() {
        // Test set literal syntax
        let script = r#"
s = {1, 2, 3}
assert(1 in s)
assert(4 not in s)
assert(2 in s)
"#;
        check_script(script);
    }

    #[test]
    fn test_set_comp() {
        let script = r#"
s = {x for x in [1, 2, 3] if x > 1}
assert(1 not in s)
assert(2 in s)
assert(3 in s)
"#;
        check_script(script);
    }

    #[test]
    fn test_empty_set_is_dict() {
        // In Python {} is a dict, not a set.
        let script = r#"
x = {}
# Dicts are not sets. We can check type if we had type().
# Or check behavior. Dicts don't support add(), sets do.
# But we only have limited builtins.
# Let's just try to treat it as dict.
x['a'] = 1
assert(x['a'] == 1)
"#;
        check_script(script);
    }

    #[test]
    fn test_string_in() {
        let script = r#"
s = "hello world"
assert("hello" in s)
assert("world" in s)
assert("o w" in s)
assert("z" not in s)
"#;
        check_script(script);
    }

    #[test]
    fn test_tuple_in() {
        let script = r#"
t = (1, 2, 3)
assert(1 in t)
assert(4 not in t)
"#;
        check_script(script);
    }

    #[test]
    fn test_not_in_precedence() {
        // Standard checks for precedence logic
        let script = r#"
assert(1 not in [2, 3])
assert((1+1) in [2, 3])
"#;
        check_script(script);
    }
}
