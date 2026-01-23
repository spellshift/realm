use super::ProcessLibrary;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod info_impl;
pub mod kill_impl;
pub mod list_impl;
pub mod name_impl;
pub mod netstat_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct StdProcessLibrary;

impl ProcessLibrary for StdProcessLibrary {
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
        info_impl::info(pid)
    }

    fn kill(&self, pid: i64) -> Result<(), String> {
        kill_impl::kill(pid)
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        list_impl::list()
    }

    fn name(&self, pid: i64) -> Result<String, String> {
        name_impl::name(pid)
    }

    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        netstat_impl::netstat()
    }
}
