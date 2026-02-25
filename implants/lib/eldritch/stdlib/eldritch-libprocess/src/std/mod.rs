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
        pub pr_pid: i32,
        pub pr_ppid: i32,
        pub pr_pgid: i32,
        pub pr_sid: i32,
        pub pr_uid: u32,
        pub pr_euid: u32,
        pub pr_gid: u32,
        pub pr_egid: u32,
        pub pr_addr: u64,
        pub pr_size: u64,
        pub pr_rssize: u64,
        pub pr_ttydev: u64,
        pub pr_pctcpu: u16,
        pub pr_pctmem: u16,
        pub _pad: [u8; 4],
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
    }

    pub fn read_psinfo(pid: i32) -> Result<PsInfo, String> {
        let path = format!("/proc/{}/psinfo", pid);
        let mut file = File::open(&path).map_err(|_| format!("Process {} not found", pid))?;

        let mut buffer = [0u8; 1024];
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;

        if n < 200 {
             return Err(format!("Failed to read psinfo for process {} (too small: {})", pid, n));
        }

        let buffer = &buffer[..n];
        let is_64bit = std::mem::size_of::<usize>() == 8;

        // Helper to read u32/i32
        let read_i32 = |offset: usize| -> i32 {
            if offset + 4 > buffer.len() { return 0; }
            let bytes = &buffer[offset..offset+4];
            i32::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u32 = |offset: usize| -> u32 {
            if offset + 4 > buffer.len() { return 0; }
            let bytes = &buffer[offset..offset+4];
            u32::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u64 = |offset: usize| -> u64 {
            if offset + 8 > buffer.len() { return 0; }
            let bytes = &buffer[offset..offset+8];
            u64::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_u32_as_u64 = |offset: usize| -> u64 {
            if offset + 4 > buffer.len() { return 0; }
            let bytes = &buffer[offset..offset+4];
            u32::from_ne_bytes(bytes.try_into().unwrap()) as u64
        };
        let read_u16 = |offset: usize| -> u16 {
            if offset + 2 > buffer.len() { return 0; }
            let bytes = &buffer[offset..offset+2];
            u16::from_ne_bytes(bytes.try_into().unwrap())
        };
        let read_i8 = |offset: usize| -> i8 {
            if offset + 1 > buffer.len() { return 0; }
            i8::from_ne_bytes([buffer[offset]])
        };
        let read_time = |offset: usize| -> TimeStruc {
            if is_64bit {
                if offset + 16 > buffer.len() { return TimeStruc { tv_sec: 0, tv_nsec: 0 }; }
                TimeStruc {
                    tv_sec: i64::from_ne_bytes(buffer[offset..offset+8].try_into().unwrap()),
                    tv_nsec: i64::from_ne_bytes(buffer[offset+8..offset+16].try_into().unwrap()),
                }
            } else {
                if offset + 8 > buffer.len() { return TimeStruc { tv_sec: 0, tv_nsec: 0 }; }
                TimeStruc {
                    tv_sec: i32::from_ne_bytes(buffer[offset..offset+4].try_into().unwrap()) as i64,
                    tv_nsec: i32::from_ne_bytes(buffer[offset+4..offset+8].try_into().unwrap()) as i64,
                }
            }
        };

        // Dynamic PID offset detection
        let mut pr_pid_offset = 12; // Default
        let mut found_pid = false;

        // If PID is 0, we trust offset 12 because scanning for 0 is ambiguous
        if pid == 0 {
            found_pid = true;
        } else {
            // Scan first 64 bytes for PID
            for offset in (0..64).step_by(4) {
                if read_i32(offset) == pid {
                    pr_pid_offset = offset;
                    found_pid = true;
                    // println!("DEBUG: Found PID {} at offset {}", pid, offset);
                    break;
                }
            }
        }

        if !found_pid {
             // Fallback: try to read at 12 anyway and report error if mismatch
             let val_at_12 = read_i32(12);
             return Err(format!("PID mismatch in psinfo: expected {}, not found in header (offset 12 has {})", pid, val_at_12));
        }

        let pr_pid = read_i32(pr_pid_offset);

        // Calculate other offsets relative to pr_pid
        // pr_pid is at offset 12 usually.
        // pr_ppid (4) -> +4
        // pr_pgid (4) -> +8
        // pr_sid (4) -> +12
        // pr_uid (4) -> +16
        // pr_euid (4) -> +20
        // pr_gid (4) -> +24
        // pr_egid (4) -> +28
        // pr_addr (ptr) -> +32 (aligned to 8 if 64-bit)

        let off = pr_pid_offset;
        let pr_ppid = read_i32(off + 4);
        let pr_pgid = read_i32(off + 8);
        let pr_sid = read_i32(off + 12);
        let pr_uid = read_u32(off + 16);
        let pr_euid = read_u32(off + 20);
        let pr_gid = read_u32(off + 24);
        let pr_egid = read_u32(off + 28);

        let pr_egid_end = off + 32;
        let pr_addr_off = if is_64bit {
            (pr_egid_end + 7) & !7 // Align to 8
        } else {
            pr_egid_end
        };

        let (pr_addr, pr_size, pr_rssize, pr_ttydev, pr_pctcpu, pr_pctmem, pr_start_off) = if is_64bit {
            (
                read_u64(pr_addr_off),
                read_u64(pr_addr_off + 8),
                read_u64(pr_addr_off + 16),
                read_u64(pr_addr_off + 24),
                read_u16(pr_addr_off + 32),
                read_u16(pr_addr_off + 34),
                pr_addr_off + 40, // 36 + 4 pad -> 40
            )
        } else {
            (
                read_u32_as_u64(pr_addr_off),
                read_u32_as_u64(pr_addr_off + 4),
                read_u32_as_u64(pr_addr_off + 8),
                read_u64(pr_addr_off + 12), // dev_t 64-bit
                read_u16(pr_addr_off + 20),
                read_u16(pr_addr_off + 22),
                pr_addr_off + 28, // 24 + 4 pad?
            )
        };

        // Time structs size
        let time_size = if is_64bit { 16 } else { 8 };
        // pr_start, pr_time, pr_ctime
        let pr_fname_off = pr_start_off + (3 * time_size);
        let pr_psargs_off = pr_fname_off + 16; // PRFNSZ = 16

        let mut fname = [0u8; 16];
        if pr_fname_off + 16 <= buffer.len() {
            fname.copy_from_slice(&buffer[pr_fname_off..pr_fname_off+16]);
        }

        let mut psargs = [0u8; 80];
        if pr_psargs_off + 80 <= buffer.len() {
            psargs.copy_from_slice(&buffer[pr_psargs_off..pr_psargs_off+80]);
        }

        // Additional fields at end
        // pr_wstat, pr_argc, pr_argv, pr_envp, pr_dmodel
        // Offset calculation:
        // pr_psargs_off + 80 = pr_wstat_off
        let pr_wstat_off = pr_psargs_off + 80;

        let (pr_wstat, pr_argc, pr_argv, pr_envp, pr_dmodel) = if is_64bit {
            (
                read_i32(pr_wstat_off),
                read_i32(pr_wstat_off + 4),
                read_u64(pr_wstat_off + 8),
                read_u64(pr_wstat_off + 16),
                read_i8(pr_wstat_off + 24),
            )
        } else {
            (
                read_i32(pr_wstat_off),
                read_i32(pr_wstat_off + 4),
                read_u32_as_u64(pr_wstat_off + 8),
                read_u32_as_u64(pr_wstat_off + 12),
                read_i8(pr_wstat_off + 16),
            )
        };

        // Reconstruct PsInfo.
        // Note: pr_flag, pr_nlwp, pr_nzomb might be before pr_pid.
        // We only found pr_pid. We can try to read them backward if pr_pid_offset >= 12.
        let (pr_flag, pr_nlwp, pr_nzomb) = if pr_pid_offset >= 12 {
            (
                read_i32(pr_pid_offset - 12),
                read_i32(pr_pid_offset - 8),
                read_i32(pr_pid_offset - 4),
            )
        } else {
            (0, 0, 0)
        };

        Ok(PsInfo {
            pr_flag,
            pr_nlwp,
            pr_nzomb,
            pr_pid,
            pr_ppid,
            pr_pgid,
            pr_sid,
            pr_uid,
            pr_euid,
            pr_gid,
            pr_egid,
            pr_addr,
            pr_size,
            pr_rssize,
            pr_ttydev,
            pr_pctcpu,
            pr_pctmem,
            _pad: [0; 4],
            pr_start: read_time(pr_start_off),
            pr_time: read_time(pr_start_off + time_size),
            pr_ctime: read_time(pr_start_off + 2 * time_size),
            pr_fname: fname,
            pr_psargs: psargs,
            pr_wstat,
            pr_argc,
            pr_argv,
            pr_envp,
            pr_dmodel,
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
