extern crate alloc;

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use eldritch_core::Value;
use eldritch_core::introspection::{find_best_match, get_type_name, is_truthy};
use spin::RwLock;

#[test]
fn test_find_best_match() {
    let candidates = vec![
        "append".to_string(),
        "extend".to_string(),
        "insert".to_string(),
        "pop".to_string(),
    ];

    // Exact match
    assert_eq!(
        find_best_match("append", &candidates),
        Some("append".to_string())
    );

    // Close matches
    assert_eq!(
        find_best_match("apend", &candidates),
        Some("append".to_string())
    );
    assert_eq!(
        find_best_match("ppend", &candidates),
        Some("append".to_string())
    );
    assert_eq!(
        find_best_match("appnd", &candidates),
        Some("append".to_string())
    );

    // Too far
    assert_eq!(find_best_match("xyz", &candidates), None);
    // Based on actual behavior observed: "appenddddd" (len 10) vs "append" (len 6).
    // Threshold calculation:
    // target="appenddddd" (10). threshold = 10/2 + 1 = 6. clamp(1, 4) -> 4.
    // len_diff = 4. 4 > 4 is False.
    // dist("appenddddd", "append") = 4 deletions.
    // 4 <= 4 is True. Match!
    assert_eq!(
        find_best_match("appenddddd", &candidates),
        Some("append".to_string())
    );

    // Empty candidates
    assert_eq!(find_best_match("append", &[]), None);

    // Empty target
    // threshold = 0/2 + 1 = 1.
    // "append" len 6. len_diff = 6 > 1. Skip.
    assert_eq!(find_best_match("", &candidates), None);

    // Test threshold logic "config" -> "get_config"
    let candidates2 = vec!["get_config".to_string()];
    // "config" len 6. threshold = 3+1 = 4.
    // "get_config" len 10. diff 4. OK.
    // dist("config", "get_config") = 4. (insert "get_").
    // 4 <= 4. Match.
    assert_eq!(
        find_best_match("config", &candidates2),
        Some("get_config".to_string())
    );

    // Boundary check
    // "a" -> "b"
    // threshold = 0+1 = 1.
    // diff 0.
    // dist 1.
    // match.
    let candidates3 = vec!["b".to_string()];
    assert_eq!(find_best_match("a", &candidates3), Some("b".to_string()));
}

#[test]
fn test_is_truthy() {
    assert!(!is_truthy(&Value::None));
    assert!(is_truthy(&Value::Bool(true)));
    assert!(!is_truthy(&Value::Bool(false)));
    assert!(is_truthy(&Value::Int(1)));
    assert!(!is_truthy(&Value::Int(0)));
    assert!(is_truthy(&Value::Float(1.0)));
    assert!(!is_truthy(&Value::Float(0.0)));
    assert!(is_truthy(&Value::String("a".to_string())));
    assert!(!is_truthy(&Value::String("".to_string())));
    assert!(is_truthy(&Value::Bytes(vec![1])));
    assert!(!is_truthy(&Value::Bytes(vec![])));
    assert!(is_truthy(&Value::List(Arc::new(RwLock::new(vec![
        Value::Int(1)
    ])))));
    assert!(!is_truthy(&Value::List(Arc::new(RwLock::new(vec![])))));
    assert!(is_truthy(&Value::Tuple(vec![Value::Int(1)])));
    assert!(!is_truthy(&Value::Tuple(vec![])));
}

#[test]
fn test_get_type_name() {
    assert_eq!(get_type_name(&Value::None), "NoneType");
    assert_eq!(get_type_name(&Value::Bool(true)), "bool");
    assert_eq!(get_type_name(&Value::Int(1)), "int");
    assert_eq!(get_type_name(&Value::Float(1.0)), "float");
    assert_eq!(get_type_name(&Value::String("".to_string())), "string");
    assert_eq!(get_type_name(&Value::Bytes(vec![])), "bytes");
    assert_eq!(
        get_type_name(&Value::List(Arc::new(RwLock::new(vec![])))),
        "list"
    );
    assert_eq!(
        get_type_name(&Value::Dictionary(Arc::new(
            RwLock::new(Default::default())
        ))),
        "dict"
    );
    assert_eq!(
        get_type_name(&Value::Set(Arc::new(RwLock::new(Default::default())))),
        "set"
    );
    assert_eq!(get_type_name(&Value::Tuple(vec![])), "tuple");
}
