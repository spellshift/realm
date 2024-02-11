pub mod assets;
pub mod crypto;
pub mod file;
pub mod pivot;
pub mod process;
mod report;
mod runtime;
pub mod sys;
pub mod time;

pub mod pb {
    include!("eldritch.rs");
}

pub use runtime::{Broker, FileRequest, Runtime};

#[allow(unused_imports)]
use starlark::const_frozen_string;

macro_rules! insert_dict_kv {
    ($dict:expr, $heap:expr, $key:expr, $val:expr, String) => {
        #[allow(clippy::unnecessary_to_owned)]
        let val_val = $heap.alloc_str(&$val);
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            val_val.to_value(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, i32) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u32) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u64) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, None) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            Value::new_none(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, Vec<_>) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
}
pub(crate) use insert_dict_kv;
