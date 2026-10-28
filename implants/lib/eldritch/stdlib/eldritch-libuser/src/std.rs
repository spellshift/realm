use crate::UserLibrary;
pub mod info_impl;
pub mod list_impl;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(UserLibrary)]
pub struct StdUserLibrary;

impl UserLibrary for StdUserLibrary {
    fn info(&self) -> Result<BTreeMap<String, Value>, String> {
        info_impl::get_user().map_err(|e| e.to_string())
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        list_impl::list_users().map_err(|e| e.to_string())
    }
}
