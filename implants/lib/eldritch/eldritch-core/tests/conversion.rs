extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_core::conversion::{FromValue, IntoEldritchResult, ToValue};

#[test]
fn test_conversion_i64() {
    let val = 42i64.to_value();
    assert!(matches!(val, Value::Int(42)));
    let extracted: i64 = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, 42);

    // Invalid conversion
    let err = i64::from_value(&Value::String(String::from("not int"))).unwrap_err();
    assert!(err.contains("Expected Int, got str"));
}

#[test]
fn test_conversion_f64() {
    let val = 42.5f64.to_value();
    assert!(matches!(val, Value::Float(f) if f == 42.5));
    let extracted: f64 = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, 42.5);

    // Coercion from Int
    let extracted_from_int: f64 = FromValue::from_value(&Value::Int(42)).unwrap();
    assert_eq!(extracted_from_int, 42.0);

    // Invalid conversion
    let err = f64::from_value(&Value::String(String::from("not float"))).unwrap_err();
    assert!(err.contains("Expected Float or Int, got str"));
}

#[test]
fn test_conversion_string() {
    let val = String::from("hello").to_value();
    assert!(matches!(val, Value::String(ref s) if s == "hello"));
    let extracted: String = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, "hello");

    let err = String::from_value(&Value::Int(1)).unwrap_err();
    assert!(err.contains("Expected String, got int"));
}

#[test]
fn test_conversion_bool() {
    let val = true.to_value();
    assert!(matches!(val, Value::Bool(true)));
    let extracted: bool = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, true);

    let err = bool::from_value(&Value::Int(1)).unwrap_err();
    assert!(err.contains("Expected Bool, got int"));
}

#[test]
fn test_conversion_bytes() {
    let bytes = vec![1, 2, 3];
    let val = bytes.clone().to_value();
    assert!(matches!(val, Value::Bytes(ref b) if b == &bytes));
    let extracted: Vec<u8> = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, bytes);

    let err = Vec::<u8>::from_value(&Value::Int(1)).unwrap_err();
    assert!(err.contains("Expected Bytes, got int"));
}

#[test]
fn test_conversion_vec() {
    let vec = vec![1i64, 2, 3];
    let val = vec.clone().to_value();
    if let Value::List(l) = &val {
        let list = l.read();
        assert_eq!(list.len(), 3);
    } else {
        panic!("Expected list");
    }

    let extracted: Vec<i64> = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, vec);

    // From Tuple
    let tuple_val = Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let extracted_tuple: Vec<i64> = FromValue::from_value(&tuple_val).unwrap();
    assert_eq!(extracted_tuple, vec);

    let err = Vec::<i64>::from_value(&Value::Int(1)).unwrap_err();
    assert!(err.contains("Expected List or Tuple, got int"));
}

#[test]
fn test_conversion_btreemap() {
    let mut map = BTreeMap::new();
    map.insert(String::from("a"), 1i64);
    map.insert(String::from("b"), 2i64);

    let val = map.clone().to_value();
    if let Value::Dictionary(d) = &val {
        let dict = d.read();
        assert_eq!(dict.len(), 2);
    } else {
        panic!("Expected dict");
    }

    let extracted: BTreeMap<String, i64> = FromValue::from_value(&val).unwrap();
    assert_eq!(extracted, map);

    let err = BTreeMap::<String, i64>::from_value(&Value::Int(1)).unwrap_err();
    assert!(err.contains("Expected Dictionary, got int"));
}

#[test]
fn test_conversion_option() {
    let val_some = Some(42i64).to_value();
    assert!(matches!(val_some, Value::Int(42)));
    let extracted_some: Option<i64> = FromValue::from_value(&val_some).unwrap();
    assert_eq!(extracted_some, Some(42));

    let val_none: Option<i64> = None;
    let val_none_val = val_none.to_value();
    assert!(matches!(val_none_val, Value::None));
    let extracted_none: Option<i64> = FromValue::from_value(&val_none_val).unwrap();
    assert_eq!(extracted_none, None);
}

#[test]
fn test_conversion_unit() {
    let val = ().to_value();
    assert!(matches!(val, Value::None));
}

#[test]
fn test_conversion_value() {
    let val = Value::Int(42);
    let to_val = val.clone().to_value();
    assert!(matches!(to_val, Value::Int(42)));
    let from_val: Value = FromValue::from_value(&val).unwrap();
    assert!(matches!(from_val, Value::Int(42)));
}

#[test]
fn test_into_eldritch_result() {
    let res: Result<i64, String> = Ok(42);
    let val = res.into_eldritch_result().unwrap();
    assert!(matches!(val, Value::Int(42)));

    let err: Result<i64, String> = Err(String::from("error"));
    let val_err = err.into_eldritch_result().unwrap_err();
    assert_eq!(val_err, "error");
}
