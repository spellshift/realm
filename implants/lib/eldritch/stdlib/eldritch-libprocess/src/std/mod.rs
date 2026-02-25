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

// Module for shared Solaris process utilities
#[cfg(target_os = "solaris")]
pub mod solaris_proc {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct TimeStruc {
        pub tv_sec: i64, // time_t
        pub tv_nsec: i64, // long
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct PsInfo {
        pub pr_flag: i32,
        pub pr_nlwp: i32,
        pub pr_nzomb: i32,
        pub pr_pid: i32, // pid_t is usually i32
        pub pr_ppid: i32,
        pub pr_pgid: i32,
        pub pr_sid: i32,
        pub pr_uid: u32, // uid_t is usually u32
        pub pr_euid: u32,
        pub pr_gid: u32, // gid_t is usually u32
        pub pr_egid: u32,
        pub pr_addr: u64, // uintptr_t
        pub pr_size: u64, // size_t
        pub pr_rssize: u64,
        pub pr_ttydev: u64, // dev_t (can vary, usually u64 on modern solaris)
        pub pr_pctcpu: u16,
        pub pr_pctmem: u16,
        pub _pad: [u8; 4], // alignment padding might be needed?
        pub pr_start: TimeStruc,
        pub pr_time: TimeStruc,
        pub pr_ctime: TimeStruc,
        pub pr_fname: [u8; 16],
        pub pr_psargs: [u8; 80],
        pub pr_wstat: i32,
        pub pr_argc: i32,
        pub pr_argv: u64,
        pub pr_envp: u64,
        pub pr_dmodel: i8,
        // ... rest ignored
    }

    pub fn read_psinfo(pid: i32) -> Result<PsInfo, String> {
        let path = format!("/proc/{}/psinfo", pid);
        let mut file = File::open(&path).map_err(|_| format!("Process {} not found", pid))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        // Helper to read u32/i32
        let read_i32 = |offset: usize| -> i32 {
            let bytes = &buffer[offset..offset+4];
            i32::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u32 = |offset: usize| -> u32 {
            let bytes = &buffer[offset..offset+4];
            u32::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u64 = |offset: usize| -> u64 {
            let bytes = &buffer[offset..offset+8];
            u64::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u16 = |offset: usize| -> u16 {
            let bytes = &buffer[offset..offset+2];
            u16::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_i8 = |offset: usize| -> i8 {
            i8::from_ne_bytes([buffer[offset]])
        };
        let read_time = |offset: usize| -> TimeStruc {
            TimeStruc {
                tv_sec: i64::from_ne_bytes(buffer[offset..offset+8].try_into().unwrap()),
                tv_nsec: i64::from_ne_bytes(buffer[offset+8..offset+16].try_into().unwrap()),
            }
        };

        if buffer.len() < 260 { // minimal size we expect
             return Err(format!("Failed to read psinfo for process {}", pid));
        }

        // Verify PID match (at offset 12)
        let pr_pid = read_i32(12);
        if pr_pid != pid {
             return Err(format!("PID mismatch in psinfo: expected {}, got {}", pid, pr_pid));
        }

        let mut fname = [0u8; 16];
        fname.copy_from_slice(&buffer[136..136+16]);

        let mut psargs = [0u8; 80];
        psargs.copy_from_slice(&buffer[152..152+80]);

        Ok(PsInfo {
            pr_flag: read_i32(0),
            pr_nlwp: read_i32(4),
            pr_nzomb: read_i32(8),
            pr_pid,
            pr_ppid: read_i32(16),
            pr_pgid: read_i32(20),
            pr_sid: read_i32(24),
            pr_uid: read_u32(28),
            pr_euid: read_u32(32),
            pr_gid: read_u32(36),
            pr_egid: read_u32(40),
            pr_addr: read_u64(48),
            pr_size: read_u64(56),
            pr_rssize: read_u64(64),
            pr_ttydev: read_u64(72),
            pr_pctcpu: read_u16(80),
            pr_pctmem: read_u16(82),
            _pad: [0; 4],
            pr_start: read_time(88),
            pr_time: read_time(104),
            pr_ctime: read_time(120),
            pr_fname: fname,
            pr_psargs: psargs,
            pr_wstat: read_i32(232),
            pr_argc: read_i32(236),
            pr_argv: read_u64(240),
            pr_envp: read_u64(248),
            pr_dmodel: read_i8(256),
        })
    }
}


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
