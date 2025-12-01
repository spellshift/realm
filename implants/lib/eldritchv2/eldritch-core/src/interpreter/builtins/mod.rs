use crate::ast::{BuiltinFn, BuiltinFnWithKwargs, Value};
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

mod assert;
mod assert_eq;
mod bool;
mod builtins_fn;
mod bytes;
mod dir;
mod enumerate;
mod fail;
mod int;
mod len;
mod libs;
mod pprint;
mod print;
mod range;
mod str;
mod type_;

// New builtins
mod abs;
mod all;
mod any;
mod dict;
mod float;
mod list;
mod max;
mod min;
mod repr;
mod reversed;
mod set;
mod sorted;
mod tuple;
mod zip;

pub fn get_all_builtins() -> Vec<(&'static str, BuiltinFn)> {
    let mut builtins = Vec::new();

    // Existing
    builtins.push(("print", print::builtin_print as BuiltinFn));
    builtins.push(("pprint", pprint::builtin_pprint as BuiltinFn));
    builtins.push(("len", len::builtin_len as BuiltinFn));
    builtins.push(("range", range::builtin_range as BuiltinFn));
    builtins.push(("type", type_::builtin_type as BuiltinFn));
    builtins.push(("bool", bool::builtin_bool as BuiltinFn));
    builtins.push(("str", str::builtin_str as BuiltinFn));
    builtins.push(("int", int::builtin_int as BuiltinFn));
    builtins.push(("dir", dir::builtin_dir as BuiltinFn));
    builtins.push(("assert", assert::builtin_assert as BuiltinFn));
    builtins.push(("assert_eq", assert_eq::builtin_assert_eq as BuiltinFn));
    builtins.push(("fail", fail::builtin_fail as BuiltinFn));
    builtins.push(("enumerate", enumerate::builtin_enumerate as BuiltinFn));
    builtins.push(("libs", libs::builtin_libs as BuiltinFn));
    builtins.push(("builtins", builtins_fn::builtin_builtins as BuiltinFn));
    builtins.push(("bytes", bytes::builtin_bytes as BuiltinFn));

    // New
    builtins.push(("abs", abs::builtin_abs as BuiltinFn));
    builtins.push(("any", any::builtin_any as BuiltinFn));
    builtins.push(("all", all::builtin_all as BuiltinFn));
    builtins.push(("float", float::builtin_float as BuiltinFn));
    builtins.push(("list", list::builtin_list as BuiltinFn));
    builtins.push(("max", max::builtin_max as BuiltinFn));
    builtins.push(("min", min::builtin_min as BuiltinFn));
    builtins.push(("repr", repr::builtin_repr as BuiltinFn));
    builtins.push(("reversed", reversed::builtin_reversed as BuiltinFn));
    builtins.push(("set", set::builtin_set as BuiltinFn));
    builtins.push(("sorted", sorted::builtin_sorted as BuiltinFn));
    builtins.push(("tuple", tuple::builtin_tuple as BuiltinFn));
    builtins.push(("zip", zip::builtin_zip as BuiltinFn));

    builtins
}

// Separate function for kwargs builtins
pub fn get_all_builtins_with_kwargs() -> Vec<(&'static str, BuiltinFnWithKwargs)> {
    vec![("dict", dict::builtin_dict as BuiltinFnWithKwargs)]
}

// I need to handle stubs.
pub fn builtin_stub(
    _env: &alloc::rc::Rc<core::cell::RefCell<crate::ast::Environment>>,
    _args: &[Value],
) -> Result<Value, alloc::string::String> {
    Err("internal error: this function should be handled by interpreter".to_string())
}

pub fn get_stubs() -> Vec<(&'static str, BuiltinFn)> {
    vec![
        ("map", builtin_stub as BuiltinFn),
        ("filter", builtin_stub as BuiltinFn),
        ("reduce", builtin_stub as BuiltinFn),
    ]
}
