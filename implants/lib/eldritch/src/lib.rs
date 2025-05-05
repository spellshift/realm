#![deny(warnings)]
#![allow(clippy::needless_lifetimes) /*Appears necessary for starlark macros*/]

pub mod agent;
pub mod assets;
pub mod crypto;
pub mod file;
pub mod http;
pub mod pivot;
pub mod process;
pub mod random;
pub mod regex;
mod report;
pub mod runtime;
pub mod sys;
pub mod time;

pub use runtime::{start, Runtime};

#[allow(unused_imports)]
use starlark::const_frozen_string;

macro_rules! insert_dict_kv {
    ($dict:expr, $heap:expr, $key:expr, $val:expr, String) => {
        #[allow(clippy::unnecessary_to_owned)]
        let val_val = $heap.alloc_str(&$val);
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            val_val.to_value(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, i32) => {
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u32) => {
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u64) => {
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, None) => {
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            Value::new_none(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, Vec<_>) => {
        $dict.insert_hashed(
            match const_frozen_string!($key).to_value().get_hashed() {
                Ok(v) => v,
                Err(err) => return Err(err.into_anyhow()),
            },
            $heap.alloc($val),
        );
    };
}
pub(crate) use insert_dict_kv;

macro_rules! eldritch_lib {
    ($name:ident, $t:literal) => {
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            derive_more::Display,
            starlark::values::ProvidesStaticType,
            starlark::values::NoSerialize,
            allocative::Allocative,
        )]
        #[display(fmt = stringify!($name))]
        pub struct $name;
        starlark::starlark_simple_value!($name);

        #[starlark_value(type = $t)]
        impl<'v> starlark::values::StarlarkValue<'v> for $name {
            fn get_methods() -> Option<&'static starlark::environment::Methods> {
                static RES: starlark::environment::MethodsStatic =
                    starlark::environment::MethodsStatic::new();
                RES.methods(methods)
            }
        }
    };
}
pub(crate) use eldritch_lib;
