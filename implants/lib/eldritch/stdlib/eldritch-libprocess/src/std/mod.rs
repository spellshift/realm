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
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        // Basic sanity check on size
        if buffer.len() < 200 {
             return Err(format!("Failed to read psinfo for process {} (too small: {})", pid, buffer.len()));
        }

        // Determine offsets based on pointer width
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
        // For 32-bit read, extend to u64
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

        // Offsets
        // 0-40: same for both (11 i32/u32)
        // 64-bit:
        // 44-48: pad
        // 48: pr_addr (8)
        // 56: pr_size (8)
        // 64: pr_rssize (8)
        // 72: pr_ttydev (8)
        // 80: pr_pctcpu (2)
        // ...
        // 88: pr_start (16)

        // 32-bit:
        // 44: pr_addr (4)
        // 48: pr_size (4)
        // 52: pr_rssize (4)
        // 56: pr_ttydev (4) (dev_t is 32-bit on 32-bit Solaris typically, or 64-bit aligned? Let's assume 32-bit for now or check header)
        // Actually, dev_t is often 64-bit even on 32-bit Solaris since 2.5?
        // If dev_t is 64-bit (8 bytes):
        // 56: pr_ttydev (8)
        // 64: pr_pctcpu (2)
        // 66: pr_pctmem (2)
        // 68: pad?
        // 72: pr_start (8) (time_t 4 + long 4)

        // Verify PID match (at offset 12)
        let pr_pid = read_i32(12);
        if pr_pid != pid {
             // If PID mismatches, offsets might be totally wrong or file is wrong.
             // But we opened /proc/PID/psinfo.
             return Err(format!("PID mismatch in psinfo: expected {}, got {}", pid, pr_pid));
        }

        let (pr_addr, pr_size, pr_rssize, pr_ttydev, pr_pctcpu, pr_pctmem, pr_start_off, pr_fname_off) = if is_64bit {
            (
                read_u64(48),
                read_u64(56),
                read_u64(64),
                read_u64(72),
                read_u16(80),
                read_u16(82),
                88, // pr_start
                136 // pr_fname
            )
        } else {
            // 32-bit offsets (assuming dev_t is 64-bit)
            // 44: pr_addr (4)
            // 48: pr_size (4)
            // 52: pr_rssize (4)
            // 56: pr_ttydev (8) ??
            // Let's try to detect based on buffer size or align?
            // If dev_t is 32-bit:
            // 56: pr_ttydev (4)
            // 60: pr_pctcpu (2)
            // ...
            // Let's assume standard ILP32 with largefile/dev support.
            // On Solaris 10+, dev_t is 64-bit (u_longlong_t).
            // But padding?
            // 44: pr_addr (4)
            // 48: pr_size (4)
            // 52: pr_rssize (4)
            // 56: pr_ttydev (8) (aligned?) 56 is div by 8.
            // 64: pr_pctcpu (2)
            // 66: pr_pctmem (2)
            // 68: pad?
            // 72: pr_start (8)
            (
                read_u32_as_u64(44),
                read_u32_as_u64(48),
                read_u32_as_u64(52),
                read_u64(56), // dev_t 64-bit
                read_u16(64),
                read_u16(66),
                72, // pr_start
                72 + 8 + 8 + 8 + 16 // start(8)+time(8)+ctime(8)+fname(16) -> 120?
                // Wait, structure:
                // timestruc_t (8 bytes: 4+4)
                // pr_start (8) at 72
                // pr_time (8) at 80
                // pr_ctime (8) at 88
                // pr_fname (16) at 96
                // pr_psargs (80) at 112
                // pr_wstat (4) at 192
                // pr_argc (4) at 196
                // pr_argv (4) at 200
                // pr_envp (4) at 204
                // pr_dmodel (1) at 208
            )
        };

        // Recalculate fname/psargs offsets for 32-bit if needed.
        let fname_off = if is_64bit { 136 } else { 96 };
        let psargs_off = if is_64bit { 152 } else { 112 };

        let mut fname = [0u8; 16];
        if fname_off + 16 <= buffer.len() {
            fname.copy_from_slice(&buffer[fname_off..fname_off+16]);
        }

        let mut psargs = [0u8; 80];
        if psargs_off + 80 <= buffer.len() {
            psargs.copy_from_slice(&buffer[psargs_off..psargs_off+80]);
        }

        // Additional fields at end
        let (pr_wstat, pr_argc, pr_argv, pr_envp, pr_dmodel) = if is_64bit {
            (
                read_i32(232),
                read_i32(236),
                read_u64(240),
                read_u64(248),
                read_i8(256),
            )
        } else {
            (
                read_i32(192),
                read_i32(196),
                read_u32_as_u64(200),
                read_u32_as_u64(204),
                read_i8(208),
            )
        };

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
            pr_addr,
            pr_size,
            pr_rssize,
            pr_ttydev,
            pr_pctcpu,
            pr_pctmem,
            _pad: [0; 4],
            pr_start: read_time(pr_start_off),
            pr_time: read_time(pr_start_off + (if is_64bit { 16 } else { 8 })),
            pr_ctime: read_time(pr_start_off + (if is_64bit { 32 } else { 16 })),
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
