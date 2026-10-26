use crate::NetLibrary;
pub mod interfaces_impl;
pub mod sockets_impl;
pub mod tcp_impl;
pub mod udp_impl;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(NetLibrary)]
pub struct StdNetLibrary;

impl NetLibrary for StdNetLibrary {
    fn interfaces(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        interfaces_impl::get_ip().map_err(|e| e.to_string())
    }

    fn conns(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        sockets_impl::netstat()
    }

    fn tcp(&self, address: String, port: i64, data: String) -> Result<String, String> {
        tcp_impl::tcp(address, port as i32, data).map_err(|e| e.to_string())
    }

    fn udp(&self, address: String, port: i64, data: String) -> Result<String, String> {
        udp_impl::udp(address, port as i32, data).map_err(|e| e.to_string())
    }
}
