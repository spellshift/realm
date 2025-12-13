use super::impls::*;
use quote::quote;

#[test]
fn test_expand_library_basic() {
    let attr = quote!("test_lib");
    let item = quote! {
        trait MyLib {
            #[eldritch_method]
            fn my_fn(&self, x: i64) -> Result<i64, String>;
        }
    };

    let result = expand_eldritch_library(attr, item).unwrap();
    let s = result.to_string();

    // Verify injected methods exist
    assert!(s.contains("fn _eldritch_type_name"));
    assert!(s.contains("fn _eldritch_method_names"));
    assert!(s.contains("fn _eldritch_call_method"));

    // Verify library name
    assert!(s.contains("\"test_lib\""));

    // Verify dispatch
    assert!(s.contains("my_fn"));
    assert!(s.contains("self . my_fn (x)"));
}

#[test]
fn test_expand_library_impl() {
    let attr = quote!(MyLib);
    let item = quote! {
        struct MyStruct;
    };

    let result = expand_eldritch_library_impl(attr, item).unwrap();
    let s = result.to_string();

    assert!(s.contains("impl eldritch_core :: ForeignValue for MyStruct"));
    assert!(s.contains("< Self as MyLib > :: _eldritch_type_name"));
}

#[test]
fn test_expand_library_missing_name() {
    let attr = quote!(); // Missing name
    let item = quote! {
        trait MyLib {}
    };

    let result = expand_eldritch_library(attr, item);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().to_string(),
        "Expected string literal for library name"
    );
}

#[test]
fn test_arg_parsing_str() {
    let attr = quote!("test");
    let item = quote! {
        trait MyLib {
            #[eldritch_method]
            fn handle_str(&self, s: &str) -> Result<(), String>;
        }
    };

    let result = expand_eldritch_library(attr, item).unwrap();
    let s = result.to_string();

    // Check for string allocation and borrowing
    assert!(s.contains("let s : alloc :: string :: String ="));
    assert!(s.contains("self . handle_str (& s)"));
}

#[test]
fn test_arg_parsing_option() {
    let attr = quote!("test");
    let item = quote! {
        trait MyLib {
            #[eldritch_method]
            fn handle_opt(&self, o: Option<i64>) -> Result<(), String>;
        }
    };

    let result = expand_eldritch_library(attr, item).unwrap();
    let s = result.to_string();

    // Check default handling
    assert!(s.contains("Default :: default ()"));
}
