use crate::PivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_agent::{Agent, Context};
use eldritch_core::Value;
use eldritch_libnet::{NetLibrary, std::StdNetLibrary};
use eldritch_libportal::{PortalLibrary, std::StdPortalLibrary};
use eldritch_libscan::{ScanLibrary, std::StdScanLibrary};
use eldritch_libssh::{SshLibrary, std::StdSshLibrary};
use eldritch_macros::eldritch_library_impl;
use std::sync::Arc;

#[derive(Default)]
#[eldritch_library_impl(PivotLibrary)]
pub struct StdPivotLibrary {
    pub agent: Option<Arc<dyn Agent>>,
    pub context: Option<Context>,
}

impl core::fmt::Debug for StdPivotLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdPivotLibrary")
            .field("context", &self.context)
            .finish()
    }
}

impl StdPivotLibrary {
    pub fn new(agent: Arc<dyn Agent>, context: Context) -> Self {
        Self {
            agent: Some(agent),
            context: Some(context),
        }
    }
}

impl PivotLibrary for StdPivotLibrary {
    fn create_portal(&self) -> Result<(), String> {
        let portal = StdPortalLibrary {
            agent: self.agent.clone(),
            context: self.context.clone(),
        };
        portal.create()
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
        StdSshLibrary.exec(
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
        StdSshLibrary.copy(
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
        StdScanLibrary.port(target_cidrs, ports, protocol, timeout, fd_limit)
    }

    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        StdScanLibrary.arp(target_cidrs)
    }

    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String> {
        match protocol.as_str() {
            "tcp" => StdNetLibrary.tcp(address, port, data),
            "udp" => StdNetLibrary.udp(address, port, data),
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
        StdSshLibrary.deploy(
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
