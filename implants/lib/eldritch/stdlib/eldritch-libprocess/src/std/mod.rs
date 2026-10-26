use super::ProcessLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_libnet::{NetLibrary, std::StdNetLibrary};
use eldritch_macros::eldritch_library_impl;

pub mod env_impl;
pub mod info_impl;
pub mod kill_impl;
pub mod list_impl;
pub mod name_impl;
pub mod pid_impl;
pub mod signal_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct StdProcessLibrary;

impl ProcessLibrary for StdProcessLibrary {
    fn env(&self) -> Result<BTreeMap<String, String>, String> {
        env_impl::get_env().map_err(|e| e.to_string())
    }

    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
        info_impl::info(pid)
    }

    fn kill(&self, pid: i64) -> Result<(), String> {
        kill_impl::kill(pid)
    }

    fn list(&self, include_env: Option<bool>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        list_impl::list(include_env)
    }

    fn name(&self, pid: i64) -> Result<String, String> {
        name_impl::name(pid)
    }

    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        StdNetLibrary.sockets()
    }

    fn pid(&self) -> Result<i64, String> {
        pid_impl::get_pid()
            .map(|pid| pid as i64)
            .map_err(|e| e.to_string())
    }

    fn signal(&self, pid: i64, signal: i64) -> Result<(), String> {
        signal_impl::signal(pid, signal)
    }
}
