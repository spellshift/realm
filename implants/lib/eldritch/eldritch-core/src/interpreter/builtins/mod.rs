use crate::ast::{BuiltinFn, BuiltinFnWithKwargs, Value};
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

mod assert;
mod assert_eq;
mod bool;
mod builtins_fn;
mod bytes;
mod chr;
mod dir;
mod enumerate;
mod eprint;
mod fail;
mod int;
mod len;
mod libs;
mod ord;
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
mod hex;
mod list;
mod max;
mod min;
mod repr;
mod reversed;
mod set;
mod tprint;
mod tuple;
mod zip;

// Moved from eval/
pub mod eval_builtin;
pub mod filter;
pub mod map;
pub mod reduce;
pub mod sorted;

pub fn get_all_builtins() -> Vec<(&'static str, BuiltinFn)> {
    vec![
        // Existing
        ("print", print::builtin_print as BuiltinFn),
        ("eprint", eprint::builtin_eprint as BuiltinFn),
        ("pprint", pprint::builtin_pprint as BuiltinFn),
        ("tprint", tprint::builtin_tprint as BuiltinFn),
        ("len", len::builtin_len as BuiltinFn),
        ("range", range::builtin_range as BuiltinFn),
        ("type", type_::builtin_type as BuiltinFn),
        ("bool", bool::builtin_bool as BuiltinFn),
        ("str", str::builtin_str as BuiltinFn),
        ("int", int::builtin_int as BuiltinFn),
        ("dir", dir::builtin_dir as BuiltinFn),
        ("assert", assert::builtin_assert as BuiltinFn),
        ("assert_eq", assert_eq::builtin_assert_eq as BuiltinFn),
        ("fail", fail::builtin_fail as BuiltinFn),
        ("enumerate", enumerate::builtin_enumerate as BuiltinFn),
        ("libs", libs::builtin_libs as BuiltinFn),
        ("builtins", builtins_fn::builtin_builtins as BuiltinFn),
        ("bytes", bytes::builtin_bytes as BuiltinFn),
        ("chr", chr::builtin_chr as BuiltinFn),
        ("ord", ord::builtin_ord as BuiltinFn),
        // New
        ("abs", abs::builtin_abs as BuiltinFn),
        ("any", any::builtin_any as BuiltinFn),
        ("all", all::builtin_all as BuiltinFn),
        ("float", float::builtin_float as BuiltinFn),
        ("hex", hex::builtin_hex as BuiltinFn),
        ("list", list::builtin_list as BuiltinFn),
        ("max", max::builtin_max as BuiltinFn),
        ("min", min::builtin_min as BuiltinFn),
        ("repr", repr::builtin_repr as BuiltinFn),
        ("reversed", reversed::builtin_reversed as BuiltinFn),
        ("set", set::builtin_set as BuiltinFn),
        ("tuple", tuple::builtin_tuple as BuiltinFn),
        ("zip", zip::builtin_zip as BuiltinFn),
    ]
}

// Separate function for kwargs builtins
pub fn get_all_builtins_with_kwargs() -> Vec<(&'static str, BuiltinFnWithKwargs)> {
    vec![("dict", dict::builtin_dict as BuiltinFnWithKwargs)]
}

// I need to handle stubs.
pub fn builtin_stub(
    _env: &alloc::sync::Arc<spin::RwLock<crate::ast::Environment>>,
    _args: &[Value],
) -> Result<Value, alloc::string::String> {
    Err("internal error: this function should be handled by interpreter".to_string())
}

pub fn get_stubs() -> Vec<(&'static str, BuiltinFn)> {
    vec![
        ("map", builtin_stub as BuiltinFn),
        ("filter", builtin_stub as BuiltinFn),
        ("reduce", builtin_stub as BuiltinFn),
        ("sorted", builtin_stub as BuiltinFn),
        ("eval", builtin_stub as BuiltinFn),
    ]
}
