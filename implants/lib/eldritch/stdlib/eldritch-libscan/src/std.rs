use crate::ScanLibrary;
pub mod arp_impl;
pub mod port_impl;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ScanLibrary)]
pub struct StdScanLibrary;

impl ScanLibrary for StdScanLibrary {
    fn arp(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        arp_impl::arp_scan(target_cidrs).map_err(|e| e.to_string())
    }

    fn ports(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let ports_i32: Vec<i32> = ports.into_iter().map(|p| p as i32).collect();
        port_impl::port_scan(target_cidrs, ports_i32, protocol, timeout as i32, fd_limit)
            .map_err(|e| e.to_string())
    }
}
