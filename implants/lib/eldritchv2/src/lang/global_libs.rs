use super::ast::ForeignValue;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL_LIBRARIES: Mutex<BTreeMap<String, Arc<dyn ForeignValue>>> = Mutex::new(BTreeMap::new());
}

pub fn register_lib(val: impl ForeignValue + 'static) {
    let mut libs = GLOBAL_LIBRARIES.lock();
    let name = val.type_name().to_string();
    libs.insert(name, Arc::new(val));
}

pub(crate) fn get_global_libraries() -> BTreeMap<String, Arc<dyn ForeignValue>> {
    GLOBAL_LIBRARIES.lock().clone()
}
