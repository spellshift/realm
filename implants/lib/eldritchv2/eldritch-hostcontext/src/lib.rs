#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod host {
    include!(concat!(env!("OUT_DIR"), "/host.rs"));
}

// Re-export generated types
pub use host::*;

use alloc::string::String;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// HostContext defines the strict interface between the Eldritch VM and the host agent.
/// It uses Protobuf-generated structs for all data exchange to ensure strict typing and serialization support.
pub trait HostContext: Send + Sync + core::fmt::Debug {
    // Filesystem
    fn list_dir(&self, req: ListDirRequest) -> Result<ListDirResponse, String>;
    fn file_read(&self, req: FileReadRequest) -> Result<FileReadResponse, String>;
    fn file_write(&self, req: FileWriteRequest) -> Result<FileWriteResponse, String>;
    fn file_remove(&self, req: FileRemoveRequest) -> Result<FileRemoveResponse, String>;

    // Process
    fn process_list(&self, req: ProcessListRequest) -> Result<ProcessListResponse, String>;
    fn process_kill(&self, req: ProcessKillRequest) -> Result<ProcessKillResponse, String>;
    fn exec(&self, req: ExecRequest) -> Result<ExecResponse, String>;

    // System Info
    fn sys_info(&self, req: SysInfoRequest) -> Result<SysInfoResponse, String>;

    // Environment
    fn env_get(&self, req: EnvGetRequest) -> Result<EnvGetResponse, String>;
    fn env_set(&self, req: EnvSetRequest) -> Result<EnvSetResponse, String>;
}
