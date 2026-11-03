use crate::LinuxLibrary;
pub mod attr_impl;
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

    fn chattr(&self, path: String, attrs: String) -> Result<(), String> {
        attr_impl::chattr(path, attrs)
    }
}
