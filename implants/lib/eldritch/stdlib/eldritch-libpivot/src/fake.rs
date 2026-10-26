use super::PivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_libnet::{NetLibrary, fake::NetLibraryFake};
use eldritch_libportal::{PortalLibrary, fake::PortalLibraryFake};
use eldritch_libscan::{ScanLibrary, fake::ScanLibraryFake};
use eldritch_libssh::{SshLibrary, fake::SshLibraryFake};
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(PivotLibrary)]
pub struct PivotLibraryFake;

impl PivotLibrary for PivotLibraryFake {
    fn create_portal(&self) -> Result<(), String> {
        PortalLibraryFake.create()
    }

    fn ssh_exec(
        &self,
        target: String,
        port: i64,
        command: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<BTreeMap<String, Value>, String> {
        SshLibraryFake.exec(
            target,
            port,
            command,
            username,
            password,
            key,
            key_password,
            timeout,
        )
    }

    fn ssh_copy(
        &self,
        target: String,
        port: i64,
        src: String,
        dst: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<String, String> {
        SshLibraryFake.copy(
            target,
            port,
            src,
            dst,
            username,
            password,
            key,
            key_password,
            timeout,
        )
    }

    fn port_scan(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        ScanLibraryFake.port(target_cidrs, ports, protocol, timeout, fd_limit)
    }

    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        ScanLibraryFake.arp(target_cidrs)
    }

    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String> {
        match protocol.as_str() {
            "tcp" => NetLibraryFake.tcp(address, port, data),
            "udp" => NetLibraryFake.udp(address, port, data),
            _ => Err("Protocol not supported please use: udp or tcp.".to_string()),
        }
    }

    fn ssh_deploy(
        &self,
        ips: Vec<String>,
        credentials: Vec<BTreeMap<String, Value>>,
        cmd: String,
        privesc_cmd: Option<String>,
        payload: Option<Vec<u8>>,
        payload_dst: Option<String>,
        timeout: Option<i64>,
        retries: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        SshLibraryFake.deploy(
            ips,
            credentials,
            cmd,
            privesc_cmd,
            payload,
            payload_dst,
            timeout,
            retries,
        )
    }
}
