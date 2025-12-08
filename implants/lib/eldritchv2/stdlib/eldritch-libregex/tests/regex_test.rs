use eldritch_core::{Interpreter, Value};
use eldritch_libregex::std::StdRegexLibrary;

#[test]
fn test_regex() {
    let lib = StdRegexLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // regex.match
    // The library requires exactly 1 capture group.
    // Argument order in v2: (pattern, haystack)
    // pattern: '(a+)'
    // haystack: 'aaa'

    let res = interp.interpret("regex.match('(a+)', 'aaa')").unwrap();
    if let Value::String(s) = res {
        // v2 regex.match returns String (the single capture)
        assert_eq!(s, "aaa".to_string());
    } else {
        panic!("Expected string, got {:?}", res);
    }

    // regex.match no match
    let res = interp.interpret("regex.match('(b+)', 'aaa')").unwrap();
    if let Value::String(s) = res {
        assert_eq!(s, "".to_string());
    } else {
        panic!("Expected string, got {:?}", res);
    }

    // regex.match_all
    // Each match must have 1 capture group.
    // pattern: '(a)'
    // haystack: 'aaa'
    let res = interp.interpret("regex.match_all('(a)', 'aaa')").unwrap();
    if let Value::List(l) = res {
        let list = l.read();
        assert_eq!(list.len(), 3);
        // match_all returns list of strings (captures)
        assert_eq!(list[0], Value::String("a".to_string()));
    } else {
        panic!("Expected list, got {:?}", res);
    }

    // regex.replace
    // regex.replace(pattern, haystack, replacement)

    let res = interp.interpret("regex.replace('a', 'aaa', 'b')").unwrap();
    // Replaces first occurrence of 'a' in 'aaa' with 'b' -> 'baa'
    assert_eq!(res, Value::String("baa".to_string()));

    // regex.replace_all
    // regex.replace_all(pattern, haystack, replacement)

    let res = interp.interpret("regex.replace_all('a', 'aaa', 'b')").unwrap();
    // Replaces all occurrences of 'a' in 'aaa' with 'b' -> 'bbb'
    assert_eq!(res, Value::String("bbb".to_string()));
}
