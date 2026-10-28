use crate::LinuxLibrary;
pub mod uname_impl;

use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(LinuxLibrary)]
pub struct StdLinuxLibrary;

impl LinuxLibrary for StdLinuxLibrary {
    fn uname(&self) -> Result<BTreeMap<String, String>, String> {
        uname_impl::uname()
    }
}
